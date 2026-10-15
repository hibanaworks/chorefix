//! Host RPC queues. Endpoint operations in localside authorize domain calls.

use super::domain::{Domain, Work, WorkRequest};
use futures::{
    StreamExt,
    channel::{mpsc, oneshot},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, BufRead, Write},
};

pub struct Call {
    id: u64,
    work: Work,
    arguments: Value,
}

impl WorkRequest for Call {
    fn work(&self) -> Work {
        self.work
    }
}

pub struct Bridge {
    calls: RefCell<BTreeMap<String, mpsc::UnboundedReceiver<Call>>>,
    replies: RefCell<BTreeMap<u64, oneshot::Sender<Value>>>,
}

pub struct Reader {
    input: mpsc::UnboundedReceiver<Result<Value, String>>,
    calls: BTreeMap<String, mpsc::UnboundedSender<Call>>,
}

fn emit(value: Value) -> Result<(), String> {
    let mut output = io::stdout().lock();
    writeln!(output, "{value}").map_err(|error| error.to_string())?;
    output.flush().map_err(|error| error.to_string())
}

pub fn open(agents: &[String]) -> (Bridge, Reader) {
    let mut senders = BTreeMap::new();
    let mut receivers = BTreeMap::new();
    for agent in agents {
        let (sender, receiver) = mpsc::unbounded();
        senders.insert(agent.clone(), sender);
        receivers.insert(agent.clone(), receiver);
    }
    let (sender, input) = mpsc::unbounded();
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let result = line
                .map_err(|error| error.to_string())
                .and_then(|line| serde_json::from_str(&line).map_err(|error| error.to_string()));
            if sender.unbounded_send(result).is_err() {
                break;
            }
        }
    });
    (
        Bridge {
            calls: RefCell::new(receivers),
            replies: RefCell::default(),
        },
        Reader {
            input,
            calls: senders,
        },
    )
}

impl Reader {
    pub async fn run(&mut self, bridge: &Bridge) -> Result<(), String> {
        while let Some(input) = self.input.next().await {
            let value = input?;
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
                    let work = match value["tool"].as_str() {
                        Some("think") => Work::Think,
                        Some("eat") => Work::Eat,
                        _ => return Err("unknown domain tool".into()),
                    };
                    let call = Call {
                        id,
                        work,
                        arguments: value["arguments"].clone(),
                    };
                    self.calls
                        .get(agent)
                        .ok_or("unknown participant")?
                        .unbounded_send(call)
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

impl Domain for Bridge {
    type Error = String;
    type Call = Call;

    async fn request(&self, agent: &'static str) -> Result<Call, String> {
        futures::future::poll_fn(|context| {
            self.calls
                .borrow_mut()
                .get_mut(agent)
                .expect("attached participant")
                .poll_next_unpin(context)
        })
        .await
        .ok_or_else(|| "participant request direction closed".into())
    }

    async fn execute(&self, agent: &'static str, call: Call) -> Result<(), String> {
        // Only localside calls this, after the typed endpoint operation succeeds.
        let (sender, reply) = oneshot::channel();
        if self.replies.borrow_mut().insert(call.id, sender).is_some() {
            return Err("duplicate in-flight RPC id".into());
        }
        let tool = match call.work {
            Work::Think => "think",
            Work::Eat => "eat",
        };
        emit(json!({"type":"execute", "id":call.id, "agent":agent,
                    "tool":tool, "arguments":call.arguments}))?;
        let result = reply
            .await
            .map_err(|_| "simulator reply direction closed")?;
        emit(json!({"type":"tool_result", "id":call.id, "result":result}))?;
        if result["success"] != true {
            return Err(format!("domain operation failed: {result}"));
        }
        Ok(())
    }
}

pub fn report(result: Result<(), String>) -> Result<(), String> {
    match result {
        Ok(()) => emit(json!({"type":"run_result", "status":"protocol_completed"})),
        Err(error) => emit(json!({"type":"run_result", "status":"failed", "error":error})),
    }
}
