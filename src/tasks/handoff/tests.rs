use super::{
    global::*,
    imp::{Artifact, Repository, Verification, VerificationInput},
    localside,
};
use crate::{imp::transport::InMemoryTransport, shell::imp::bus::Bus};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use std::{cell::Cell, time::Duration};

// The same schedules run under Miri; this is a validation budget, not a protocol deadline.
const VALIDATION_TIMEOUT: Duration = Duration::from_secs(60);

#[tokio::test(flavor = "current_thread")]
async fn a_failed_candidate_moves_to_b_and_both_resources_guard_the_actual_commit() {
    let programs = programs();
    let session = SessionId::new(301);
    let (bus, _, _) = Bus::open([], [(session, R), (session, S)]);
    let result = VerificationInput::default();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &bus.transport)
        .unwrap();
    let mut recovery_slab = vec![0; 1024 * 1024];
    let mut recovery_storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let recovery = recovery_storage
        .init()
        .rendezvous(&mut recovery_slab, &bus.transport)
        .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<VERIFY>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<COMMIT_DECISION>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    let mut a = rv.enter(session, &programs.a).unwrap();
    let r = rv.enter(session, &programs.r).unwrap();
    let mut b = recovery.enter(session, &programs.b).unwrap();
    let mut c = rv.enter(session, &programs.c).unwrap();
    let mut store = rv.enter(session, &programs.store).unwrap();
    let mut aux = rv.enter(session, &programs.aux).unwrap();
    let mut owner = rv.enter(session, &programs.s_owner).unwrap();
    let s = rv.enter(session, &programs.s).unwrap();
    let initial = Artifact {
        target: 42,
        revision: 0,
        value: 0,
    };
    let observations = Cell::new(Vec::new());
    let mut repository = Repository {
        current: initial,
        published: Vec::new(),
        observations: &observations,
    };
    let execution = tokio::time::timeout(VALIDATION_TIMEOUT, async {
        tokio::try_join!(
            localside::initial(
                &mut a,
                r,
                Artifact {
                    value: 1,
                    ..initial
                }
            ),
            localside::recover(&mut b, [3, 6]),
            localside::independent(&mut c, 10),
            localside::service(&mut store, &mut repository, &result),
            localside::auxiliary(&mut aux),
            localside::second_owner(&mut owner, s),
        )
    })
    .await;
    if execution.is_err() {
        for frame in bus
            .transport
            .take_observations()
            .iter()
            .rev()
            .take(24)
            .rev()
        {
            eprintln!("{frame}");
        }
    }
    let (actual, recovered, independent, (), (), ()) = execution.expect("handoff stalled").unwrap();
    assert_eq!(
        actual,
        Artifact {
            target: 42,
            revision: 3,
            value: 6
        }
    );
    assert_eq!(actual, recovered);
    assert_eq!(independent, 100);
    assert_eq!(repository.published, [(actual, 100)]);
    assert_eq!(observations.take(), [actual]);
}

