use super::*;
use crate::shell::imp::file_wire::{ClosedFile, ClosedFileMsg, ResultChunk, ResultChunkMsg};
use crate::tasks::review::{
    choreofs::imp::ReviewTarget,
    imp::{ReviewInput, decide},
};
use hibana_wasip1_runtime::protocol;
use std::time::Duration;

struct RejectedReviews;
impl Domain for RejectedReviews {
    async fn execute(
        &self,
        _: &'static str,
        tool: &'static str,
        _: Value,
    ) -> Result<Value, String> {
        hibana_runtime::host::sleep(Duration::from_millis(1))
            .await
            .expect("test timer");
        Ok(json!({"success": tool != "code_review"}))
    }
}

#[hibana_runtime::main]
#[test]
async fn a_waiting_developer_does_not_repeat_work_during_the_other_review_loop()
-> Result<(), crate::shell::ShellError> {
    let p = programs();
    let session = SessionId::new(6204);
    let (bus, _, _) = Bus::open([], [(session, REPO_ENDPOINT)]);
    let target = ReviewTarget::default();
    let input_a = ReviewInput::default();
    let input_b = ReviewInput::default();
    let mut slab = vec![0; 4 * 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &bus.transport)
        .unwrap();
    rv.set_resolver(
        &p.review_wasi,
        ResolverRef::<SELECT_REVIEW>::decision_state(&target, ReviewTarget::decide),
    )
    .unwrap();
    rv.set_resolver(
        &p.review_a,
        ResolverRef::<DECIDE_A>::decision_state(&input_a, decide),
    )
    .unwrap();
    rv.set_resolver(
        &p.review_b,
        ResolverRef::<DECIDE_B>::decision_state(&input_b, decide),
    )
    .unwrap();
    let mut a = rv.enter(session, &p.a_wasi).unwrap();
    let mut b = rv.enter(session, &p.b_wasi).unwrap();
    let mut reviewer = rv.enter(session, &p.review_wasi).unwrap();
    let mut review_a = rv.enter(session, &p.review_a).unwrap();
    let mut review_b = rv.enter(session, &p.review_b).unwrap();
    let mut a_io = rv.enter(session, &p.a_io).unwrap();
    let mut repo = rv.enter(session, &p.repo).unwrap();
    let repository = rv.enter(session, &p.repo_endpoint).unwrap();
    let [mut a_task, mut a_private] = a.par().unwrap();
    let [mut b_task, _b_private] = b.par().unwrap();
    let [mut reviewer_task, _reviewer_private] = reviewer.par().unwrap();
    let arguments_a =
        RepositoryArguments::<1>::from_bytes(br#"{"feature_name":"user_authentication"}"#).unwrap();
    let arguments_b =
        RepositoryArguments::<1>::from_bytes(br#"{"feature_name":"notification_feature"}"#)
            .unwrap();
    let review_arguments_a =
        RepositoryArguments::<3>::from_bytes(br#"{"pr_id":"user_authentication"}"#).unwrap();
    let review_arguments_b =
        RepositoryArguments::<3>::from_bytes(br#"{"pr_id":"notification_feature"}"#).unwrap();
    let domain = RejectedReviews;
    hibana_runtime::race(
        async {
            {
                async {
                    hibana_runtime::race(
                        async {
                            let () = async {
                                async {
                                    #[localside]
                                    let first = async {
                                        let task = &mut a_task;
                                        for _ in 0..4 {
                                            a_private
                                                .send::<protocol::FdCloseReqMsg>(
                                                    &protocol::FdCloseReq(
                                                        protocol::FdRequest::new(4),
                                                    ),
                                                )
                                                .await
                                                .unwrap();
                                            a_private.recv::<ClosedFileMsg>().await.unwrap();
                                            a_private
                                                .send::<ResultChunkMsg>(
                                                    &ResultChunk::new(0, b"{}").unwrap(),
                                                )
                                                .await
                                                .unwrap();
                                        }
                                        task.offer().await.unwrap().recv::<FirstA>().await.unwrap();
                                        drop(task.offer().await.unwrap());
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<ImplementA>(&arguments_a)
                                            .await
                                            .unwrap();
                                        let mut resource = task.recv::<RepoLoan>().await.unwrap();
                                        resource
                                            .send::<ImplementOnRepo>(&arguments_a)
                                            .await
                                            .unwrap();
                                        resource.recv::<RepoResult>().await.unwrap();
                                        task.send::<RepoLoan>(resource).await.unwrap();
                                        task.send::<ReadyA>(&b"{}".as_slice()).await.unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedA>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<BDuringARevision>()
                                            .await
                                            .unwrap();
                                        a_private
                                            .send::<protocol::FdCloseReqMsg>(&protocol::FdCloseReq(
                                                protocol::FdRequest::new(4),
                                            ))
                                            .await
                                            .unwrap();
                                        a_private.recv::<ClosedFileMsg>().await.unwrap();
                                        a_private
                                            .send::<ResultChunkMsg>(
                                                &ResultChunk::new(0, b"{}").unwrap(),
                                            )
                                            .await
                                            .unwrap();
                                        drop(task.offer().await.unwrap());
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<ReviseA>(&arguments_a)
                                            .await
                                            .unwrap();
                                        let mut resource = task.recv::<RepoLoan>().await.unwrap();
                                        resource
                                            .send::<ImplementOnRepo>(&arguments_a)
                                            .await
                                            .unwrap();
                                        resource.recv::<RepoResult>().await.unwrap();
                                        task.send::<RepoLoan>(resource).await.unwrap();
                                        task.send::<ReadyA>(&b"{}".as_slice()).await.unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<AgainBBeforeAReview>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<AgainBBeforeA>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();
                                        {
                                            match hibana_runtime::race(
                                                async {
                                                    Ok::<_, std::io::Error>(task.offer().await)
                                                },
                                                hibana_runtime::host::deadline(
                                                    Duration::from_millis(20),
                                                ),
                                            )
                                            .await
                                            {
                                                Err(error) => assert_eq!(
                                                    error.kind(),
                                                    std::io::ErrorKind::TimedOut
                                                ),
                                                Ok(Ok(branch)) => panic!(
                                                    "offered {} before a new intent",
                                                    branch.label()
                                                ),
                                                Ok(Err(e)) => panic!("waiting offer failed: {e:?}"),
                                            };
                                            Ok(())
                                        }
                                    };
                                    #[localside]
                                    let second = async {
                                        let task = &mut b_task;
                                        task.offer().await.unwrap().recv::<FirstA>().await.unwrap();
                                        drop(task.offer().await.unwrap());
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<ImplementB>(&arguments_b)
                                            .await
                                            .unwrap();
                                        let mut resource = task.recv::<RepoLoan>().await.unwrap();
                                        resource
                                            .send::<ImplementOnRepo>(&arguments_b)
                                            .await
                                            .unwrap();
                                        resource.recv::<RepoResult>().await.unwrap();
                                        task.send::<RepoLoan>(resource).await.unwrap();
                                        task.send::<ReadyB>(&b"{}".as_slice()).await.unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedA>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<BDuringARevision>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<AgainBBeforeAReview>()
                                            .await
                                            .unwrap();
                                        {
                                            for iteration in 0..2 {
                                                drop(task.offer().await.unwrap());
                                                task.offer()
                                                    .await
                                                    .unwrap()
                                                    .send::<ReviseB>(&arguments_b)
                                                    .await
                                                    .unwrap();
                                                let mut resource =
                                                    task.recv::<RepoLoan>().await.unwrap();
                                                resource
                                                    .send::<ImplementOnRepo>(&arguments_b)
                                                    .await
                                                    .unwrap();
                                                resource.recv::<RepoResult>().await.unwrap();
                                                task.send::<RepoLoan>(resource).await.unwrap();
                                                task.send::<ReadyB>(&b"{}".as_slice())
                                                    .await
                                                    .unwrap();
                                                task.offer()
                                                    .await
                                                    .unwrap()
                                                    .recv::<RejectedB>()
                                                    .await
                                                    .unwrap();
                                                if iteration == 0 {
                                                    task.offer()
                                                        .await
                                                        .unwrap()
                                                        .recv::<AgainBBeforeA>()
                                                        .await
                                                        .unwrap();
                                                }
                                            }
                                            Ok(())
                                        }
                                    };
                                    #[localside]
                                    let third = async {
                                        let task = &mut reviewer_task;
                                        target.record(FEATURE_A);
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<FirstA>(&review_arguments_a)
                                            .await
                                            .unwrap();
                                        for _ in 0..4 {
                                            task.send::<FirstA>(&review_arguments_a).await.unwrap();
                                        }
                                        task.recv::<ReadyA>().await.unwrap();
                                        {
                                            let [mut review, mut other] = task.par().unwrap();
                                            async {
                                                #[localside]
                                                let first = async {
                                                    review
                                                        .send::<ReviewA>(&review_arguments_a)
                                                        .await
                                                        .unwrap();

                                                    Ok(())
                                                };
                                                #[localside]
                                                let second = async {
                                                    other.recv::<ReadyB>().await.unwrap();

                                                    Ok(())
                                                };
                                                Ok((first, second))
                                            }
                                            .await
                                            .unwrap();
                                        }
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedA>()
                                            .await
                                            .unwrap();
                                        target.record(FEATURE_B);
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<BDuringARevision>(&review_arguments_b)
                                            .await
                                            .unwrap();
                                        for _ in 0..4 {
                                            task.send::<BDuringARevision>(&review_arguments_b)
                                                .await
                                                .unwrap();
                                        }
                                        {
                                            let [mut other, mut review] = task.par().unwrap();
                                            async {
                                                #[localside]
                                                let first = async {
                                                    other.recv::<ReadyA>().await.unwrap();

                                                    Ok(())
                                                };
                                                #[localside]
                                                let second = async {
                                                    review
                                                        .send::<ReviewB>(&review_arguments_b)
                                                        .await
                                                        .unwrap();

                                                    Ok(())
                                                };
                                                Ok((first, second))
                                            }
                                            .await
                                            .unwrap();
                                        }
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<AgainBBeforeAReview>(&review_arguments_b)
                                            .await
                                            .unwrap();
                                        for _ in 0..4 {
                                            task.send::<AgainBBeforeAReview>(&review_arguments_b)
                                                .await
                                                .unwrap();
                                        }
                                        task.recv::<ReadyB>().await.unwrap();
                                        task.send::<ReReviewB>(&review_arguments_b).await.unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .send::<AgainBBeforeA>(&review_arguments_b)
                                            .await
                                            .unwrap();
                                        for _ in 0..4 {
                                            task.send::<AgainBBeforeA>(&review_arguments_b)
                                                .await
                                                .unwrap();
                                        }
                                        task.recv::<ReadyB>().await.unwrap();
                                        task.send::<ReReviewB>(&review_arguments_b).await.unwrap();
                                        task.offer()
                                            .await
                                            .unwrap()
                                            .recv::<RejectedB>()
                                            .await
                                            .unwrap();

                                        Ok(())
                                    };
                                    #[localside]
                                    let fourth = async {
                                        {
                                            for _ in 0..5 {
                                                a_io.offer()
                                                    .await
                                                    .unwrap()
                                                    .recv::<protocol::FdCloseReqMsg>()
                                                    .await
                                                    .unwrap();
                                                a_io.send::<ClosedFileMsg>(&ClosedFile {
                                                    completion: protocol::FdClosedRet(
                                                        protocol::FdClosed::new(4),
                                                    ),
                                                    path: vec![],
                                                    bytes: vec![],
                                                })
                                                .await
                                                .unwrap();
                                                a_io.offer()
                                                    .await
                                                    .unwrap()
                                                    .recv::<ResultChunkMsg>()
                                                    .await
                                                    .unwrap();
                                            }
                                            Ok(())
                                        }
                                    };
                                    Ok((first, second, third, fourth))
                                }
                                .await
                                .unwrap();
                            }
                            .await;
                            {};
                            Ok::<(), crate::shell::ShellError>(())
                        },
                        hibana_runtime::race(
                            async {
                                let result =
                                    localside::repo::run(&mut repo, repository, &domain).await;
                                Err::<(), crate::shell::ShellError>(
                                    crate::shell::ShellError::message(format!(
                                        "repo ended: {result:?}"
                                    )),
                                )
                            },
                            hibana_runtime::race(
                                async {
                                    let result = localside::reviewer_a::run(
                                        &mut review_a,
                                        &domain,
                                        &input_a,
                                    )
                                    .await;
                                    Err::<(), crate::shell::ShellError>(
                                        crate::shell::ShellError::message(format!(
                                            "review A ended: {result:?}"
                                        )),
                                    )
                                },
                                async {
                                    let result = localside::reviewer_b::run(
                                        &mut review_b,
                                        &domain,
                                        &input_b,
                                    )
                                    .await;
                                    Err::<(), crate::shell::ShellError>(
                                        crate::shell::ShellError::message(format!(
                                            "review B ended: {result:?}"
                                        )),
                                    )
                                },
                            ),
                        ),
                    )
                    .await
                    .unwrap();
                }
                .await;
                Ok::<_, std::io::Error>(())
            }
        },
        hibana_runtime::host::deadline(Duration::from_secs(10)),
    )
    .await
    .expect("review prefix stalled");

    Ok(())
}
