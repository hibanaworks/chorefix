use super::{
    global::*,
    imp::{Artifact, Repository, Verification, VerificationInput},
    localside,
};
use crate::{imp::transport::InMemoryTransport, shell::imp::bus::Bus};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use std::{cell::Cell, time::Duration};
#[cfg(miri)]
mod clock;
#[cfg(miri)]
use clock::deadline;
#[cfg(not(miri))]
use hibana_runtime::host::deadline;

// The same schedules run under Miri; this is a validation budget, not a protocol deadline.
const VALIDATION_TIMEOUT: Duration = Duration::from_secs(60);

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn a_failed_candidate_moves_to_b_and_both_resources_guard_the_actual_commit()
-> Result<(), crate::shell::ShellError> {
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
    let execution = hibana_runtime::race(
        async {
            Ok::<_, std::io::Error>(
                async {
                    #[localside]
                    let first = localside::initial(
                        &mut a,
                        r,
                        Artifact {
                            value: 1,
                            ..initial
                        },
                    );
                    #[localside]
                    let second = localside::recover(&mut b, [3, 6]);
                    #[localside]
                    let third = localside::independent(&mut c, 10);
                    #[localside]
                    let fourth = localside::service(&mut store, &mut repository, &result);
                    #[localside]
                    let fifth = localside::auxiliary(&mut aux);
                    #[localside]
                    let sixth = localside::second_owner(&mut owner, s);
                    Ok((first, second, third, fourth, fifth, sixth))
                }
                .await,
            )
        },
        deadline(VALIDATION_TIMEOUT),
    )
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

    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn old_approval_wrong_target_and_changed_arguments_never_publish()
-> Result<(), crate::shell::ShellError> {
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
        hibana_runtime::race(
            async {
                {
                    async {
                        hibana_runtime::race(
                            async {
                                let result =
                                    localside::service(&mut store, &mut repository, &result).await;
                                {
                                    assert_eq!(
                                        result.unwrap_err().to_string(),
                                        "commit differs from the actual verified artifact",
                                        "{name}"
                                    );
                                };
                                Ok::<(), crate::shell::ShellError>(())
                            },
                            async {
                                let result = async {
                                    #[localside]
                                    let first = async {
                                        let mut r = r;
                                        r.send::<Prepare>(
                                            &Artifact {
                                                value: 1,
                                                ..initial
                                            }
                                            .bytes(),
                                        )
                                        .await?;
                                        let prepared = r.recv::<Prepared>().await?;
                                        r.send::<Inspect>(&prepared).await?;
                                        let checked = r.recv::<Inspected>().await?;
                                        a.send::<Context>(&checked).await?;
                                        a.send::<LoanR>(r).await?;
                                        let checked = a.recv::<CandidateReady>().await?;
                                        let mut r = a.recv::<LoanR>().await?;
                                        a.send::<CandidateReady>(&checked).await?;
                                        r.offer().await?.recv::<CommitReady>().await?;
                                        r.send::<CommitR>(&first.bytes()).await?;
                                        let published = r.recv::<CommittedR>().await?;
                                        Err::<Artifact, _>(crate::shell::ShellError::message(
                                            format!("forged commit materialized: {published:?}"),
                                        ))
                                    };
                                    #[localside]
                                    let second = async {
                                        let context =
                                            Verification::decode(b.recv::<Context>().await?)?;
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
                                        b.send::<LoanR>(r).await?;
                                        b.offer().await?.recv::<CommitReady>().await?;
                                        let mut s = b.recv::<LoanS>().await?;
                                        s.send::<CommitS>(&second.bytes()).await?;
                                        let published = s.recv::<CommittedS>().await?;
                                        Err::<(), _>(crate::shell::ShellError::message(format!(
                                            "forged commit materialized: {published:?}"
                                        )))
                                    };
                                    #[localside]
                                    let third = localside::independent(&mut c, 10);
                                    #[localside]
                                    let fourth = localside::auxiliary(&mut aux);
                                    #[localside]
                                    let fifth = localside::second_owner(&mut owner, s);
                                    Ok((first, second, third, fourth, fifth))
                                }
                                .await;
                                Err::<(), crate::shell::ShellError>(
                                    crate::shell::ShellError::message(format!(
                                        "{name}: unexpected client completion: {result:?}"
                                    )),
                                )
                            },
                        )
                        .await
                        .unwrap();
                    }
                    .await;
                    Ok::<_, std::io::Error>(())
                }
            },
            deadline(VALIDATION_TIMEOUT),
        )
        .await
        .expect("forged commit test stalled");
        assert!(repository.published.is_empty(), "{name}");
        assert!(observations.take().is_empty(), "{name}");
        assert!(rv.enter(session, &programs.r).is_err());
        assert!(rv.enter(session, &programs.s).is_err());
        eprintln!("{name}: rejected before publication");
    }
    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn a_missing_second_request_keeps_the_effect_pending_then_can_complete()
-> Result<(), crate::shell::ShellError> {
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
    let (actual, _, (), (), ()) = hibana_runtime::race(
        async {
            Ok::<_, std::io::Error>(
                async {
                    #[localside]
                    let actual = async {
                        let mut r = r;
                        r.send::<Prepare>(
                            &Artifact {
                                value: 1,
                                ..initial
                            }
                            .bytes(),
                        )
                        .await?;
                        let prepared = r.recv::<Prepared>().await?;
                        r.send::<Inspect>(&prepared).await?;
                        let checked = r.recv::<Inspected>().await?;
                        a.send::<Context>(&checked).await?;
                        a.send::<LoanR>(r).await?;
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
                        let declared = a.recv::<CandidateReady>().await?;
                        b.send::<LoanR>(r).await?;
                        let mut r = a.recv::<LoanR>().await?;
                        a.send::<CandidateReady>(&declared).await?;
                        r.offer().await?.recv::<CommitReady>().await?;
                        b.offer().await?.recv::<CommitReady>().await?;
                        let mut s = b.recv::<LoanS>().await?;
                        r.send::<CommitR>(&checked).await?;
                        let mut response = Box::pin(r.recv::<CommittedR>());
                        assert!(
                            hibana_runtime::race(
                                async { Ok::<_, std::io::Error>((&mut response).await) },
                                deadline(Duration::from_millis(10))
                            )
                            .await
                            .is_err_and(|error| error.kind() == std::io::ErrorKind::TimedOut)
                        );
                        assert!(
                            observations.take().is_empty(),
                            "effect occurred without the S request"
                        );
                        s.send::<CommitS>(&checked).await?;
                        let (first, second) = async {
                            #[localside]
                            let first = response;
                            #[localside]
                            let second = s.recv::<CommittedS>();
                            Ok((first, second))
                        }
                        .await?;
                        assert_eq!(first, second);
                        b.send::<LoanS>(s).await?;
                        b.send::<Report>(&second).await?;
                        let reported = a.recv::<Report>().await?;
                        assert_eq!(reported, first);
                        let actual = Artifact::decode(first);
                        r.send::<Audit>(&actual.target).await?;
                        assert_eq!(r.recv::<Audited>().await?, first);
                        Ok(actual)
                    };
                    #[localside]
                    let independent = localside::independent(&mut c, 10);
                    #[localside]
                    let service = localside::service(&mut store, &mut repository, &result);
                    #[localside]
                    let auxiliary = localside::auxiliary(&mut aux);
                    #[localside]
                    let owner = localside::second_owner(&mut owner, s);
                    Ok((actual, independent, service, auxiliary, owner))
                }
                .await,
            )
        },
        deadline(VALIDATION_TIMEOUT),
    )
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

    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn claiming_approval_without_repair_does_not_replace_the_actual_failure()
-> Result<(), crate::shell::ShellError> {
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
    hibana_runtime::race(
        async {
            {
                async {
                    hibana_runtime::race(
                        async {
                            let result =
                                localside::service(&mut store, &mut repository, &result).await;
                            {
                                assert_eq!(
                                    result.unwrap_err().to_string(),
                                    "final verification differs from the actual result"
                                );
                            };
                            Ok::<(), crate::shell::ShellError>(())
                        },
                        async {
                            let result = async {
                                #[localside]
                                let first = localside::initial(
                                    &mut a,
                                    r,
                                    Artifact {
                                        value: 1,
                                        ..initial
                                    },
                                );
                                #[localside]
                                let second = async {
                                    let context = Verification::decode(b.recv::<Context>().await?)?;
                                    let mut r = b.recv::<LoanR>().await?;
                                    let forged = Verification {
                                        passed: true,
                                        ..context
                                    };
                                    r.send::<CandidateReady>(&forged.bytes()).await?;
                                    b.send::<LoanR>(r).await?;
                                    let approval = b.recv::<CommitReady>().await?;
                                    Err::<(), _>(crate::shell::ShellError::message(format!(
                                        "unverified approval: {approval:?}"
                                    )))
                                };
                                #[localside]
                                let third = localside::independent(&mut c, 10);
                                #[localside]
                                let fourth = localside::auxiliary(&mut aux);
                                #[localside]
                                let fifth = localside::second_owner(&mut owner, s);
                                Ok((first, second, third, fourth, fifth))
                            }
                            .await;
                            Err::<(), crate::shell::ShellError>(crate::shell::ShellError::message(
                                format!(
                                    "verification skipped: unexpected client completion: {result:?}"
                                ),
                            ))
                        },
                    )
                    .await
                    .unwrap();
                }
                .await;
                Ok::<_, std::io::Error>(())
            }
        },
        deadline(VALIDATION_TIMEOUT),
    )
    .await
    .expect("skipped verification test stalled");
    assert!(repository.published.is_empty());
    assert!(observations.take().is_empty());

    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn a_real_negative_result_can_finish_without_publication_or_forced_repair()
-> Result<(), crate::shell::ShellError> {
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
    let (a_result, b_result, independent, service, auxiliary, owner) = hibana_runtime::race(
        async {
            Ok::<_, std::io::Error>(
                async {
                    async {
                        #[localside]
                        let first = async {
                            Ok(localside::initial(
                                &mut a,
                                r,
                                Artifact {
                                    value: 1,
                                    ..initial
                                },
                            )
                            .await)
                        };
                        #[localside]
                        let second = async {
                            Ok(async {
                                let context = Verification::decode(b.recv::<Context>().await?)?;
                                assert!(!context.passed);
                                let mut r = b.recv::<LoanR>().await?;
                                r.send::<CandidateReady>(&context.bytes()).await?;
                                b.send::<LoanR>(r).await?;
                                let actual = Verification::decode(
                                    b.offer().await?.recv::<CommitDenied>().await?,
                                )?;
                                assert_eq!(actual, context);
                                Ok::<(), crate::shell::ShellError>(())
                            }
                            .await)
                        };
                        #[localside]
                        let third = async { Ok(localside::independent(&mut c, 10).await) };
                        #[localside]
                        let fourth = async {
                            Ok(localside::service(&mut store, &mut repository, &result).await)
                        };
                        #[localside]
                        let fifth = async { Ok(localside::auxiliary(&mut aux).await) };
                        #[localside]
                        let sixth = async { Ok(localside::second_owner(&mut owner, s).await) };
                        Ok((first, second, third, fourth, fifth, sixth))
                    }
                    .await
                    .unwrap()
                }
                .await,
            )
        },
        deadline(VALIDATION_TIMEOUT),
    )
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

    Ok(())
}

#[cfg_attr(not(miri), hibana_runtime::main)]
#[cfg_attr(miri, hibana_runtime::main(clock::Clock))]
#[test]
async fn candidate_ready_seals_rework_before_the_publication_requests() {
    let programs = programs();
    let session = SessionId::new(303);
    let (bus, _, _) = Bus::open([], [(session, R), (session, S)]);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport).unwrap();
    let mut peer_slab = vec![0; 1024 * 1024];
    let mut peer_storage = SessionKitStorage::uninit();
    let peer_kit = peer_storage.init();
    let peer = peer_kit.rendezvous(&mut peer_slab, &bus.transport).unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<VERIFY>::decision_state(&(), |_| {
            Ok(hibana::runtime::resolver::DecisionArm::Left)
        }),
    )
    .unwrap();
    rv.set_resolver(
        &programs.store,
        ResolverRef::<COMMIT_DECISION>::decision_state(&(), |_| {
            Ok(hibana::runtime::resolver::DecisionArm::Left)
        }),
    )
    .unwrap();
    let mut a = rv.enter(session, &programs.a).unwrap();
    let mut r = rv.enter(session, &programs.r).unwrap();
    let mut b = peer.enter(session, &programs.b).unwrap();
    let mut c = rv.enter(session, &programs.c).unwrap();
    let mut store = rv.enter(session, &programs.store).unwrap();
    let mut aux = rv.enter(session, &programs.aux).unwrap();
    let mut owner = rv.enter(session, &programs.s_owner).unwrap();
    let s = rv.enter(session, &programs.s).unwrap();
    let artifact = [0; 24];
    let mut checked = [0; 25];
    checked[24] = 1;

    r.send::<Prepare>(&artifact).await.unwrap();
    store.recv::<Prepare>().await.unwrap();
    store.send::<Prepared>(&artifact).await.unwrap();
    r.recv::<Prepared>().await.unwrap();
    r.send::<Inspect>(&artifact).await.unwrap();
    store.recv::<Inspect>().await.unwrap();
    store.send::<Inspected>(&checked).await.unwrap();
    r.recv::<Inspected>().await.unwrap();
    a.send::<Context>(&checked).await.unwrap();
    b.recv::<Context>().await.unwrap();
    a.send::<LoanR>(r).await.unwrap();
    let mut r = b.recv::<LoanR>().await.unwrap();

    owner.send::<LoanS>(s).await.unwrap();
    let mut s = c.recv::<LoanS>().await.unwrap();
    s.send::<Work>(&3).await.unwrap();
    aux.recv::<Work>().await.unwrap();
    aux.send::<Worked>(&9).await.unwrap();
    s.recv::<Worked>().await.unwrap();
    c.send::<LoanS>(s).await.unwrap();
    let s = owner.recv::<LoanS>().await.unwrap();
    c.send::<WorkReport>(&9).await.unwrap();
    store.recv::<WorkReport>().await.unwrap();

    r.send::<CandidateReady>(&checked).await.unwrap();
    let declared = a.recv::<CandidateReady>().await.unwrap();
    b.send::<LoanR>(r).await.unwrap();
    let mut r = a.recv::<LoanR>().await.unwrap();
    a.send::<CandidateReady>(&declared).await.unwrap();
    let branch = store.offer().await.unwrap();
    assert_eq!(branch.label(), 25);
    branch.recv::<CandidateReady>().await.unwrap();
    store.send::<CommitReady>(&checked).await.unwrap();
    r.recv::<CommitReady>().await.unwrap();
    store.send::<CommitReady>(&checked).await.unwrap();
    owner.recv::<CommitReady>().await.unwrap();
    store.send::<CommitReady>(&checked).await.unwrap();
    b.recv::<CommitReady>().await.unwrap();
    owner.send::<LoanS>(s).await.unwrap();
    let _second = b.recv::<LoanS>().await.unwrap();

    let prefix = bus.transport.take_observations();
    let result = hibana_runtime::race(
        async { Ok::<_, std::io::Error>(r.send::<Revise>(&artifact).await) },
        deadline(Duration::from_secs(2)),
    )
    .await
    .expect("the probe remained pending rather than accepting or rejecting rework");
    assert!(
        result.is_err(),
        "CandidateReady did not seal rework: {result:?}; carrier tail: {}",
        serde_json::json!(
            prefix
                .into_iter()
                .rev()
                .take(6)
                .collect::<Vec<_>>()
        )
    );
    assert!(
        bus.transport.take_observations().is_empty(),
        "rejected rework reached the transport"
    );
}
