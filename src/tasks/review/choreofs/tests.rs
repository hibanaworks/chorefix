//! Drive the production effect receiver with independent Endpoint callers.
mod repeated_rejection;
use super::{global::*, imp::RepositoryArguments, localside};
use crate::shell::imp::{bus::Bus, service::Domain};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use serde_json::{Value, json};

#[test]
fn review_reply_retains_the_response_and_rejects_the_opposite_outcome() {
    use super::imp::ReviewReply;
    use hibana::runtime::wire::{Payload, WireEncode, WirePayload};

    let approved = br#"{"success":true,"feedback":"merge this revision"}"#;
    let rejected = br#"{"success":false,"feedback":"revise this change"}"#;
    assert!(ReviewReply::<true>::from_bytes(approved).is_ok());
    assert!(ReviewReply::<false>::from_bytes(rejected).is_ok());
    assert!(ReviewReply::<true>::from_bytes(rejected).is_err());
    assert!(ReviewReply::<false>::from_bytes(approved).is_err());
    for malformed in [b"{}".as_slice(), b"null", br#"{"success":"true"}"#] {
        assert!(ReviewReply::<true>::validate_payload(Payload::new(malformed)).is_err());
        assert!(ReviewReply::<false>::validate_payload(Payload::new(malformed)).is_err());
    }
    let response = ReviewReply::<true>::from_bytes(approved).unwrap();
    let mut encoded = [0; 128];
    let written = response.encode_into(&mut encoded).unwrap();
    assert_eq!(&encoded[..written], approved);
}

struct NoEffects;
impl Domain for NoEffects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        panic!("rejected request reached the effect: {agent} {tool} {arguments}")
    }
}

#[tokio::test(flavor = "current_thread")]
async fn repository_receiver_rejects_substitution_and_wrong_target_before_effect() {
    for (admitted, actual) in [
        (FEATURE_A, FEATURE_B),
        (FEATURE_B, FEATURE_B),
        (FEATURE_A, "user_authentication "),
    ] {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let p = programs();
            let session = SessionId::new(6201);
            let (bus, _reader, _terminals) = Bus::open(
                ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"],
                [(session, REPO_ENDPOINT)],
            );
            let target = super::imp::ReviewTarget::default();
            target.record(FEATURE_A);
            let mut owner_slab = vec![0; 1024 * 1024];
            let mut owner_storage = SessionKitStorage::uninit();
            let owner = owner_storage
                .init()
                .rendezvous(&mut owner_slab, &bus.transport)
                .unwrap();
            let mut guest_slab = vec![0; 1024 * 1024];
            let mut guest_storage = SessionKitStorage::uninit();
            let guest = guest_storage
                .init()
                .rendezvous(&mut guest_slab, &bus.transport)
                .unwrap();
            guest
                .set_resolver(
                    &p.review_wasi,
                    ResolverRef::<SELECT_REVIEW>::decision_state(
                        &target,
                        super::imp::ReviewTarget::decide,
                    ),
                )
                .unwrap();
            let mut service = owner.enter(session, &p.repo).unwrap();
            let repository = owner.enter(session, &p.repo_endpoint).unwrap();
            let mut a = guest.enter(session, &p.a_wasi).unwrap();
            let mut reviewer = guest.enter(session, &p.review_wasi).unwrap();
            let [mut review, _review_private] = reviewer.par().unwrap();
            let [mut task, _private] = a.par().unwrap();
            let selected = RepositoryArguments::<3>::from_bytes(
                &serde_json::to_vec(&json!({"pr_id":FEATURE_A})).unwrap(),
            )
            .unwrap();
            review.send::<FirstA>(&selected).await.unwrap();
            review.send::<FirstA>(&selected).await.unwrap();
            review.send::<FirstA>(&selected).await.unwrap();
            review.send::<FirstA>(&selected).await.unwrap();
            review.send::<FirstA>(&selected).await.unwrap();
            task.recv::<FirstA>().await.unwrap();
            let admitted = RepositoryArguments::<1>::from_bytes(
                &serde_json::to_vec(&json!({"feature_name":admitted})).unwrap(),
            )
            .unwrap();
            let substituted = RepositoryArguments::<1>::from_bytes(
                &serde_json::to_vec(&json!({"feature_name":actual})).unwrap(),
            )
            .unwrap();

            let (result, held) = tokio::join!(
                localside::repo::run(&mut service, repository, &NoEffects),
                async {
                    task.send::<ImplementA>(&admitted).await.unwrap();
                    let mut repository = task.recv::<RepoLoan<'_>>().await.unwrap();
                    assert!(owner.enter(session, &p.repo_endpoint).is_err());
                    assert!(guest.enter(session, &p.repo_endpoint).is_err());
                    repository
                        .send::<ImplementOnRepo>(&substituted)
                        .await
                        .unwrap();
                    repository
                }
            );
            assert_eq!(
                result.unwrap_err().to_string(),
                "repository implementation arguments differ"
            );
            assert!(owner.enter(session, &p.repo_endpoint).is_err());
            assert!(guest.enter(session, &p.repo_endpoint).is_err());
            drop(held);
        })
        .await
        .expect("repository rejection stalled");
    }
}

