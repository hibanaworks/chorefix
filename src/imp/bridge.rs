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

/// Actual input events from the model transport, never expected task steps.
enum Input {
    Call(Call),
    ModelFinished,
}

pub struct PhilosopherCall(Call);
pub struct CodeCall(Call);
pub struct WritingCall(Call);
pub struct CicdCall(Call);
pub struct ApiCall(Call);
pub struct ReviewCall(Input);

impl crate::tasks::review::Request for ReviewCall {
    fn operation(&self) -> crate::tasks::review::Operation {
        use crate::tasks::review::Operation;
        let Input::Call(call) = &self.0 else {
            return Operation::ModelFinished;
        };
        match call.tool.as_str() {
            "implement_feature" => Operation::Implement,
            "merge_code" => Operation::Merge,
            "code_review" => match call.arguments["pr_id"].as_str() {
                Some("user_authentication") => Operation::ReviewA,
                Some("notification_feature") => Operation::ReviewB,
                _ => unreachable!("validated original review target"),
            },
            _ => unreachable!("validated original review tool"),
        }
    }
}

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

impl crate::tasks::writing::Request for WritingCall {
    fn operation(&self) -> crate::tasks::writing::Operation {
        use crate::tasks::writing::Operation;
        match self.0.tool.as_str() {
            "read_literature" => Operation::Literature,
            "draft_content" => Operation::Draft,
            "write_to_document" => Operation::Write,
            "review_own_work" => Operation::Review,
            _ => unreachable!("validated writing tool name"),
        }
    }
}

impl crate::tasks::cicd::Request for CicdCall {
    fn operation(&self) -> crate::tasks::cicd::Operation {
        use crate::tasks::cicd::Operation;
        match self.0.tool.as_str() {
            "compile_and_test" => Operation::Compile,
            "run_integration_tests" => Operation::Test,
            "deploy_application" => Operation::Deploy,
            _ => unreachable!("validated CI/CD tool name"),
        }
    }
}

impl crate::tasks::api::Request for ApiCall {
    fn operation(&self) -> crate::tasks::api::Operation {
        use crate::tasks::api::Operation;
        match self.0.tool.as_str() {
            "migrate_schema" => Operation::Migrate,
            "implement_backend" => Operation::Implement,
            "run_integration_tests" => Operation::Test,
            _ => unreachable!("validated API task tool"),
        }
    }
}

pub struct Bridge {
    calls: BTreeMap<String, Mutex<mpsc::UnboundedReceiver<Input>>>,
    output: Mutex<Stdout>,
    replies: RefCell<BTreeMap<u64, oneshot::Sender<Value>>>,
}

pub struct Reader {
    input: Lines<BufReader<Stdin>>,
    calls: BTreeMap<String, mpsc::UnboundedSender<Input>>,
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
                Some("model_finished") => {
                    let agent = value["agent"].as_str().ok_or("RPC agent missing")?;
                    self.calls
                        .remove(agent)
                        .ok_or("unknown participant direction")?
                        .send(Input::ModelFinished)
                        .map_err(|_| "participant input closed")?;
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
                        .send(Input::Call(call))
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
    async fn request_input(&self, agent: &'static str) -> Result<Input, String> {
        self.calls
            .get(agent)
            .expect("attached participant")
            .lock()
            .await
            .recv()
            .await
            .ok_or_else(|| "participant request direction closed".into())
    }

    async fn request_call(&self, agent: &'static str) -> Result<Call, String> {
        match self.request_input(agent).await? {
            Input::Call(call) => Ok(call),
            Input::ModelFinished => Err("participant request direction closed".into()),
        }
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

    async fn execute_deferred(
        &self,
        agent: &'static str,
        call: Call,
    ) -> Result<(u64, Value), String> {
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
        Ok((call.id, result))
    }

    async fn execute_call(&self, agent: &'static str, call: Call) -> Result<Value, String> {
        let (id, result) = self.execute_deferred(agent, call).await?;
        self.emit(json!({"type":"tool_result", "id":id, "result":result}))
            .await?;
        Ok(result)
    }
}

impl crate::tasks::review::Domain for Bridge {
    type Call = ReviewCall;
    type Receipt = u64;
    async fn request(&self, agent: &'static str) -> Result<ReviewCall, String> {
        let input = self.request_input(agent).await?;
        if let Input::Call(call) = &input {
            match call.tool.as_str() {
                "implement_feature" | "merge_code" => (),
                "code_review"
                    if matches!(
                        call.arguments["pr_id"].as_str(),
                        Some("user_authentication" | "notification_feature")
                    ) => {}
                _ => return Err("unknown review tool or original feature target".into()),
            }
        }
        Ok(ReviewCall(input))
    }
    async fn execute(&self, agent: &'static str, call: ReviewCall) -> Result<(u64, Value), String> {
        let Input::Call(call) = call.0 else {
            return Err("model completion is not a domain call".into());
        };
        self.execute_deferred(agent, call).await
    }
    async fn reply(&self, receipt: u64, result: Value) -> Result<(), String> {
        self.emit(json!({"type":"tool_result","id":receipt,"result":result}))
            .await
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

impl crate::tasks::writing::Domain for Bridge {
    type Call = WritingCall;
    async fn request(&self, agent: &'static str) -> Result<WritingCall, String> {
        let call = self.request_call(agent).await?;
        if !matches!(
            call.tool.as_str(),
            "read_literature" | "draft_content" | "write_to_document" | "review_own_work"
        ) {
            return Err("unknown writing tool".into());
        }
        Ok(WritingCall(call))
    }
    async fn execute(&self, agent: &'static str, call: WritingCall) -> Result<Value, String> {
        self.execute_call(agent, call.0).await
    }
}

impl crate::tasks::cicd::Domain for Bridge {
    type Call = CicdCall;
    async fn request(&self, agent: &'static str) -> Result<CicdCall, String> {
        let call = self.request_call(agent).await?;
        if !matches!(
            call.tool.as_str(),
            "compile_and_test" | "run_integration_tests" | "deploy_application"
        ) {
            return Err("unknown CI/CD tool".into());
        }
        Ok(CicdCall(call))
    }
    async fn execute(&self, agent: &'static str, call: CicdCall) -> Result<Value, String> {
        self.execute_call(agent, call.0).await
    }
}

impl crate::tasks::api::Domain for Bridge {
    type Call = ApiCall;
    async fn request(&self, agent: &'static str) -> Result<ApiCall, String> {
        let call = self.request_call(agent).await?;
        if !matches!(
            call.tool.as_str(),
            "migrate_schema" | "implement_backend" | "run_integration_tests"
        ) {
            return Err("unknown API task tool".into());
        }
        Ok(ApiCall(call))
    }
    async fn execute(&self, agent: &'static str, call: ApiCall) -> Result<Value, String> {
        self.execute_call(agent, call.0).await
    }
}
