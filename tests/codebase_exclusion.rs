//! Attempts another legal communication order against the actual 1H global.
//! No production localside, resource mutex, simulator lock or owner table runs.
use choreofix::tasks::codebase::global::{reconsider::*, *};
use hibana::runtime::{
    SessionKitStorage,
    ids::SessionId,
    resolver::{DecisionArm, ResolverError, ResolverRef},
};
use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    task::{Context, Poll, Waker},
};

#[path = "../src/imp/transport.rs"]
mod transport;

fn request<const ROLE: u8>(calls: &RefCell<VecDeque<u8>>) -> Result<DecisionArm, ResolverError> {
    let calls = calls.borrow();
    Ok(if calls.contains(&ROLE) {
        DecisionArm::Left
    } else {
        DecisionArm::Right
    })
}

fn failed_test(_: &()) -> Result<DecisionArm, ResolverError> {
    Ok(DecisionArm::Right)
}

#[tokio::test(flavor = "current_thread")]
async fn grants_and_queued_competing_visits_are_guarded_by_the_actual_1h_global() {
    for attempt in ["early_commit", "direct_recv", "offer", "next_round"] {
        let p = project_1h();
        let calls = RefCell::new(VecDeque::from([A, B]));
        let mut slab = vec![0; 512 * 1024];
        let wire = transport::InMemoryTransport::new([]);
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &wire).unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_A>::decision_state(&calls, request::<0>),
        )
        .unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_B>::decision_state(&calls, request::<1>),
        )
        .unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_C>::decision_state(&calls, request::<2>),
        )
        .unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_D>::decision_state(&calls, request::<3>),
        )
        .unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_E>::decision_state(&calls, request::<4>),
        )
        .unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_F>::decision_state(&calls, request::<5>),
        )
        .unwrap();
        rv.set_resolver(
            &p.dispatch,
            ResolverRef::<READY_G>::decision_state(&calls, request::<6>),
        )
        .unwrap();
        rv.set_resolver(
            &p.a,
            ResolverRef::<TEST_A>::decision_state(&(), failed_test),
        )
        .unwrap();
        let sid = SessionId::new(83);
        let mut dispatch = rv.enter(sid, &p.dispatch).unwrap();
        let mut a = rv.enter(sid, &p.a).unwrap();
        let mut b = rv.enter(sid, &p.b).unwrap();
        let mut auth = rv.enter(sid, &p.auth).unwrap();
        let mut db = rv.enter(sid, &p.database).unwrap();
        a.send::<Design>(&()).await.unwrap();
        a.send::<Implement>(&()).await.unwrap();
        b.send::<Design>(&()).await.unwrap();
        b.send::<Implement>(&()).await.unwrap();
        for _ in 0..3 {
            dispatch.send::<ChooseA>(&()).await.unwrap();
        }
        a.recv::<ChooseA>().await.unwrap();
        assert_eq!(calls.borrow_mut().pop_front(), Some(A));
        a.send::<SubmittedA>(&()).await.unwrap();
        dispatch.recv::<SubmittedA>().await.unwrap();
        a.send::<<ChooseA as Loan>::Local>(&()).await.unwrap();
        auth.recv::<ChooseA>().await.unwrap();
        db.recv::<ChooseA>().await.unwrap();
        auth.send::<<ChooseA as Loan>::Local>(&()).await.unwrap();
        db.send::<<ChooseA as Loan>::Local>(&()).await.unwrap();
        auth.send::<<ChooseA as Loan>::AvailableFirst>(&())
            .await
            .unwrap();
        db.send::<<ChooseA as Loan>::AvailableSecond>(&())
            .await
            .unwrap();
        a.recv::<<ChooseA as Loan>::AvailableFirst>().await.unwrap();
        a.recv::<<ChooseA as Loan>::AvailableSecond>()
            .await
            .unwrap();
        a.send::<<ChooseA as Loan>::AcquireFirst>(&())
            .await
            .unwrap();
        a.send::<<ChooseA as Loan>::AcquireSecond>(&())
            .await
            .unwrap();
        auth.recv::<<ChooseA as Loan>::AcquireFirst>()
            .await
            .unwrap();
        db.recv::<<ChooseA as Loan>::AcquireSecond>().await.unwrap();
        auth.send::<<ChooseA as Loan>::GrantedFirst>(&())
            .await
            .unwrap();
        a.recv::<<ChooseA as Loan>::GrantedFirst>().await.unwrap();
        if attempt == "early_commit" {
            let observed = a.send::<Commit>(&()).await;
            assert!(observed.is_err(), "commit was admitted with only one grant");
            eprintln!(
                "{}",
                serde_json::json!({"kind":"full_1h_global_exclusion", "attempt":attempt,
                "admission":format!("{observed:?}"),"frames":wire.take_observations(),"model_calls":0,"scored_benchmark":false})
            );
            continue;
        }
        db.send::<<ChooseA as Loan>::GrantedSecond>(&())
            .await
            .unwrap();
        a.recv::<<ChooseA as Loan>::GrantedSecond>().await.unwrap();
        a.send::<Commit>(&()).await.unwrap();
        if attempt == "next_round" {
            a.send::<<ChooseA as Loan>::ReturnFirst>(&()).await.unwrap();
            a.send::<<ChooseA as Loan>::ReturnSecond>(&())
                .await
                .unwrap();
            auth.recv::<<ChooseA as Loan>::ReturnFirst>().await.unwrap();
            db.recv::<<ChooseA as Loan>::ReturnSecond>().await.unwrap();
            auth.send::<<ChooseA as Loan>::ReturnedFirst>(&())
                .await
                .unwrap();
            db.send::<<ChooseA as Loan>::ReturnedSecond>(&())
                .await
                .unwrap();
            a.recv::<<ChooseA as Loan>::ReturnedFirst>().await.unwrap();
            a.recv::<<ChooseA as Loan>::ReturnedSecond>().await.unwrap();
            a.send::<Test>(&()).await.unwrap();
            a.offer().await.unwrap().recv::<Implement>().await.unwrap();
        }
        for _ in 0..3 {
            dispatch.send::<SkipC>(&()).await.unwrap();
        }
        for _ in 0..3 {
            dispatch.send::<SkipE>(&()).await.unwrap();
        }
        // Queue the competing branch's real notification before challenging DB.
        for _ in 0..3 {
            dispatch.send::<ChooseB>(&()).await.unwrap();
        }
        b.recv::<ChooseB>().await.unwrap();
        assert_eq!(calls.borrow_mut().pop_front(), Some(B));
        b.send::<SubmittedB>(&()).await.unwrap();
        dispatch.recv::<SubmittedB>().await.unwrap();
        b.send::<<ChooseB as Loan>::Local>(&()).await.unwrap();
        if attempt == "next_round" {
            db.recv::<ChooseB>().await.unwrap();
            db.send::<<ChooseB as Loan>::Local>(&()).await.unwrap();
            db.send::<<ChooseB as Loan>::AvailableFirst>(&())
                .await
                .unwrap();
            b.recv::<<ChooseB as Loan>::AvailableFirst>().await.unwrap();
            b.send::<<ChooseB as Loan>::AcquireFirst>(&())
                .await
                .unwrap();
            db.recv::<<ChooseB as Loan>::AcquireFirst>().await.unwrap();
            db.send::<<ChooseB as Loan>::GrantedFirst>(&())
                .await
                .unwrap();
            b.recv::<<ChooseB as Loan>::GrantedFirst>().await.unwrap();
            // The dispatcher can already enter another round. B still holds
            // DB; this tests exclusion across roll reentry, not a localside wait.
            for _ in 0..3 {
                dispatch.send::<SkipD>(&()).await.unwrap();
            }
            for _ in 0..3 {
                dispatch.send::<SkipF>(&()).await.unwrap();
            }
            for _ in 0..3 {
                dispatch.send::<SkipG>(&()).await.unwrap();
            }
            calls.borrow_mut().push_back(A);
            for _ in 0..3 {
                dispatch.send::<ChooseA>(&()).await.unwrap();
            }
            a.recv::<ChooseA>().await.unwrap();
            assert_eq!(calls.borrow_mut().pop_front(), Some(A));
            a.send::<SubmittedA>(&()).await.unwrap();
            dispatch.recv::<SubmittedA>().await.unwrap();
            a.send::<<ChooseA as Loan>::Local>(&()).await.unwrap();
        }
        let mut effects = Vec::new();
        let observed = {
            let mut trial = std::pin::pin!(async {
                if attempt == "next_round" {
                    db.recv::<ChooseA>().await?;
                    effects.push("notification_received");
                    db.send::<<ChooseA as Loan>::Local>(&()).await?;
                    effects.push("local_step");
                    db.send::<<ChooseA as Loan>::AvailableSecond>(&()).await?;
                    effects.push("readiness_sent");
                    a.recv::<<ChooseA as Loan>::AvailableSecond>().await?;
                    a.send::<<ChooseA as Loan>::AcquireSecond>(&()).await?;
                    db.recv::<<ChooseA as Loan>::AcquireSecond>().await?;
                    db.send::<<ChooseA as Loan>::GrantedSecond>(&()).await?;
                    effects.push("second_grant");
                    return Ok::<_, hibana::EndpointError>(());
                }
                if attempt == "offer" {
                    db.offer().await?.recv::<ChooseB>().await?;
                } else {
                    db.recv::<ChooseB>().await?;
                }
                effects.push("notification_received");
                db.send::<<ChooseB as Loan>::Local>(&()).await?;
                effects.push("local_step");
                db.send::<<ChooseB as Loan>::AvailableFirst>(&()).await?;
                effects.push("readiness_sent");
                b.recv::<<ChooseB as Loan>::AvailableFirst>().await?;
                b.send::<<ChooseB as Loan>::AcquireFirst>(&()).await?;
                db.recv::<<ChooseB as Loan>::AcquireFirst>().await?;
                effects.push("request_received");
                db.send::<<ChooseB as Loan>::GrantedFirst>(&()).await?;
                effects.push("grant_sent");
                Ok::<(), hibana::EndpointError>(())
            });
            let mut observed = Poll::Pending;
            for _ in 0..8 {
                observed = trial.as_mut().poll(&mut Context::from_waker(Waker::noop()));
                if observed.is_ready() {
                    break;
                }
            }
            observed
        };
        assert!(
            !matches!(observed, Poll::Ready(Ok(()))),
            "DB was granted twice before the previous borrower returned it: {effects:?}"
        );
        let observed = format!("{observed:?}");
        let frames = wire.take_observations();
        assert!(frames.iter().any(|f| f["action"] == "send"
            && f["source"] == DISPATCH
            && f["target"] == DATABASE_MODULE));
        eprintln!(
            "{}",
            serde_json::json!({"kind":"full_1h_global_exclusion", "attempt":attempt,"admission":observed,"completed_effects":effects,"frames":frames,"model_calls":0,"scored_benchmark":false})
        );
    }
}
