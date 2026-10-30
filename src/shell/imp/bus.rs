//! The external connection owns terminal output and simulator RPC correlation.
use crate::{
    shell::imp::service::Domain,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{ids::SessionId, transport::EndpointBinding};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout},
    sync::{mpsc, oneshot},
};

pub struct Bus {
    pub(crate) transport: crate::imp::transport::InMemoryTransport,
    outgoing: mpsc::UnboundedSender<Outbound>,
}

/// Messages to the single owner of the external JSON connection.
enum Outbound {
    Events {
        values: Vec<Value>,
        flushed: oneshot::Sender<()>,
    },
    Execute {
        observations: Vec<Value>,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
        reply: oneshot::Sender<Value>,
    },
}

pub struct ExternalIo {
    input: tokio::io::Lines<BufReader<Stdin>>,
    output: Stdout,
    outgoing: mpsc::UnboundedReceiver<Outbound>,
    sequence: u64,
    replies: BTreeMap<u64, oneshot::Sender<Value>>,
    terminals: BTreeMap<&'static str, mpsc::UnboundedSender<(u64, String)>>,
}
pub struct Terminal {
    pub agent: &'static str,
    commands: mpsc::UnboundedReceiver<(u64, String)>,
    // The current command RPC identifier, not a task phase or permission.
    identifier: Option<u64>,
}
impl Bus {
    /// Open the sole connection to a fresh external simulator trial.
    pub fn open<const N: usize, const R: usize>(
        agents: [&'static str; N],
        resources: [(SessionId, u8); R],
    ) -> (Self, ExternalIo, [Terminal; N]) {
        let mut senders = BTreeMap::new();
        let terminals = agents.map(|agent| {
            let (sender, commands) = mpsc::unbounded_channel();
            senders.insert(agent, sender);
            Terminal {
                agent,
                commands,
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
        let (outgoing, requests) = mpsc::unbounded_channel();
        (
            Self {
                transport: crate::imp::transport::InMemoryTransport::new(bindings),
                outgoing,
            },
            ExternalIo {
                input: BufReader::new(tokio::io::stdin()).lines(),
                output: tokio::io::stdout(),
                outgoing: requests,
                sequence: 0,
                replies: BTreeMap::new(),
                terminals: senders,
            },
            terminals,
        )
    }
    pub async fn emit(&self, value: Value) -> ShellResult<()> {
        let mut values = self.transport.take_observations();
        values.push(value);
        let (flushed, completion) = oneshot::channel();
        self.outgoing
            .send(Outbound::Events { values, flushed })
            .map_err(|_| ShellError::message("external output closed"))?;
        completion
            .await
            .map_err(|_| ShellError::message("external output was not flushed"))
    }
}
impl ExternalIo {
    pub async fn emit(&mut self, bus: &Bus, value: Value) -> ShellResult<()> {
        let mut values = bus.transport.take_observations();
        values.push(value);
        self.write(values).await
    }

    async fn write(&mut self, values: impl IntoIterator<Item = Value>) -> ShellResult<()> {
        for value in values {
            let mut bytes = serde_json::to_vec(&value)
                .map_err(|error| ShellError::message(error.to_string()))?;
            bytes.push(b'\n');
            self.output.write_all(&bytes).await?;
        }
        self.output.flush().await?;
        Ok(())
    }

    pub async fn run(&mut self, bus: &Bus) -> ShellResult<()> {
        loop {
            tokio::select! {
                line = self.input.next_line() => {
                    let line = line?.ok_or_else(|| ShellError::message("terminal transport closed before guest completion"))?;
                    let value: Value = serde_json::from_str(&line)
                        .map_err(|error| ShellError::message(error.to_string()))?;
                    let id = value["id"].as_u64().ok_or_else(|| ShellError::message("RPC id missing"))?;
                    match value["type"].as_str() {
                        Some("command") => {
                            let agent = value["agent"].as_str().ok_or_else(|| ShellError::message("RPC agent missing"))?;
                            let command = value["command"].as_str().ok_or_else(|| ShellError::message("RPC command missing"))?;
                            let terminal = self.terminals.get(agent).ok_or_else(|| ShellError::message("unknown terminal"))?;
                            if terminal.send((id, command.to_owned())).is_err() {
                                self.emit(bus, json!({"type":"terminal_closed", "id":id, "agent":agent})).await?;
                            }
                        }
                        Some("simulator_result") => {
                            self.replies.remove(&id)
                                .ok_or_else(|| ShellError::message("unexpected simulator result"))?
                                .send(value["result"].clone())
                                .map_err(|_| ShellError::message("simulator result receiver closed"))?;
                        }
                        _ => return Err(ShellError::message("unknown direct WASI RPC message")),
                    }
                }
                request = self.outgoing.recv() => {
                    match request.ok_or_else(|| ShellError::message("external output closed"))? {
                        Outbound::Events { values, flushed } => {
                            self.write(values).await?;
                            flushed.send(()).map_err(|_| ShellError::message("external output receiver closed"))?;
                        }
                        Outbound::Execute { mut observations, agent, tool, arguments, reply } => {
                            self.sequence = self.sequence.checked_add(1).ok_or_else(|| ShellError::message("RPC id overflow"))?;
                            let id = self.sequence;
                            if self.replies.insert(id, reply).is_some() {
                                return Err(ShellError::message("duplicate simulator RPC id"));
                            }
                            observations.push(json!({"type":"execute", "id":id, "agent":agent, "tool":tool, "arguments":arguments}));
                            self.write(observations).await?;
                        }
                    }
                }
            }
        }
    }
}
impl Terminal {
    pub async fn read_command(&mut self, bus: &Bus, output: Vec<u8>) -> ShellResult<String> {
        let stdout =
            String::from_utf8(output).map_err(|error| ShellError::message(error.to_string()))?;
        bus.emit(
            json!({"type":"terminal", "agent":self.agent, "id":self.identifier, "stdout":stdout}),
        )
        .await?;
        let (id, command) = self
            .commands
            .recv()
            .await
            .ok_or_else(|| ShellError::message("terminal input closed"))?;
        self.identifier = Some(id);
        Ok(command)
    }
    pub async fn finish(self, bus: &Bus, status: i32, output: Vec<u8>) -> ShellResult<()> {
        let stdout =
            String::from_utf8(output).map_err(|error| ShellError::message(error.to_string()))?;
        drop(self.commands);
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
        let (reply, completion) = oneshot::channel();
        self.outgoing
            .send(Outbound::Execute {
                observations: self.transport.take_observations(),
                agent,
                tool,
                arguments,
                reply,
            })
            .map_err(|_| "external simulator connection closed".to_owned())?;
        completion
            .await
            .map_err(|_| "simulator direction closed".to_owned())
    }
}