#[tokio::test(flavor = "current_thread")]
async fn old_approval_wrong_target_and_changed_arguments_never_publish() {
    let initial = Artifact {
        target: 42,
        revision: 0,
        value: 0,
    };
    let old = Verification {
        artifact: initial,
        passed: true,
    };
    let current = Verification {
        artifact: Artifact {
            target: 42,
            revision: 2,
            value: 2,
        },
        passed: true,
    };
    let wrong_target = Verification {
        artifact: Artifact {
            target: 99,
            ..current.artifact
        },
        ..current
    };
    let wrong_value = Verification {
        artifact: Artifact {
            value: 4,
            ..current.artifact
        },
        ..current
    };
    let failed = Verification {
        passed: false,
        ..current
    };
    for (name, first, second) in [
        ("old-approval", old, old),
        ("wrong-target", wrong_target, wrong_target),
        ("changed-arguments", wrong_value, wrong_value),
        ("failed-verification", failed, failed),
        ("wrong-first-resource", old, current),
        ("wrong-second-resource", current, old),
    ] {
        let programs = programs();
        let session = SessionId::new(302);
        let (bus, _, _) = Bus::open([], [(session, R), (session, S)]);
        let result = VerificationInput::default();
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
        let rv = storage
            .init()
            .rendezvous(&mut slab, &bus.transport)
            .unwrap();
        let mut recovery_slab = vec![0; 1024 * 1024];
        let mut recovery_storage = SessionKitStorage::<&InMemoryTransport>::uninit();
        let recovery = recovery_storage
            .init()
            .rendezvous(&mut recovery_slab, &bus.transport)
            .unwrap();
        rv.set_resolver(
            &programs.store,
            ResolverRef::<VERIFY>::decision_state(&result, VerificationInput::decide),
        )
        .unwrap();
        rv.set_resolver(
            &programs.store,
            ResolverRef::<COMMIT_DECISION>::decision_state(&result, VerificationInput::decide),
        )
        .unwrap();
        let mut a = rv.enter(session, &programs.a).unwrap();
        let r = rv.enter(session, &programs.r).unwrap();
        let mut b = recovery.enter(session, &programs.b).unwrap();
        let mut c = rv.enter(session, &programs.c).unwrap();
        let mut store = rv.enter(session, &programs.store).unwrap();
        let mut aux = rv.enter(session, &programs.aux).unwrap();
        let mut owner = rv.enter(session, &programs.s_owner).unwrap();
        let s = rv.enter(session, &programs.s).unwrap();
        let observations = Cell::new(Vec::new());
        let mut repository = Repository {
            current: initial,
            published: Vec::new(),
            observations: &observations,
        };
        assert_eq!(repository.verify(initial).unwrap(), old);
        tokio::time::timeout(VALIDATION_TIMEOUT, async {
            tokio::select! {
                result = localside::service(&mut store, &mut repository, &result) => {
                    assert_eq!(result.unwrap_err().to_string(), "commit differs from the actual verified artifact", "{name}");
                }
                result = async {
                    tokio::try_join!(
                        localside::initial(&mut a, r, Artifact { value: 1, ..initial }),
                        async {
                            let context = Verification::decode(b.recv::<Context>().await?)?;
                            let mut r = b.recv::<LoanR>().await?;
                            r.send::<Revise>(&Artifact { value: 2, ..context.artifact }.bytes()).await?;
                            let artifact = r.recv::<Revised>().await?;
                            r.send::<Reinspect>(&artifact).await?;
                            let checked = r.offer().await?.recv::<Passed>().await?;
                            r.send::<CandidateReady>(&checked).await?;
                            r.offer().await?.recv::<CommitReady>().await?;
                            let mut s = b.recv::<LoanS>().await?;
                            let first = first.bytes();
                            let second = second.bytes();
                            tokio::try_join!(r.send::<CommitR>(&first), s.send::<CommitS>(&second))?;
                            let published = r.recv::<CommittedR>().await?;
                            Err::<(), _>(crate::shell::ShellError::message(format!("forged commit materialized: {published:?}")))
                        },
                        localside::independent(&mut c, 10),
                        localside::auxiliary(&mut aux),
                        localside::second_owner(&mut owner, s),
                    )
                } => panic!("{name}: unexpected client completion: {result:?}"),
            }
        }).await.expect("forged commit test stalled");
        assert!(repository.published.is_empty(), "{name}");
        assert!(observations.take().is_empty(), "{name}");
        assert!(rv.enter(session, &programs.r).is_err());
        assert!(rv.enter(session, &programs.s).is_err());
        eprintln!("{name}: rejected before publication");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn a_missing_second_request_keeps_the_effect_pending_then_can_complete() {
    let programs = programs();
    let session = SessionId::new(303);
    let (bus, _, _) = Bus::open([], [(session, R), (session, S)]);
    let result = VerificationInput::default();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &bus.transport)
        .unwrap();
    let mut recovery_slab = vec![0; 1024 * 1024];
    let mut recovery_storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let recovery = recovery_storage
        .init()
        .rendezvous(&mut recovery_slab, &bus.transport)
        .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<VERIFY>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<COMMIT_DECISION>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    let mut a = rv.enter(session, &programs.a).unwrap();
    let r = rv.enter(session, &programs.r).unwrap();
    let mut b = recovery.enter(session, &programs.b).unwrap();
    let mut c = rv.enter(session, &programs.c).unwrap();
    let mut store = rv.enter(session, &programs.store).unwrap();
    let mut aux = rv.enter(session, &programs.aux).unwrap();
    let mut owner = rv.enter(session, &programs.s_owner).unwrap();
    let s = rv.enter(session, &programs.s).unwrap();
    let initial = Artifact {
        target: 42,
        revision: 0,
        value: 0,
    };
    let observations = Cell::new(Vec::new());
    let mut repository = Repository {
        current: initial,
        published: Vec::new(),
        observations: &observations,
    };
    let (actual, (), _, (), (), ()) = tokio::time::timeout(VALIDATION_TIMEOUT, async {
        tokio::try_join!(
            localside::initial(
                &mut a,
                r,
                Artifact {
                    value: 1,
                    ..initial
                }
            ),
            async {
                let context = Verification::decode(b.recv::<Context>().await?)?;
                let mut r = b.recv::<LoanR>().await?;
                r.send::<Revise>(
                    &Artifact {
                        value: 2,
                        ..context.artifact
                    }
                    .bytes(),
                )
                .await?;
                let artifact = r.recv::<Revised>().await?;
                r.send::<Reinspect>(&artifact).await?;
                let checked = r.offer().await?.recv::<Passed>().await?;
                r.send::<CandidateReady>(&checked).await?;
                r.offer().await?.recv::<CommitReady>().await?;
                let mut s = b.recv::<LoanS>().await?;
                r.send::<CommitR>(&checked).await?;
                let mut response = Box::pin(r.recv::<CommittedR>());
                assert!(
                    tokio::time::timeout(Duration::from_millis(10), &mut response)
                        .await
                        .is_err()
                );
                assert!(
                    observations.take().is_empty(),
                    "effect occurred without the S request"
                );
                s.send::<CommitS>(&checked).await?;
                let (first, second) = tokio::try_join!(response, s.recv::<CommittedS>())?;
                assert_eq!(first, second);
                b.send::<LoanS>(s).await?;
                b.send::<LoanR>(r).await?;
                b.send::<Report>(&first).await?;
                Ok(())
            },
            localside::independent(&mut c, 10),
            localside::service(&mut store, &mut repository, &result),
            localside::auxiliary(&mut aux),
            localside::second_owner(&mut owner, s),
        )
    })
    .await
    .expect("partial-resource test stalled")
    .unwrap();
    assert_eq!(
        actual,
        Artifact {
            target: 42,
            revision: 2,
            value: 2
        }
    );
    assert_eq!(repository.published, [(actual, 100)]);
    assert_eq!(observations.take(), [actual]);
}

#[tokio::test(flavor = "current_thread")]
async fn claiming_approval_without_repair_does_not_replace_the_actual_failure() {
    let programs = programs();
    let session = SessionId::new(304);
    let (bus, _, _) = Bus::open([], [(session, R), (session, S)]);
    let result = VerificationInput::default();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &bus.transport)
        .unwrap();
    let mut recovery_slab = vec![0; 1024 * 1024];
    let mut recovery_storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let recovery = recovery_storage
        .init()
        .rendezvous(&mut recovery_slab, &bus.transport)
        .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<VERIFY>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<COMMIT_DECISION>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    let mut a = rv.enter(session, &programs.a).unwrap();
    let r = rv.enter(session, &programs.r).unwrap();
    let mut b = recovery.enter(session, &programs.b).unwrap();
    let mut c = rv.enter(session, &programs.c).unwrap();
    let mut store = rv.enter(session, &programs.store).unwrap();
    let mut aux = rv.enter(session, &programs.aux).unwrap();
    let mut owner = rv.enter(session, &programs.s_owner).unwrap();
    let s = rv.enter(session, &programs.s).unwrap();
    let initial = Artifact {
        target: 42,
        revision: 0,
        value: 0,
    };
    let observations = Cell::new(Vec::new());
    let mut repository = Repository {
        current: initial,
        published: Vec::new(),
        observations: &observations,
    };
    tokio::time::timeout(VALIDATION_TIMEOUT, async {
        tokio::select! {
            result = localside::service(&mut store, &mut repository, &result) => {
                assert_eq!(result.unwrap_err().to_string(), "final verification differs from the actual result");
            }
            result = async {
                tokio::try_join!(
                    localside::initial(&mut a, r, Artifact { value: 1, ..initial }),
                    async {
                        let context = Verification::decode(b.recv::<Context>().await?)?;
                        let mut r = b.recv::<LoanR>().await?;
                        let forged = Verification { passed: true, ..context };
                        r.send::<CandidateReady>(&forged.bytes()).await?;
                        let approval = r.recv::<CommitReady>().await?;
                        Err::<(), _>(crate::shell::ShellError::message(format!("unverified approval: {approval:?}")))
                    },
                    localside::independent(&mut c, 10),
                    localside::auxiliary(&mut aux),
                    localside::second_owner(&mut owner, s),
                )
            } => panic!("verification skipped: unexpected client completion: {result:?}"),
        }
    }).await.expect("skipped verification test stalled");
    assert!(repository.published.is_empty());
    assert!(observations.take().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn a_real_negative_result_can_finish_without_publication_or_forced_repair() {
    let programs = programs();
    let session = SessionId::new(305);
    let (bus, _, _) = Bus::open([], [(session, R), (session, S)]);
    let result = VerificationInput::default();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &bus.transport)
        .unwrap();
    let mut recovery_slab = vec![0; 1024 * 1024];
    let mut recovery_storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let recovery = recovery_storage
        .init()
        .rendezvous(&mut recovery_slab, &bus.transport)
        .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<VERIFY>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<COMMIT_DECISION>::decision_state(&result, VerificationInput::decide),
    )
    .unwrap();
    let mut a = rv.enter(session, &programs.a).unwrap();
    let r = rv.enter(session, &programs.r).unwrap();
    let mut b = recovery.enter(session, &programs.b).unwrap();
    let mut c = rv.enter(session, &programs.c).unwrap();
    let mut store = rv.enter(session, &programs.store).unwrap();
    let mut aux = rv.enter(session, &programs.aux).unwrap();
    let mut owner = rv.enter(session, &programs.s_owner).unwrap();
    let s = rv.enter(session, &programs.s).unwrap();
    let initial = Artifact {
        target: 42,
        revision: 0,
        value: 0,
    };
    let observations = Cell::new(Vec::new());
    let mut repository = Repository {
        current: initial,
        published: Vec::new(),
        observations: &observations,
    };
    let (a_result, b_result, independent, service, auxiliary, owner) =
        tokio::time::timeout(VALIDATION_TIMEOUT, async {
            tokio::join!(
                localside::initial(
                    &mut a,
                    r,
                    Artifact {
                        value: 1,
                        ..initial
                    }
                ),
                async {
                    let context = Verification::decode(b.recv::<Context>().await?)?;
                    assert!(!context.passed);
                    let mut r = b.recv::<LoanR>().await?;
                    r.send::<CandidateReady>(&context.bytes()).await?;
                    let actual =
                        Verification::decode(r.offer().await?.recv::<CommitDenied>().await?)?;
                    assert_eq!(actual, context);
                    b.send::<LoanR>(r).await?;
                    Ok::<(), crate::shell::ShellError>(())
                },
                localside::independent(&mut c, 10),
                localside::service(&mut store, &mut repository, &result),
                localside::auxiliary(&mut aux),
                localside::second_owner(&mut owner, s),
            )
        })
        .await
        .expect("explicit failure completion stalled");
    assert_eq!(
        a_result.unwrap_err().to_string(),
        "candidate verification failed"
    );
    b_result.unwrap();
    assert_eq!(independent.unwrap(), 100);
    service.unwrap();
    auxiliary.unwrap();
    owner.unwrap();
    assert!(repository.published.is_empty());
    assert!(observations.take().is_empty());
}
