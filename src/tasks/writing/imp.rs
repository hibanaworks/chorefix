use serde_json::Value;

/// The actual tool request, not an expected protocol position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Literature,
    Draft,
    Write,
    Review,
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

pub(crate) fn response(operation: Operation, result: Value) -> Result<(), crate::RunError<String>> {
    match result["success"].as_bool() {
        Some(true) => Ok(()),
        Some(false) if operation == Operation::Review => Ok(()),
        Some(false) => Err(crate::RunError::Domain(format!(
            "domain operation failed: {result}"
        ))),
        None => Err(crate::RunError::Domain(
            "domain response has no success outcome".into(),
        )),
    }
}
