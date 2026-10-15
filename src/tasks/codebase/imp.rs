use hibana::runtime::resolver::{DecisionArm, ResolverError};
use serde_json::Value;
use std::cell::RefCell;

/// Which of a module's two inbound Endpoint requests was received.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Borrower {
    First,
    Second,
}

impl hibana::runtime::wire::WireEncode for Borrower {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, hibana::runtime::wire::CodecError> {
        let byte = match self {
            Self::First => 0_u8,
            Self::Second => 1_u8,
        };
        hibana::runtime::wire::WireEncode::encode_into(&byte, out)
    }
}
impl hibana::runtime::wire::WirePayload for Borrower {
    const SCHEMA_ID: u32 = 0x5452_0001;
    type Decoded<'a> = Self;
    fn validate_payload(
        input: hibana::runtime::wire::Payload<'_>,
    ) -> Result<(), hibana::runtime::wire::CodecError> {
        match input.as_bytes() {
            [0 | 1] => Ok(()),
            [] => Err(hibana::runtime::wire::CodecError::Truncated),
            _ => Err(hibana::runtime::wire::CodecError::Malformed),
        }
    }
    fn decode_validated_payload(input: hibana::runtime::wire::Payload<'_>) -> Self {
        match input.as_bytes() {
            [0] => Self::First,
            [1] => Self::Second,
            _ => unreachable!("validated borrower"),
        }
    }
}

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

    #[test]
    fn borrower_codec_rejects_unknown_variants_and_extra_bytes() {
        use hibana::runtime::wire::{Payload, WireEncode, WirePayload};
        for borrower in [Borrower::First, Borrower::Second] {
            let mut bytes = [0; 1];
            assert_eq!(borrower.encode_into(&mut bytes).unwrap(), 1);
            Borrower::validate_payload(Payload::new(&bytes)).unwrap();
            assert_eq!(
                Borrower::decode_validated_payload(Payload::new(&bytes)),
                borrower
            );
        }
        assert!(Borrower::validate_payload(Payload::new(&[2])).is_err());
        assert!(Borrower::validate_payload(Payload::new(&[0, 0])).is_err());
        assert!(Borrower::validate_payload(Payload::new(&[])).is_err());
    }

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
        let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
        let kit = storage.init();
        let rendezvous = kit.rendezvous(&mut slab, InMemoryTransport::new()).unwrap();
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
        let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
        let kit = storage.init();
        let rendezvous = kit.rendezvous(&mut slab, InMemoryTransport::new()).unwrap();
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