struct EchoEffect;
impl Domain for EchoEffect {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        assert_eq!((agent, tool), ("DEVELOPER_A", "implement_feature"));
        assert_eq!(arguments, json!({"feature_name":FEATURE_A}));
        Ok(json!({"success":true,"agent":agent,"tool":tool,"arguments":arguments}))
    }
}

#[tokio::test(flavor = "current_thread")]
async fn repository_receiver_executes_the_exact_received_request_and_returns_its_result() {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let p = programs();
        let session = SessionId::new(6202);
        let (bus, _reader, _terminals) = Bus::open(
            ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"],
            [(session, REPO_ENDPOINT)],
        );
        let target = super::imp::ReviewTarget::default();
        target.record(FEATURE_A);
        let mut owner_slab = vec![0; 1024 * 1024];
        let mut owner_storage = SessionKitStorage::uninit();
        let owner = owner_storage
            .init()
            .rendezvous(&mut owner_slab, &bus.transport)
            .unwrap();
        let mut guest_slab = vec![0; 1024 * 1024];
        let mut guest_storage = SessionKitStorage::uninit();
        let guest = guest_storage
            .init()
            .rendezvous(&mut guest_slab, &bus.transport)
            .unwrap();
        guest
            .set_resolver(
                &p.review_wasi,
                ResolverRef::<SELECT_REVIEW>::decision_state(
                    &target,
                    super::imp::ReviewTarget::decide,
                ),
            )
            .unwrap();
        let mut service = owner.enter(session, &p.repo).unwrap();
        let repository = owner.enter(session, &p.repo_endpoint).unwrap();
        let mut a = guest.enter(session, &p.a_wasi).unwrap();
        let mut reviewer = guest.enter(session, &p.review_wasi).unwrap();
        let [mut review, _review_private] = reviewer.par().unwrap();
        let [mut task, _private] = a.par().unwrap();
        let selected = RepositoryArguments::<3>::from_bytes(
            &serde_json::to_vec(&json!({"pr_id":FEATURE_A})).unwrap(),
        )
        .unwrap();
        review.send::<FirstA>(&selected).await.unwrap();
        review.send::<FirstA>(&selected).await.unwrap();
        review.send::<FirstA>(&selected).await.unwrap();
        review.send::<FirstA>(&selected).await.unwrap();
        review.send::<FirstA>(&selected).await.unwrap();
        task.recv::<FirstA>().await.unwrap();
        let arguments = RepositoryArguments::<1>::from_bytes(
            &serde_json::to_vec(&json!({"feature_name":FEATURE_A})).unwrap(),
        )
        .unwrap();

        tokio::select! {
            result = localside::repo::run(&mut service, repository, &EchoEffect) => {
                panic!("repository owner stopped before the checked prefix: {result:?}");
            }
            result = async {
                task.send::<ImplementA>(&arguments).await?;
                let mut repository = task.recv::<RepoLoan<'_>>().await?;
                repository.send::<ImplementOnRepo>(&arguments).await?;
                let response = repository.recv::<RepoResult<'_>>().await?.to_vec();
                assert_eq!(serde_json::from_slice::<Value>(&response).unwrap(), json!({
                    "success":true,"agent":"DEVELOPER_A","tool":"implement_feature",
                    "arguments":{"feature_name":FEATURE_A}
                }));
                task.send::<RepoLoan<'_>>(repository).await?;
                task.send::<ReadyA<'_>>(&response.as_slice()).await?;
                Ok::<_, crate::shell::ShellError>(())
            } => result.unwrap(),
        }
    })
    .await
    .expect("repository effect/result prefix stalled");
}
