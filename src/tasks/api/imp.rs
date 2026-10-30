use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Migrate,
    Implement,
    Test,
}
pub trait Request {
    fn operation(&self) -> Operation;
}
pub trait Domain {
    type Call: Request;
    fn request(&self, agent: &'static str) -> impl Future<Output = Result<Self::Call, String>>;
    fn execute(
        &self,
        agent: &'static str,
        call: Self::Call,
    ) -> impl Future<Output = Result<Value, String>>;
}

/// Only the unmodified test response is stored. Endpoint owns progress.
#[derive(Default)]
pub(crate) struct TestInput(Cell<Option<Value>>);
impl TestInput {
    pub(crate) fn record(&self, result: Value) {
        self.0.replace(Some(result));
    }
}
pub(crate) fn decide(input: &TestInput) -> Result<DecisionArm, ResolverError> {
    let result = input.0.take();
    let decision = match result
        .as_ref()
        .and_then(|result| result["success"].as_bool())
    {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.0.set(result);
    decision
}
