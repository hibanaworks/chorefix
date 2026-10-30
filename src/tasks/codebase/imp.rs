use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Design,
    Implement,
    Commit,
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

/// A model tool's actual request kind and unmodified simulator response.
/// Neither field records or authorizes protocol progress.
pub(crate) struct ToolResponse {
    pub(crate) operation: Operation,
    pub(crate) result: Value,
}

pub(crate) fn response(
    operation: Operation,
    result: Value,
) -> Result<ToolResponse, crate::RunError<String>> {
    match result["success"].as_bool() {
        Some(true) => Ok(ToolResponse { operation, result }),
        Some(false) if operation == Operation::Test => Ok(ToolResponse { operation, result }),
        Some(false) => Err(crate::RunError::Domain(format!(
            "domain operation failed: {result}"
        ))),
        None => Err(crate::RunError::Domain(
            "domain response has no success outcome".into(),
        )),
    }
}

#[derive(Default)]
pub(crate) struct TestInput(RefCell<Option<ToolResponse>>);

impl TestInput {
    pub(crate) fn record(&self, response: ToolResponse) {
        self.0.replace(Some(response));
    }
}

pub(crate) fn decide(input: &TestInput) -> Result<DecisionArm, ResolverError> {
    let response = input.0.borrow();
    let response = response.as_ref().ok_or_else(ResolverError::reject)?;
    match (response.operation, response.result["success"].as_bool()) {
        (Operation::Design, Some(true)) | (Operation::Test, Some(false)) => Ok(DecisionArm::Right),
        (Operation::Test, Some(true)) => Ok(DecisionArm::Left),
        _ => Err(ResolverError::reject()),
    }
}

/// The unconsumed model requests themselves, not a second permission table.
/// Removing a request is acknowledged by `Submitted` in the global protocol.
pub(crate) struct Requests<C> {
    pub(crate) pending: RefCell<std::collections::VecDeque<(u8, C)>>,
    pub(crate) results: [TestInput; 7],
    pub(crate) available: tokio::sync::Notify,
}
impl<C> Requests<C> {
    pub(crate) fn new() -> Self {
        Self {
            pending: RefCell::new(std::collections::VecDeque::new()),
            results: std::array::from_fn(|_| TestInput::default()),
            available: tokio::sync::Notify::new(),
        }
    }
    pub(crate) fn all_passed(&self) -> bool {
        self.results.iter().all(|input| {
            let value = input.0.borrow();
            value.as_ref().is_some_and(|value| {
                value.operation == Operation::Test && value.result["success"] == true
            })
        })
    }
}
pub(crate) fn ready<const ROLE: u8, C: Request>(
    input: &Requests<C>,
) -> Result<DecisionArm, ResolverError> {
    let calls = input.pending.borrow();
    let Some((_, call)) = calls.iter().find(|(role, _)| *role == ROLE) else {
        return Ok(DecisionArm::Right);
    };
    if call.operation() != Operation::Commit {
        return Err(ResolverError::reject());
    }
    if input.results[ROLE as usize]
        .0
        .borrow()
        .as_ref()
        .is_some_and(|r| r.operation == Operation::Test && r.result["success"] == true)
    {
        return Ok(DecisionArm::Right);
    }
    Ok(DecisionArm::Left)
}
pub(crate) fn complete<C>(input: &Requests<C>) -> Result<DecisionArm, ResolverError> {
    Ok(if input.all_passed() {
        DecisionArm::Left
    } else {
        DecisionArm::Right
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imp::transport::InMemoryTransport;
    use hibana::{
        g,
        runtime::{
            SessionKitStorage,
            ids::SessionId,
            program::{RoleProgram, project},
            resolver::ResolverRef,
        },
    };

    #[tokio::test(flavor = "current_thread")]
    async fn controller_cannot_send_against_the_resolver_on_roll_reentry() {
        type Passed = g::Msg<1, ()>;
        type Retry = g::Msg<2, ()>;
        let protocol = g::route(g::send::<0, 1, Passed>(), g::send::<0, 1, Retry>())
            .resolve::<100>()
            .roll();
        let program: RoleProgram<0> = project(&protocol);
        let peer_program: RoleProgram<1> = project(&protocol);
        let input = TestInput::default();
        input.record(ToolResponse {
            operation: Operation::Test,
            result: serde_json::json!({"success":false}),
        });
        let mut slab = [0; 32 * 1024];
        let transport_storage = InMemoryTransport::new([]);
        let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
        let kit = storage.init();
        let rendezvous = kit.rendezvous(&mut slab, &transport_storage).unwrap();
        rendezvous
            .set_resolver(&program, ResolverRef::<100>::decision_state(&input, decide))
            .unwrap();
        let mut endpoint = rendezvous.enter(SessionId::new(4), &program).unwrap();
        let mut peer = rendezvous.enter(SessionId::new(4), &peer_program).unwrap();
        endpoint.send::<Retry>(&()).await.unwrap();
        peer.recv::<Retry>().await.unwrap();
        input.record(ToolResponse {
            operation: Operation::Test,
            result: serde_json::json!({"success":true}),
        });
        endpoint.send::<Passed>(&()).await.unwrap();
        peer.recv::<Passed>().await.unwrap();
        assert!(endpoint.send::<Retry>(&()).await.is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dropped_preview_keeps_domain_result_and_choice_available() {
        type Passed = g::Msg<1, ()>;
        type Retry = g::Msg<2, ()>;
        let protocol =
            g::route(g::send::<0, 1, Passed>(), g::send::<0, 1, Retry>()).resolve::<100>();
        let program: RoleProgram<0> = project(&protocol);
        let peer_program: RoleProgram<1> = project(&protocol);
        let input = TestInput::default();
        input.record(ToolResponse {
            operation: Operation::Test,
            result: serde_json::json!({"success":false}),
        });
        let mut slab = [0; 32 * 1024];
        let transport_storage = InMemoryTransport::new([]);
        let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
        let kit = storage.init();
        let rendezvous = kit.rendezvous(&mut slab, &transport_storage).unwrap();
        rendezvous
            .set_resolver(&program, ResolverRef::<100>::decision_state(&input, decide))
            .unwrap();
        let mut endpoint = rendezvous.enter(SessionId::new(2), &program).unwrap();
        let mut peer = rendezvous.enter(SessionId::new(2), &peer_program).unwrap();
        let preview = endpoint.offer().await.unwrap();
        assert_eq!(preview.label(), 2);
        drop(preview);
        let preview = endpoint.offer().await.unwrap();
        assert_eq!(preview.label(), 2);
        preview.send::<Retry>(&()).await.unwrap();
        peer.recv::<Retry>().await.unwrap();
        assert!(endpoint.send::<Passed>(&()).await.is_err());
    }
}
