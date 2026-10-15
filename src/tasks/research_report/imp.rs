use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::RefCell;

/// The actual requested tool and, for review, its requested section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Research,
    Write,
    References,
    Revise,
    ReviewA,
    ReviewB,
    Combine,
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
#[derive(Default)]
pub(crate) struct ReviewInput(RefCell<Option<Value>>);
impl ReviewInput {
    pub(crate) fn record(&self, result: Value) {
        self.0.replace(Some(result));
    }
}
pub(crate) fn decide(input: &ReviewInput) -> Result<DecisionArm, ResolverError> {
    match input
        .0
        .borrow()
        .as_ref()
        .and_then(|r| r["success"].as_bool())
    {
        Some(true) => Ok(DecisionArm::Left),
        Some(false) => Ok(DecisionArm::Right),
        None => Err(ResolverError::reject()),
    }
}
pub(crate) fn succeeded(result: &Value) -> Result<(), crate::RunError<String>> {
    match result["success"].as_bool() {
        Some(true) => Ok(()),
        Some(false) => Err(crate::RunError::Domain(format!(
            "domain operation failed: {result}"
        ))),
        None => Err(crate::RunError::Domain("missing domain outcome".into())),
    }
}
