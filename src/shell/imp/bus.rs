//! The external connection owns terminal output and simulator RPC correlation.
use crate::{
    shell::imp::service::Domain,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{ids::SessionId, transport::EndpointBinding};
use hibana_runtime::host::Io;
use serde_json::{Value, json};
use std::{
    cell::Cell,
    collections::{BTreeMap, VecDeque},
    future::poll_fn,
    io::{Stdin, Stdout},
    os::fd::AsFd,
    task::{Poll, Waker},
};

pub struct Bus {
    pub(crate) transport: crate::imp::transport::InMemoryTransport,
    connection: Cell<Connection>,
}

type Commands = (VecDeque<(u64, String)>, Option<Waker>);
type Response = (Option<Value>, Option<Waker>);

/// Bytes and correlations of the external connection, not task permissions.
#[derive(Default)]
struct Connection {
    sequence: u64,
    outgoing: VecDeque<(Vec<Value>, Option<u64>)>,
    output_waiter: Option<Waker>,
    responses: BTreeMap<u64, Response>,
    terminals: BTreeMap<&'static str, Option<Commands>>,
}

/// Own one outstanding RPC or output-flush receipt, including its cancellation.
struct Reply<'a> {
    bus: &'a Bus,
    id: u64,
}

pub struct ExternalIo {
    input: Stdin,
    output: Stdout,
}

pub struct Terminal {
    pub agent: &'static str,
    // The current command RPC identifier, not a task phase or permission.
    identifier: Option<u64>,
}

impl Bus {
    /// Open the sole connection to a fresh external simulator trial.
    pub fn open<const N: usize, const R: usize>(
        agents: [&'static str; N],
        resources: [(SessionId, u8); R],
    ) -> (Self, ExternalIo, [Terminal; N]) {
        let mut connection = Connection::default();
        let terminals = agents.map(|agent| {
            assert!(
                connection
                    .terminals
                    .insert(agent, Some(Default::default()))
                    .is_none()
            );
            Terminal {
                agent,
                identifier: None,
            }
        });
        let bindings = resources.map(|(session, role)| {
            // SAFETY: one fresh simulator trial assigns each listed role a
            // distinct physical resource. Every runtime reaches these same
            // bindings. Endpoint transfers and failures never reissue them.
            #[allow(unsafe_code)]
            let binding = unsafe { EndpointBinding::new() };
            (session, role, binding)
        });
        (
            Self {
                transport: crate::imp::transport::InMemoryTransport::new(bindings),
                connection: Cell::new(connection),
            },
            ExternalIo {
                input: std::io::stdin(),
                output: std::io::stdout(),
            },
            terminals,
        )
    }

    fn connection<T>(&self, apply: impl FnOnce(&mut Connection) -> T) -> T {
        let mut connection = self.connection.take();
        let result = apply(&mut connection);
        self.connection.set(connection);
        result
    }

    fn reply(&self) -> ShellResult<Reply<'_>> {
        let id = self
            .connection(|connection| {
                let id = connection.sequence.checked_add(1)?;
                connection.sequence = id;
                assert!(connection.responses.insert(id, (None, None)).is_none());
                Some(id)
            })
            .ok_or_else(|| ShellError::message("RPC id overflow"))?;
        Ok(Reply { bus: self, id })
    }

    fn enqueue(&self, values: Vec<Value>, flushed: Option<u64>) {
        let wake = self.connection(|connection| {
            connection.outgoing.push_back((values, flushed));
            connection.output_waiter.take()
        });
        if let Some(wake) = wake {
            wake.wake();
        }
    }

    /// Queue an immutable diagnostic observation with the I/O owner.
    pub fn observe(&self, value: Value) {
        self.enqueue(vec![value], None);
    }

    fn respond(&self, id: u64, value: Value) -> ShellResult<()> {
        let wake = self.connection(|connection| {
            let (result, wake) = connection
                .responses
                .get_mut(&id)
                .ok_or_else(|| ShellError::message("unexpected or cancelled RPC result"))?;
            if result.is_some() {
                return Err(ShellError::message("duplicate RPC result"));
            }
            *result = Some(value);
            Ok(wake.take())
        })?;
        if let Some(wake) = wake {
            wake.wake();
        }
        Ok(())
    }

    pub async fn emit(&self, value: Value) -> ShellResult<()> {
        let mut values = self.transport.take_observations();
        values.push(value);
        let reply = self.reply()?;
        self.enqueue(values, Some(reply.id));
        reply.receive().await?;
        Ok(())
    }
}

