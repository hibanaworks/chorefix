//! Terminal and simulator RPC correlation. No task progress or permissions.
use crate::{
    shell::{ShellError, ShellResult},
    tasks::cicd::choreofs::imp::Domain,
};
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout},
    sync::{Mutex, mpsc, oneshot},
};

pub struct Bus {
    output: Mutex<Stdout>,
    sequence: Cell<u64>,
    replies: RefCell<BTreeMap<u64, oneshot::Sender<Value>>>,
}
pub struct Reader {
    input: tokio::io::Lines<BufReader<Stdin>>,
    terminals: BTreeMap<&'static str, mpsc::UnboundedSender<(u64, String)>>,
}
pub struct Terminal {
    pub agent: &'static str,
    commands: mpsc::UnboundedReceiver<(u64, String)>,
    // The current command RPC identifier, not a task phase or permission.
    identifier: Option<u64>,
}
impl Bus {
    pub fn open() -> (Self, Reader, [Terminal; 3]) {
        let (a, ar) = mpsc::unbounded_channel();
        let (b, br) = mpsc::unbounded_channel();
        let (c, cr) = mpsc::unbounded_channel();
        let terminals = BTreeMap::from([("DEVELOPER", a), ("TESTER", b), ("DEPLOYER", c)]);
        (
            Self {
                output: Mutex::new(tokio::io::stdout()),
                sequence: Cell::new(0),
                replies: RefCell::new(BTreeMap::new()),
            },
            Reader {
                input: BufReader::new(tokio::io::stdin()).lines(),
                terminals,
            },
            [
                Terminal {
                    agent: "DEVELOPER",
                    commands: ar,
                    identifier: None,
                },
                Terminal {
                    agent: "TESTER",
                    commands: br,
                    identifier: None,
                },
                Terminal {
                    agent: "DEPLOYER",
                    commands: cr,
                    identifier: None,
                },
            ],
        )
    }
    pub async fn emit(&self, value: Value) -> ShellResult<()> {
        let mut bytes =
            serde_json::to_vec(&value).map_err(|error| ShellError::message(error.to_string()))?;
        bytes.push(b'\n');
        let mut output = self.output.lock().await;
        output.write_all(&bytes).await?;
        output.flush().await?;
        Ok(())
    }
}
impl Reader {
    pub async fn run(&mut self, bus: &Bus) -> ShellResult<()> {
        while let Some(line) = self.input.next_line().await? {
            let value: Value = serde_json::from_str(&line)
                .map_err(|error| ShellError::message(error.to_string()))?;
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
                    self.terminals
                        .get(agent)
                        .ok_or_else(|| ShellError::message("unknown terminal"))?
                        .send((id, command.to_owned()))
                        .map_err(|_| ShellError::message("terminal input closed"))?;
                }
                Some("simulator_result") => {
                    bus.replies
                        .borrow_mut()
                        .remove(&id)
                        .ok_or_else(|| ShellError::message("unexpected simulator result"))?
                        .send(value["result"].clone())
                        .map_err(|_| ShellError::message("simulator result receiver closed"))?;
                }
                _ => return Err(ShellError::message("unknown direct WASI RPC message")),
            }
        }
        Err(ShellError::message(
            "terminal transport closed before guest completion",
        ))
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
    pub async fn finish(&self, bus: &Bus, status: i32) -> ShellResult<()> {
        bus.emit(json!({"type":"exit", "agent":self.agent, "id":self.identifier, "status":status}))
            .await
    }
}
impl Domain for Bus {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let id = self
            .sequence
            .get()
            .checked_add(1)
            .ok_or("RPC id overflow")?;
        self.sequence.set(id);
        let (sender, reply) = oneshot::channel();
        self.replies.borrow_mut().insert(id, sender);
        self.emit(
            json!({"type":"execute", "id":id, "agent":agent, "tool":tool, "arguments":arguments}),
        )
        .await
        .map_err(|error| error.to_string())?;
        reply
            .await
            .map_err(|_| "simulator direction closed".to_owned())
    }
}
