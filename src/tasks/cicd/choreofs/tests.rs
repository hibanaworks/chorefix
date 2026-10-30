//! Typed Endpoint requests exercise the service graph without a Wasm guest.
use super::{global::*, imp::Arguments, localside};
use crate::{
    imp::transport::InMemoryTransport,
    shell::imp::bus::Bus,
    shell::imp::service::{Domain, Validation},
};
use hibana::{
    Endpoint,
    runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef},
};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};
use tokio::sync::Notify;

#[derive(Default)]
struct Journal(Cell<Vec<(&'static str, &'static str, Value)>>);
impl Domain for Journal {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut events = self.0.take();
        events.push((agent, tool, arguments.clone()));
        self.0.set(events);
        Ok(json!({"success":true, "tool":tool, "arguments":arguments}))
    }
}
async fn write<const VM: u8>(endpoint: &mut Endpoint<'_, VM>, value: Value) {
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.extend_from_slice(&[b' '; 256]);
    for chunk in bytes.chunks(37) {
        endpoint.recv::<RequestReady>().await.unwrap();
        let response =
            protocol::FdWriteObjectReq(protocol::FdWrite::new(SERVICE_FD, chunk).unwrap());
        endpoint.send::<Write>(&response).await.unwrap();
        assert_eq!(
            endpoint.recv::<Written>().await.unwrap().0.written() as usize,
            chunk.len()
        );
    }
}
async fn open<const VM: u8>(endpoint: &mut Endpoint<'_, VM>, path: &[u8]) -> u16 {
    endpoint.recv::<RequestReady>().await.unwrap();
    let response = protocol::PathOpenReq(
        protocol::PathOpen::new(3, 1 << 6, path)
            .unwrap()
            .with_flags(8, 0)
            .unwrap(),
    );
    endpoint.send::<Open>(&response).await.unwrap();
    endpoint.recv::<Opened>().await.unwrap().0.errno()
}
impl Journal {
    fn events(&self) -> Vec<(&'static str, &'static str, Value)> {
        let events = self.0.take();
        let observed = events.clone();
        self.0.set(events);
        observed
    }
}