impl Reply<'_> {
    async fn receive(self) -> ShellResult<Value> {
        poll_fn(|cx| {
            self.bus.connection(|connection| {
                let (result, wake) = connection
                    .responses
                    .get_mut(&self.id)
                    .expect("an owned RPC receipt remains registered");
                if let Some(value) = result.take() {
                    Poll::Ready(Ok(value))
                } else {
                    *wake = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
        })
        .await
    }
}

impl Drop for Reply<'_> {
    fn drop(&mut self) {
        self.bus.connection(|connection| {
            connection.responses.remove(&self.id);
        });
    }
}

impl ExternalIo {
    pub async fn emit(&mut self, bus: &Bus, value: Value) -> ShellResult<()> {
        let mut output = Io::new(self.output.as_fd().try_clone_to_owned()?)?;
        let mut values = bus.transport.take_observations();
        values.push(value);
        for value in values {
            let mut bytes = serde_json::to_vec(&value)
                .map_err(|error| ShellError::message(error.to_string()))?;
            bytes.push(b'\n');
            output.write_all(&bytes).await?;
        }
        Ok(())
    }

    #[hibana_runtime::scope]
    pub async fn run(&mut self, bus: &Bus) -> ShellResult<()> {
        let mut input = Io::new(self.input.as_fd().try_clone_to_owned()?)?;
        let mut output = Io::new(self.output.as_fd().try_clone_to_owned()?)?;
        async {
            let mut buffered = Vec::new();
            let mut bytes = [0; 4096];
            loop {
                let count = input.read(&mut bytes).await?;
                if count == 0 {
                    return Err(ShellError::message(
                        "terminal transport closed before guest completion",
                    ));
                }
                buffered.extend_from_slice(&bytes[..count]);
                while let Some(end) = buffered.iter().position(|byte| *byte == b'\n') {
                    let value: Value = serde_json::from_slice(&buffered[..end])
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    buffered.drain(..=end);
                    let id = value["id"]
                        .as_u64()
                        .ok_or_else(|| ShellError::message("RPC id missing"))?;
                    match value["type"].as_str() {
                        Some("command") => {
                            let agent = value["agent"]
                                .as_str()
                                .ok_or_else(|| ShellError::message("RPC agent missing"))?;
                            let command = value["command"]
                                .as_str()
                                .ok_or_else(|| ShellError::message("RPC command missing"))?;
                            let delivery = bus.connection(|connection| {
                                let terminal = connection
                                    .terminals
                                    .get_mut(agent)
                                    .ok_or_else(|| ShellError::message("unknown terminal"))?;
                                Ok::<_, ShellError>(terminal.as_mut().map(|(commands, wake)| {
                                    commands.push_back((id, command.to_owned()));
                                    wake.take()
                                }))
                            })?;
                            match delivery {
                                Some(Some(wake)) => wake.wake(),
                                Some(None) => {}
                                None => {
                                    bus.emit(
                                        json!({"type":"terminal_closed", "id":id, "agent":agent}),
                                    )
                                    .await?
                                }
                            }
                        }
                        Some("simulator_result") => bus.respond(id, value["result"].clone())?,
                        _ => return Err(ShellError::message("unknown direct WASI RPC message")),
                    }
                }
            }
        };
        async {
            loop {
                let (values, flushed) = poll_fn(|cx| {
                    bus.connection(|connection| {
                        if let Some(values) = connection.outgoing.pop_front() {
                            Poll::Ready(values)
                        } else {
                            connection.output_waiter = Some(cx.waker().clone());
                            Poll::Pending
                        }
                    })
                })
                .await;
                for value in values {
                    let mut bytes = serde_json::to_vec(&value)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    bytes.push(b'\n');
                    output.write_all(&bytes).await?;
                }
                if let Some(id) = flushed {
                    bus.respond(id, Value::Null)?;
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "bus/tests.rs"]
mod tests;

impl Terminal {
    pub async fn read_command(&mut self, bus: &Bus, output: Vec<u8>) -> ShellResult<String> {
        let stdout =
            String::from_utf8(output).map_err(|error| ShellError::message(error.to_string()))?;
        bus.emit(
            json!({"type":"terminal", "agent":self.agent, "id":self.identifier, "stdout":stdout}),
        )
        .await?;
        let (id, command) = poll_fn(|cx| {
            bus.connection(|connection| {
                let (commands, wake) = connection
                    .terminals
                    .get_mut(self.agent)
                    .and_then(Option::as_mut)
                    .expect("an active terminal remains registered");
                if let Some(command) = commands.pop_front() {
                    Poll::Ready(command)
                } else {
                    *wake = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
        })
        .await;
        self.identifier = Some(id);
        Ok(command)
    }

    pub async fn finish(self, bus: &Bus, status: i32, output: Vec<u8>) -> ShellResult<()> {
        let stdout =
            String::from_utf8(output).map_err(|error| ShellError::message(error.to_string()))?;
        bus.connection(|connection| {
            connection
                .terminals
                .get_mut(self.agent)
                .expect("a terminal remains registered")
                .take();
        });
        bus.emit(json!({"type":"exit", "agent":self.agent, "id":self.identifier, "status":status, "stdout":stdout})).await
    }
}

impl Domain for Bus {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let reply = self.reply().map_err(|error| error.to_string())?;
        let mut values = self.transport.take_observations();
        values.push(json!({"type":"execute", "id":reply.id, "agent":agent, "tool":tool, "arguments":arguments}));
        self.enqueue(values, None);
        reply.receive().await.map_err(|error| error.to_string())
    }
}
