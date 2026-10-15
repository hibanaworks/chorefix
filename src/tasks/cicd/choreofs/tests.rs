//! Typed Endpoint requests exercise the service graph without a Wasm guest.
use super::{global::*, localside};
use crate::{
    imp::transport::InMemoryTransport,
    shell::imp::service::{Domain, Validation},
};
use hibana::{
    Endpoint,
    runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef},
};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::RefCell, time::Duration};
use tokio::sync::Notify;

#[derive(Default)]
struct Journal(RefCell<Vec<(&'static str, &'static str, Value)>>);
impl Domain for Journal {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        self.0.borrow_mut().push((agent, tool, arguments.clone()));
        Ok(json!({"success":true, "tool":tool, "arguments":arguments}))
    }
}
async fn write<const VM: u8>(endpoint: &mut Endpoint<'_, VM>, value: Value) {
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.extend_from_slice(&[b' '; 256]);
    for chunk in bytes.chunks(37) {
        endpoint
            .send::<Write>(&protocol::FdWriteReq(
                protocol::FdWrite::new(SERVICE_FD, chunk).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(
            endpoint.recv::<Written>().await.unwrap().0.written() as usize,
            chunk.len()
        );
    }
}
async fn open<const VM: u8>(endpoint: &mut Endpoint<'_, VM>, path: &[u8]) -> u16 {
    endpoint
        .send::<Open>(&protocol::PathOpenReq(
            protocol::PathOpen::new(3, 1 << 6, path)
                .unwrap()
                .with_flags(8, 0)
                .unwrap(),
        ))
        .await
        .unwrap();
    endpoint.recv::<Opened>().await.unwrap().0.errno()
}
async fn commit<const VM: u8>(endpoint: &mut Endpoint<'_, VM>) {
    endpoint
        .send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
        .await
        .unwrap();
    assert!(
        serde_json::from_slice::<Value>(endpoint.recv::<Committed<'_>>().await.unwrap()).unwrap()["success"]
            == true
    );
    endpoint.recv::<ServiceDone>().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn wasi_roles_reach_the_service_owners_only_through_projected_messages() {
    let p = programs();
    let compile = Validation::new();
    let test = Validation::new();
    let deploy = Validation::new();
    let journal = Journal::default();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, InMemoryTransport::new()).unwrap();
    rv.set_resolver(
        &p.developer,
        ResolverRef::<COMPILE_CLOSE>::decision_state(&compile, Validation::decide),
    )
    .unwrap();
    rv.set_resolver(
        &p.tester,
        ResolverRef::<TEST_CLOSE>::decision_state(&test, Validation::decide),
    )
    .unwrap();
    rv.set_resolver(
        &p.deployer,
        ResolverRef::<DEPLOY_CLOSE>::decision_state(&deploy, Validation::decide),
    )
    .unwrap();
    let session = SessionId::new(16);
    let mut a = rv.enter(session, &p.dev_wasi).unwrap();
    let mut b = rv.enter(session, &p.test_wasi).unwrap();
    let mut c = rv.enter(session, &p.deploy_wasi).unwrap();
    let mut developer = rv.enter(session, &p.developer).unwrap();
    let mut tester = rv.enter(session, &p.tester).unwrap();
    let mut deployer = rv.enter(session, &p.deployer).unwrap();
    let mut build = rv.enter(session, &p.build_server).unwrap();
    let mut env = rv.enter(session, &p.deploy_env).unwrap();
    let test_requested = Notify::new();
    let developer_closed = Notify::new();
    let clients = async {
        tokio::join!(
            async {
                test_requested.notified().await;
                assert_ne!(open(&mut a, b"service/deploy_application").await, 0);
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 0);
                write(&mut a, json!({"wrong":"application"})).await;
                a.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(4)))
                    .await
                    .unwrap();
                let rejected =
                    serde_json::from_slice::<Value>(a.recv::<Rejected<'_>>().await.unwrap())
                        .unwrap();
                assert_eq!(rejected["status"], "invalid_arguments");
                assert!(journal.0.borrow().is_empty());
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 0);
                write(&mut a, json!({"component":"application"})).await;
                commit(&mut a).await;
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 8);
                // Closed-service errors must not wait for downstream effects.
                assert_eq!(journal.0.borrow().len(), 1);
                developer_closed.notify_one();
                a.send::<ServiceExit>(&0).await.unwrap();
            },
            async {
                b.send::<Open>(&protocol::PathOpenReq(
                    protocol::PathOpen::new(3, 1 << 6, b"service/run_integration_tests")
                        .unwrap()
                        .with_flags(8, 0)
                        .unwrap(),
                ))
                .await
                .unwrap();
                let mut response = Box::pin(b.recv::<Opened>());
                assert!(
                    tokio::time::timeout(Duration::from_millis(10), &mut response)
                        .await
                        .is_err()
                );
                assert!(journal.0.borrow().is_empty());
                test_requested.notify_one();
                assert_eq!(response.await.unwrap().0.errno(), 0);
                developer_closed.notified().await;
                write(&mut b, json!({"suite":"integration"})).await;
                commit(&mut b).await;
                b.send::<ServiceExit>(&0).await.unwrap();
            },
            async {
                assert_eq!(open(&mut c, b"service/deploy_application").await, 0);
                write(&mut c, json!({"target":"production"})).await;
                commit(&mut c).await;
                c.send::<ServiceExit>(&0).await.unwrap();
            },
        );
        Ok(())
    };
    tokio::time::timeout(Duration::from_secs(3), async {
        tokio::try_join!(
            clients,
            localside::developer::run(&mut developer, &journal, &compile),
            localside::tester::run(&mut tester, &journal, &test),
            localside::deployer::run(&mut deployer, &journal, &deploy),
            localside::infrastructure::build_server(&mut build),
            localside::infrastructure::deploy_env(&mut env),
        )
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        journal
            .0
            .borrow()
            .iter()
            .map(|row| row.1)
            .collect::<Vec<_>>(),
        [
            "compile_and_test",
            "run_integration_tests",
            "deploy_application"
        ]
    );
}
