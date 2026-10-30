use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::Cell;

/// The actual tool target or the model stream's observed completion event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Implement,
    Merge,
    ReviewA,
    ReviewB,
    ModelFinished,
}
pub trait Request {
    fn operation(&self) -> Operation;
}
pub trait Domain {
    type Call: Request;
    type Receipt;
    fn request(&self, agent: &'static str) -> impl Future<Output = Result<Self::Call, String>>;
    fn execute(
        &self,
        agent: &'static str,
        call: Self::Call,
    ) -> impl Future<Output = Result<(Self::Receipt, Value), String>>;
    fn reply(
        &self,
        receipt: Self::Receipt,
        result: Value,
    ) -> impl Future<Output = Result<(), String>>;
}

/// The original response at this explicit resolver point, never a progress flag.
#[derive(Default)]
pub(crate) struct ReviewInput(Cell<Option<Value>>);
impl ReviewInput {
    pub(crate) fn record(&self, result: Value) {
        self.0.replace(Some(result));
    }
}
pub(crate) fn decide(input: &ReviewInput) -> Result<DecisionArm, ResolverError> {
    let result = input.0.take();
    let decision = match result.as_ref().and_then(|v| v["success"].as_bool()) {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    };
    input.0.set(result);
    decision
}
