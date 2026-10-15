use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Compile,
    Test,
    Deploy,
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