#[tokio::test(flavor = "current_thread")]
async fn wasi_roles_reach_the_service_owners_only_through_projected_messages() {
    let p = programs();
    let compile = Validation::new();
    let test = Validation::new();
    let deploy = Validation::new();
    let journal = Journal::default();
    let mut slab = vec![0; 1024 * 1024];
    let session = SessionId::new(16);
    let (bus, _, _) = Bus::open([], [(session, BUILD_SERVER), (session, DEPLOY_ENV)]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport).unwrap();
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
    let mut a = rv.enter(session, &p.dev_wasi).unwrap();
    let mut b = rv.enter(session, &p.test_wasi).unwrap();
    let mut c = rv.enter(session, &p.deploy_wasi).unwrap();
    let mut developer = rv.enter(session, &p.developer).unwrap();
    let mut tester = rv.enter(session, &p.tester).unwrap();
    let mut deployer = rv.enter(session, &p.deployer).unwrap();
    let mut build_owner = rv.enter(session, &p.build_owner).unwrap();
    let build = rv.enter(session, &p.build_server).unwrap();
    let mut deploy_owner = rv.enter(session, &p.deploy_owner).unwrap();
    let env = rv.enter(session, &p.deploy_env).unwrap();
    let test_requested = Notify::new();
    let developer_closed = Notify::new();
    let clients = async {
        tokio::join!(
            async {
                test_requested.notified().await;
                assert_ne!(open(&mut a, b"service/deploy_application").await, 0);
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 0);
                write(&mut a, json!({"wrong":"application"})).await;
                a.recv::<RequestReady>().await.unwrap();
                a.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(4)))
                    .await
                    .unwrap();
                let rejected =
                    serde_json::from_slice::<Value>(a.recv::<Rejected<'_>>().await.unwrap())
                        .unwrap();
                assert_eq!(rejected["status"], "invalid_arguments");
                assert!(journal.events().is_empty());
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 0);
                write(&mut a, json!({"component":"application"})).await;
                a.recv::<RequestReady>().await.unwrap();
                a.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
                    .await
                    .unwrap();
                let arguments = {
                    let bytes = a.recv::<Admitted<'_>>().await.unwrap();
                    Arguments::<1>::from_bytes(bytes).unwrap()
                };
                let mut resource = a.recv::<Loan<'_, BUILD_SERVER>>().await.unwrap();
                resource.send::<CompileRequest>(&arguments).await.unwrap();
                let result = resource.recv::<CompileResult<'_>>().await.unwrap().to_vec();
                assert_eq!(
                    serde_json::from_slice::<Value>(&result).unwrap()["success"],
                    true
                );
                a.send::<Loan<'_, BUILD_SERVER>>(resource).await.unwrap();
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 8);
                // Closed-service errors must not wait for downstream effects.
                assert_eq!(journal.events().len(), 1);
                developer_closed.notify_one();
                a.recv::<RequestReady>().await.unwrap();
                a.send::<ServiceExit>(&0).await.unwrap();
            },
            async {
                let mut ready = Box::pin(b.recv::<RequestReady>());
                assert!(
                    tokio::time::timeout(Duration::from_millis(10), &mut ready)
                        .await
                        .is_err()
                );
                assert!(journal.events().is_empty());
                test_requested.notify_one();
                ready.await.unwrap();
                b.send::<Open>(&protocol::PathOpenReq(
                    protocol::PathOpen::new(3, 1 << 6, b"service/run_integration_tests")
                        .unwrap()
                        .with_flags(8, 0)
                        .unwrap(),
                ))
                .await
                .unwrap();
                assert_eq!(b.recv::<Opened>().await.unwrap().0.errno(), 0);
                developer_closed.notified().await;
                write(&mut b, json!({"suite":"integration"})).await;
                b.recv::<RequestReady>().await.unwrap();
                b.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
                    .await
                    .unwrap();
                let arguments = {
                    let bytes = b.recv::<Admitted<'_>>().await.unwrap();
                    Arguments::<2>::from_bytes(bytes).unwrap()
                };
                let mut resource = b.recv::<Loan<'_, BUILD_SERVER>>().await.unwrap();
                resource.send::<TestRequest>(&arguments).await.unwrap();
                let result = resource.recv::<TestResult<'_>>().await.unwrap().to_vec();
                assert_eq!(
                    serde_json::from_slice::<Value>(&result).unwrap()["success"],
                    true
                );
                b.send::<Loan<'_, BUILD_SERVER>>(resource).await.unwrap();
                b.recv::<RequestReady>().await.unwrap();
                b.send::<ServiceExit>(&0).await.unwrap();
            },
            async {
                assert_eq!(open(&mut c, b"service/deploy_application").await, 0);
                write(&mut c, json!({"target":"production"})).await;
                c.recv::<RequestReady>().await.unwrap();
                c.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
                    .await
                    .unwrap();
                let arguments = {
                    let bytes = c.recv::<Admitted<'_>>().await.unwrap();
                    Arguments::<3>::from_bytes(bytes).unwrap()
                };
                let mut resource = c.recv::<Loan<'_, DEPLOY_ENV>>().await.unwrap();
                resource.send::<DeployRequest>(&arguments).await.unwrap();
                let result = resource.recv::<DeployResult<'_>>().await.unwrap().to_vec();
                assert_eq!(
                    serde_json::from_slice::<Value>(&result).unwrap()["success"],
                    true
                );
                c.send::<Loan<'_, DEPLOY_ENV>>(resource).await.unwrap();
                c.recv::<RequestReady>().await.unwrap();
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
            localside::infrastructure::build_server(&mut build_owner, build),
            localside::infrastructure::deploy_env(&mut deploy_owner, env),
        )
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        journal.events().iter().map(|row| row.1).collect::<Vec<_>>(),
        [
            "compile_and_test",
            "run_integration_tests",
            "deploy_application"
        ]
    );
}
