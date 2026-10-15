//! Host RPC queues. Endpoint operations in localside authorize domain calls.

use super::domain::{Domain, Work, WorkRequest};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, Stdin, Stdout},
    sync::{Mutex, mpsc, oneshot},
};

struct Call {
    id: u64,
    tool: String,
    arguments: Value,
}

pub struct PhilosopherCall(Call);
pub struct CodeCall(Call);

impl WorkRequest for PhilosopherCall {
    fn work(&self) -> Work {
        match self.0.tool.as_str() {
            "think" => Work::Think,
            "eat" => Work::Eat,
            _ => unreachable!("validated philosopher tool name"),
        }
    }
}

impl crate::tasks::codebase::Request for CodeCall {
    fn operation(&self) -> crate::tasks::codebase::Operation {
        use crate::tasks::codebase::Operation;
        match self.0.tool.as_str() {
            "design_feature" => Operation::Design,
            "implement_code" => Operation::Implement,
            "commit_changes" => Operation::Commit,
            "run_local_tests" => Operation::Test,
            _ => unreachable!("validated code tool name"),
        }
    }
}

pub struct Bridge {
    calls: BTreeMap<String, Mutex<mpsc::UnboundedReceiver<Call>>>,
    output: Mutex<Stdout>,
    replies: RefCell<BTreeMap<u64, oneshot::Sender<Value>>>,
}

pub struct Reader {
    input: Lines<BufReader<Stdin>>,
    calls: BTreeMap<String, mpsc::UnboundedSender<Call>>,
}

pub fn open(agents: &[String]) -> (Bridge, Reader) {
    let mut senders = BTreeMap::new();
    let mut receivers = BTreeMap::new();
    for agent in agents {
        let (sender, receiver) = mpsc::unbounded_channel();
        senders.insert(agent.clone(), sender);
        receivers.insert(agent.clone(), Mutex::new(receiver));
    }
    (
        Bridge {
            calls: receivers,
            output: Mutex::new(tokio::io::stdout()),
            replies: RefCell::default(),
        },
        Reader {
            input: BufReader::new(tokio::io::stdin()).lines(),
            calls: senders,
        },
    )
}

impl Reader {
    pub async fn run(&mut self, bridge: &Bridge) -> Result<(), String> {
        while let Some(line) = self
            .input
            .next_line()
            .await
            .map_err(|error| error.to_string())?
        {
            let value: Value = serde_json::from_str(&line).map_err(|error| error.to_string())?;
            let id = value["id"].as_u64().ok_or("RPC id missing")?;
            match value["type"].as_str() {
                Some("close_role") => {
                    let agent = value["agent"].as_str().ok_or("RPC agent missing")?;
                    self.calls
                        .remove(agent)
                        .ok_or("unknown participant direction")?;
                }
                Some("call") => {
                    let agent = value["agent"].as_str().ok_or("RPC agent missing")?;
                    let tool = value["tool"].as_str().ok_or("RPC tool missing")?.to_owned();
                    let call = Call {
                        id,
                        tool,
                        arguments: value["arguments"].clone(),
                    };
                    self.calls
                        .get(agent)
                        .ok_or("unknown participant")?
                        .send(call)
                        .map_err(|_| "participant input closed")?;
                }
                Some("simulator_result") => {
                    bridge
                        .replies
                        .borrow_mut()
                        .remove(&id)
                        .ok_or("unexpected simulator reply")?
                        .send(value["result"].clone())
                        .map_err(|_| "simulator receiver closed")?;
                }
                _ => return Err("unknown RPC message".into()),
            }
        }
        Err("model transport closed before protocol completion".into())
    }
}

impl Bridge {
    async fn request_call(&self, agent: &'static str) -> Result<Call, String> {
        self.calls
            .get(agent)
            .expect("attached participant")
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(|| "participant request direction closed".into())
    }

    async fn emit(&self, value: Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        let mut output = self.output.lock().await;
        output
            .write_all(&bytes)
            .await
            .map_err(|error| error.to_string())?;
        output.flush().await.map_err(|error| error.to_string())
    }

    async fn execute_call(&self, agent: &'static str, call: Call) -> Result<Value, String> {
        // Only localside calls this, after the typed endpoint operation succeeds.
        let (sender, reply) = oneshot::channel();
        if self.replies.borrow_mut().insert(call.id, sender).is_some() {
            return Err("duplicate in-flight RPC id".into());
        }
        self.emit(json!({"type":"execute", "id":call.id, "agent":agent,
                    "tool":call.tool, "arguments":call.arguments}))
            .await?;
        let result = reply
            .await
            .map_err(|_| "simulator reply direction closed")?;
        self.emit(json!({"type":"tool_result", "id":call.id, "result":result}))
            .await?;
        Ok(result)
    }
}

impl Domain for Bridge {
    type Error = String;
    type Call = PhilosopherCall;

    async fn request(&self, agent: &'static str) -> Result<PhilosopherCall, String> {
        let call = self.request_call(agent).await?;
        if !matches!(call.tool.as_str(), "think" | "eat") {
            return Err("unknown philosopher tool".into());
        }
        Ok(PhilosopherCall(call))
    }

    async fn execute(&self, agent: &'static str, call: PhilosopherCall) -> Result<(), String> {
        let result = self.execute_call(agent, call.0).await?;
        if result["success"] == true {
            Ok(())
        } else {
            Err(format!("domain operation failed: {result}"))
        }
    }
}

impl crate::tasks::codebase::Domain for Bridge {
    type Call = CodeCall;

    async fn request(&self, agent: &'static str) -> Result<CodeCall, String> {
        let call = self.request_call(agent).await?;
        if !matches!(
            call.tool.as_str(),
            "design_feature" | "implement_code" | "commit_changes" | "run_local_tests"
        ) {
            return Err("unknown code tool".into());
        }
        Ok(CodeCall(call))
    }

    async fn execute(&self, agent: &'static str, call: CodeCall) -> Result<Value, String> {
        self.execute_call(agent, call.0).await
    }
}

impl Bridge {
    pub async fn report(&self, result: Result<(), String>) -> Result<(), String> {
        let message = match result {
            Ok(()) => json!({"type":"run_result", "status":"protocol_completed"}),
            Err(error) => json!({"type":"run_result", "status":"failed", "error":error}),
        };
        self.emit(message).await
    }
}
