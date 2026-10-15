//! Mechanism tests use actual projected localsides and ChoreoFS, but construct
//! the WASI request values manually. They do not qualify a real Wasm guest.
use choreofix::tasks::cicd::choreofs::{
    self,
    imp::{Domain, Import},
};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::RefCell, time::Duration};
use tokio::sync::{Notify, mpsc, oneshot};

type Sender = mpsc::UnboundedSender<Import>;
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
async fn open(sender: &Sender, path: &[u8]) -> protocol::PathOpenedRet {
    let request = protocol::PathOpenReq(
        protocol::PathOpen::new(3, 1 << 6, path)
            .unwrap()
            .with_flags(8, 0)
            .unwrap(),
    );
    let (reply, response) = oneshot::channel();
    sender.send(Import::Open(request, reply)).unwrap();
    response.await.unwrap()
}
async fn write(sender: &Sender, fd: u8, bytes: &[u8]) -> protocol::FdWriteDoneRet {
    let request = protocol::FdWriteReq(protocol::FdWrite::new(fd, bytes).unwrap());
    let (reply, response) = oneshot::channel();
    sender.send(Import::Write(request, reply)).unwrap();
    response.await.unwrap()
}
async fn close(sender: &Sender, fd: u8) -> (protocol::FdClosedRet, Vec<u8>) {
    let (reply, response) = oneshot::channel();
    sender
        .send(Import::Close(protocol::FdRequest::new(fd), reply))
        .unwrap();
    response.await.unwrap()
}
async fn submit(sender: &Sender, path: &[u8], arguments: Value) -> Value {
    let opened = open(sender, path).await;
    assert_eq!(opened.0.errno(), 0);
    let mut bytes = serde_json::to_vec(&arguments).unwrap();
    bytes.extend_from_slice(&[b' '; 256]);
    for chunk in bytes.chunks(37) {
        let written = write(sender, opened.0.fd(), chunk).await;
        assert_eq!(written.0.errno(), 0);
        assert_eq!(written.0.written() as usize, chunk.len());
    }
    let (closed, bytes) = close(sender, opened.0.fd()).await;
    assert_eq!(closed.0.errno(), 0);
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn chunked_file_requests_follow_the_full_16e_dependencies_once() {
    let (developer, a) = mpsc::unbounded_channel();
    let (tester, b) = mpsc::unbounded_channel();
    let (deployer, c) = mpsc::unbounded_channel();
    let journal = Journal::default();
    let guest = async {
        let (build, test, deploy) = tokio::join!(
            submit(
                &developer,
                b"service/compile_and_test",
                json!({"component":"application"})
            ),
            submit(
                &tester,
                b"service/run_integration_tests",
                json!({"suite":"integration"})
            ),
            submit(
                &deployer,
                b"service/deploy_application",
                json!({"target":"production"})
            ),
        );
        assert_eq!(build["tool"], "compile_and_test");
        assert_eq!(test["tool"], "run_integration_tests");
        assert_eq!(deploy["tool"], "deploy_application");
        for sender in [&developer, &tester, &deployer] {
            let (reply, _) = oneshot::channel();
            assert!(
                sender
                    .send(Import::Close(protocol::FdRequest::new(4), reply))
                    .is_err(),
                "completed affine service must not accept a second commit"
            );
        }
    };
    tokio::time::timeout(Duration::from_secs(3), async {
        tokio::try_join!(choreofs::run(a, b, c, &journal), async {
            guest.await;
            Ok(())
        })
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        *journal.0.borrow(),
        [
            (
                "DEVELOPER",
                "compile_and_test",
                json!({"component":"application"})
            ),
            (
                "TESTER",
                "run_integration_tests",
                json!({"suite":"integration"})
            ),
            (
                "DEPLOYER",
                "deploy_application",
                json!({"target":"production"})
            ),
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_files_and_arguments_return_errors_then_valid_work_can_continue() {
    let (developer, a) = mpsc::unbounded_channel();
    let (tester, b) = mpsc::unbounded_channel();
    let (deployer, c) = mpsc::unbounded_channel();
    let journal = Journal::default();
    let guest = async {
        // An observed close without an opened service does not get permission.
        let (rejected, _) = close(&developer, 4).await;
        assert_eq!(rejected.0.errno(), 8);
        let bad = open(&developer, b"service/deploy_application").await;
        assert_ne!(bad.0.errno(), 0);
        assert!(journal.0.borrow().is_empty());
        let opened = open(&developer, b"service/compile_and_test").await;
        assert_eq!(opened.0.errno(), 0);
        assert_eq!(
            write(&developer, 4, br#"{"wrong":"application"}"#)
                .await
                .0
                .errno(),
            0
        );
        let (closed, result) = close(&developer, 4).await;
        assert_eq!(closed.0.errno(), 28);
        assert_eq!(
            serde_json::from_slice::<Value>(&result).unwrap()["status"],
            "invalid_arguments"
        );
        assert!(
            journal.0.borrow().is_empty(),
            "invalid arguments must not reach the domain"
        );
        tokio::join!(
            submit(
                &developer,
                b"service/compile_and_test",
                json!({"component":"application"})
            ),
            submit(
                &tester,
                b"service/run_integration_tests",
                json!({"suite":"integration"})
            ),
            submit(
                &deployer,
                b"service/deploy_application",
                json!({"target":"production"})
            ),
        );
    };
    tokio::time::timeout(Duration::from_secs(3), async {
        tokio::try_join!(choreofs::run(a, b, c, &journal), async {
            guest.await;
            Ok(())
        })
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(journal.0.borrow().len(), 3);
}

struct DelayedBuild {
    journal: Journal,
    started: Notify,
    release: Notify,
}
impl Domain for DelayedBuild {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        if tool == "compile_and_test" {
            self.started.notify_one();
            self.release.notified().await;
        }
        self.journal.execute(agent, tool, arguments).await
    }
}
#[tokio::test(flavor = "current_thread")]
async fn early_test_open_waits_for_real_build_completion() {
    let (developer, a) = mpsc::unbounded_channel();
    let (tester, b) = mpsc::unbounded_channel();
    let (deployer, c) = mpsc::unbounded_channel();
    let domain = DelayedBuild {
        journal: Journal::default(),
        started: Notify::new(),
        release: Notify::new(),
    };
    let guests = async {
        let build = submit(
            &developer,
            b"service/compile_and_test",
            json!({"component":"application"}),
        );
        let test = async {
            let request = protocol::PathOpenReq(
                protocol::PathOpen::new(3, 1 << 6, b"service/run_integration_tests")
                    .unwrap()
                    .with_flags(8, 0)
                    .unwrap(),
            );
            let (reply, mut response) = oneshot::channel();
            tester.send(Import::Open(request, reply)).unwrap();
            domain.started.notified().await;
            assert!(
                tokio::time::timeout(Duration::from_millis(10), &mut response)
                    .await
                    .is_err()
            );
            assert!(domain.journal.0.borrow().is_empty());
            domain.release.notify_one();
            assert_eq!(response.await.unwrap().0.errno(), 0);
            write(&tester, 4, br#"{"suite":"integration"}"#).await;
            assert_eq!(close(&tester, 4).await.0.0.errno(), 0);
        };
        let deploy = submit(
            &deployer,
            b"service/deploy_application",
            json!({"target":"production"}),
        );
        tokio::join!(build, test, deploy);
    };
    tokio::time::timeout(Duration::from_secs(3), async {
        tokio::try_join!(choreofs::run(a, b, c, &domain), async {
            guests.await;
            Ok(())
        })
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!(domain.journal.0.borrow().len(), 3);
}
