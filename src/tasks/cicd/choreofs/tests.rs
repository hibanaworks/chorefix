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
use hibana_runtime::host::notification;
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

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

#[hibana_runtime::main]
#[test]
async fn replacing_an_admitted_compile_argument_is_rejected_before_the_effect()
-> Result<(), crate::shell::ShellError> {
    let p = programs();
    let validation = Validation::new();
    let journal = Journal::default();
    let session = SessionId::new(1621);
    let (bus, _reader, _terminals) = Bus::open([], [(session, BUILD_SERVER)]);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &bus.transport)
        .unwrap();
    rv.set_resolver(
        &p.developer,
        ResolverRef::<COMPILE_CLOSE>::decision_state(&validation, Validation::decide),
    )
    .unwrap();
    let mut developer = rv.enter(session, &p.developer).unwrap();
    let mut guest = rv.enter(session, &p.dev_wasi).unwrap();
    let mut owner = rv.enter(session, &p.build_owner).unwrap();
    let build = rv.enter(session, &p.build_server).unwrap();

    hibana_runtime::race(
        async {
            {
                async {
                    hibana_runtime::race(
                        async {
                            let result =
                                localside::developer::run(&mut developer, &journal, &validation)
                                    .await;
                            {
                                assert_eq!(
                                    result.unwrap_err().to_string(),
                                    "compile arguments differ from the admitted request"
                                );
                            };
                            Ok::<(), crate::shell::ShellError>(())
                        },
                        async {
                            let _ = async {
                                assert_eq!(open(&mut guest, b"service/compile_and_test").await, 0);
                                write(&mut guest, json!({"component":"reviewed_candidate"})).await;
                                guest.recv::<RequestReady>().await.unwrap();
                                guest
                                    .send::<Commit>(&protocol::FdCloseReq(
                                        protocol::FdRequest::new(SERVICE_FD),
                                    ))
                                    .await
                                    .unwrap();
                                assert_eq!(
                                    serde_json::from_slice::<Value>(
                                        guest.recv::<Admitted>().await.unwrap()
                                    )
                                    .unwrap(),
                                    json!({"component":"reviewed_candidate"})
                                );
                                owner.send::<Loan<BUILD_SERVER>>(build).await.unwrap();
                                let mut build = guest.recv::<Loan<BUILD_SERVER>>().await.unwrap();
                                let substituted = Arguments::<1>::from_bytes(
                                    br#"{"component":"unreviewed_candidate"}"#,
                                )
                                .unwrap();
                                build.send::<CompileRequest>(&substituted).await.unwrap();
                                let response = build.recv::<CompileResult>().await.unwrap();
                                panic!(
                                    "substituted argument caused an effect: {:?}; reply: {:?}",
                                    journal.events(),
                                    response
                                );
                            }
                            .await;
                            Err::<(), crate::shell::ShellError>(crate::shell::ShellError::message(
                                "rejected request completed unexpectedly",
                            ))
                        },
                    )
                    .await
                    .unwrap();
                    assert!(journal.events().is_empty());
                }
                .await;
                Ok::<_, std::io::Error>(())
            }
        },
        hibana_runtime::host::deadline(Duration::from_secs(3)),
    )
    .await
    .expect("argument substitution test stalled");

    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn wasi_roles_reach_the_service_owners_only_through_projected_messages()
-> Result<(), crate::shell::ShellError> {
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
    let (test_requested, mut test_ready) = notification().unwrap();
    let (developer_closed, mut closed) = notification().unwrap();
    let clients = async {
        async {
            #[localside]
            let first = async {
                assert_eq!(test_ready.read(&mut [0; 1]).await.unwrap(), 1);
                assert_ne!(open(&mut a, b"service/deploy_application").await, 0);
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 0);
                write(&mut a, json!({"wrong":"application"})).await;
                a.recv::<RequestReady>().await.unwrap();
                a.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(4)))
                    .await
                    .unwrap();
                let rejected =
                    serde_json::from_slice::<Value>(a.recv::<Rejected>().await.unwrap()).unwrap();
                assert_eq!(rejected["status"], "invalid_arguments");
                assert!(journal.events().is_empty());
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 0);
                write(&mut a, json!({"component":"application"})).await;
                a.recv::<RequestReady>().await.unwrap();
                a.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
                    .await
                    .unwrap();
                let arguments = {
                    let bytes = a.recv::<Admitted>().await.unwrap();
                    Arguments::<1>::from_bytes(bytes).unwrap()
                };
                let mut resource = a.recv::<Loan<BUILD_SERVER>>().await.unwrap();
                resource.send::<CompileRequest>(&arguments).await.unwrap();
                let result = resource.recv::<CompileResult>().await.unwrap().to_vec();
                assert_eq!(
                    serde_json::from_slice::<Value>(&result).unwrap()["success"],
                    true
                );
                a.send::<Loan<BUILD_SERVER>>(resource).await.unwrap();
                assert_eq!(open(&mut a, b"service/compile_and_test").await, 8);
                // Closed-service errors must not wait for downstream effects.
                assert_eq!(journal.events().len(), 1);
                developer_closed.notify().unwrap();
                a.recv::<RequestReady>().await.unwrap();
                a.send::<ServiceExit>(&0).await.unwrap();

                Ok(())
            };
            #[localside]
            let second = async {
                {
                    let mut ready = std::pin::pin!(b.recv::<RequestReady>());
                    assert!(
                        hibana_runtime::race(
                            async { Ok::<_, std::io::Error>((&mut ready).await) },
                            hibana_runtime::host::deadline(Duration::from_millis(10))
                        )
                        .await
                        .is_err_and(|error| error.kind() == std::io::ErrorKind::TimedOut)
                    );
                    assert!(journal.events().is_empty());
                    test_requested.notify().unwrap();
                    ready.await.unwrap();
                }
                b.send::<Open>(&protocol::PathOpenReq(
                    protocol::PathOpen::new(3, 1 << 6, b"service/run_integration_tests")
                        .unwrap()
                        .with_flags(8, 0)
                        .unwrap(),
                ))
                .await
                .unwrap();
                assert_eq!(b.recv::<Opened>().await.unwrap().0.errno(), 0);
                assert_eq!(closed.read(&mut [0; 1]).await.unwrap(), 1);
                write(&mut b, json!({"suite":"integration"})).await;
                b.recv::<RequestReady>().await.unwrap();
                b.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
                    .await
                    .unwrap();
                let arguments = {
                    let bytes = b.recv::<Admitted>().await.unwrap();
                    Arguments::<2>::from_bytes(bytes).unwrap()
                };
                let mut resource = b.recv::<Loan<BUILD_SERVER>>().await.unwrap();
                resource.send::<TestRequest>(&arguments).await.unwrap();
                let result = resource.recv::<TestResult>().await.unwrap().to_vec();
                assert_eq!(
                    serde_json::from_slice::<Value>(&result).unwrap()["success"],
                    true
                );
                b.send::<Loan<BUILD_SERVER>>(resource).await.unwrap();
                b.recv::<RequestReady>().await.unwrap();
                b.send::<ServiceExit>(&0).await.unwrap();

                Ok(())
            };
            #[localside]
            let third = async {
                assert_eq!(open(&mut c, b"service/deploy_application").await, 0);
                write(&mut c, json!({"target":"production"})).await;
                c.recv::<RequestReady>().await.unwrap();
                c.send::<Commit>(&protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD)))
                    .await
                    .unwrap();
                let arguments = {
                    let bytes = c.recv::<Admitted>().await.unwrap();
                    Arguments::<3>::from_bytes(bytes).unwrap()
                };
                let mut resource = c.recv::<Loan<DEPLOY_ENV>>().await.unwrap();
                resource.send::<DeployRequest>(&arguments).await.unwrap();
                let result = resource.recv::<DeployResult>().await.unwrap().to_vec();
                assert_eq!(
                    serde_json::from_slice::<Value>(&result).unwrap()["success"],
                    true
                );
                c.send::<Loan<DEPLOY_ENV>>(resource).await.unwrap();
                c.recv::<RequestReady>().await.unwrap();
                c.send::<ServiceExit>(&0).await.unwrap();

                Ok(())
            };
            Ok((first, second, third))
        }
        .await
        .unwrap();
        Ok::<(), crate::shell::ShellError>(())
    };
    hibana_runtime::race(
        async {
            Ok::<_, std::io::Error>(
                async {
                    #[localside]
                    let first = clients;
                    #[localside]
                    let second = localside::developer::run(&mut developer, &journal, &compile);
                    #[localside]
                    let third = localside::tester::run(&mut tester, &journal, &test);
                    #[localside]
                    let fourth = localside::deployer::run(&mut deployer, &journal, &deploy);
                    #[localside]
                    let fifth = localside::infrastructure::build_server(&mut build_owner, build);
                    #[localside]
                    let sixth = localside::infrastructure::deploy_env(&mut deploy_owner, env);
                    Ok((first, second, third, fourth, fifth, sixth))
                }
                .await,
            )
        },
        hibana_runtime::host::deadline(Duration::from_secs(3)),
    )
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

    Ok(())
}
