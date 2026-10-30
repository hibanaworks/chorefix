//! Drive the current 1H projection without its production module localside.
use choreofix::tasks::codebase::choreofs::{
    global::{roles1h::*, *},
    wire::{COMMIT, Close, DESIGN, IMPLEMENT},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, transport::EndpointBinding};
use hibana_wasip1_runtime::protocol;
use std::{future::Future, task::Poll};

#[path = "../src/imp/transport.rs"]
mod transport;

#[tokio::test(flavor = "current_thread")]
async fn the_actual_1h_database_endpoint_reaches_b_only_after_a_returns_it() {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let programs = project_1h();
        let session = SessionId::new(9020);
        let bindings = [AUTH_RESOURCE, DATABASE_RESOURCE].map(|role| {
            // SAFETY: these are the sole issuers for this test's two physical modules.
            #[allow(unsafe_code)]
            let binding = unsafe { EndpointBinding::new() };
            (session, role, binding)
        });
        let wire = transport::InMemoryTransport::new(bindings);
        let mut slab = vec![0; 2 * 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let runtime = storage.init().rendezvous(&mut slab, &wire).unwrap();
        let mut a = runtime.enter(session, &programs.a).unwrap();
        let mut a_wasi = runtime.enter(session, &programs.a_wasi).unwrap();
        let mut b = runtime.enter(session, &programs.b).unwrap();
        let mut b_wasi = runtime.enter(session, &programs.b_wasi).unwrap();
        let mut auth = runtime.enter(session, &programs.auth).unwrap();
        let mut database = runtime.enter(session, &programs.database).unwrap();
        let auth_resource = runtime.enter(session, &programs.auth_resource).unwrap();
        let database_resource = runtime.enter(session, &programs.database_resource).unwrap();
        let [mut a_task, _a_private] = a_wasi.par().unwrap();
        let [mut b_task, _b_private] = b_wasi.par().unwrap();
        let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
        let result: &[u8] = b"{}";

        let design_a = Close::<HARD, A, DESIGN>::new(
            close,
            serde_json::json!({"feature_name":"user_authentication"}),
        )
        .unwrap();
        a_task
            .send::<DesignRequest<HARD, A>>(&design_a)
            .await
            .unwrap();
        a.recv::<DesignRequest<HARD, A>>().await.unwrap();
        a.send::<DesignCompleted<'_>>(&result).await.unwrap();
        a_task.recv::<DesignCompleted<'_>>().await.unwrap();
        let implement_a = Close::<HARD, A, IMPLEMENT>::new(
            close,
            serde_json::json!({"feature_name":"user_authentication"}),
        )
        .unwrap();
        a_task
            .send::<ImplementRequest<HARD, A>>(&implement_a)
            .await
            .unwrap();
        a.recv::<ImplementRequest<HARD, A>>().await.unwrap();
        a.send::<ImplementCompleted<'_>>(&result).await.unwrap();
        a_task.recv::<ImplementCompleted<'_>>().await.unwrap();

        let design_b = Close::<HARD, B, DESIGN>::new(
            close,
            serde_json::json!({"feature_name":"rest_api_endpoints"}),
        )
        .unwrap();
        b_task
            .send::<DesignRequest<HARD, B>>(&design_b)
            .await
            .unwrap();
        b.recv::<DesignRequest<HARD, B>>().await.unwrap();
        b.send::<DesignCompleted<'_>>(&result).await.unwrap();
        b_task.recv::<DesignCompleted<'_>>().await.unwrap();
        let implement_b = Close::<HARD, B, IMPLEMENT>::new(
            close,
            serde_json::json!({"feature_name":"rest_api_endpoints"}),
        )
        .unwrap();
        b_task
            .send::<ImplementRequest<HARD, B>>(&implement_b)
            .await
            .unwrap();
        b.recv::<ImplementRequest<HARD, B>>().await.unwrap();
        b.send::<ImplementCompleted<'_>>(&result).await.unwrap();
        b_task.recv::<ImplementCompleted<'_>>().await.unwrap();

        auth.send::<Loan<'_, AUTH_RESOURCE>>(auth_resource)
            .await
            .unwrap();
        database
            .send::<Loan<'_, DATABASE_RESOURCE>>(database_resource)
            .await
            .unwrap();
        let [mut a_auth, mut a_database] = a_task.par().unwrap();
        let (mut auth_resource, mut database_resource) = tokio::try_join!(
            a_auth.recv::<Loan<'_, AUTH_RESOURCE>>(),
            a_database.recv::<Loan<'_, DATABASE_RESOURCE>>()
        )
        .unwrap();
        assert!(runtime.enter(session, &programs.database_resource).is_err());

        // B has already completed its real prerequisites. No module localside or
        // owner table decides whether this receive is available.
        let [_b_api, mut b_database] = b_task.par().unwrap();
        let mut waiting = std::pin::pin!(b_database.recv::<Loan<'_, DATABASE_RESOURCE>>());
        assert!(
            std::future::poll_fn(|cx| Poll::Ready(waiting.as_mut().poll(cx)))
                .await
                .is_pending()
        );

        let commit = Close::<HARD, A, COMMIT>::new(
            close,
            serde_json::json!({"modules":"AUTH_MODULE,DATABASE_MODULE"}),
        )
        .unwrap();
        tokio::try_join!(
            auth_resource.send::<CommitRequest<HARD, A, AUTH_RESOURCE>>(&commit),
            database_resource.send::<CommitRequest<HARD, A, DATABASE_RESOURCE>>(&commit)
        )
        .unwrap();
        {
            let [mut first, mut second] = a.par().unwrap();
            tokio::try_join!(
                first.recv::<CommitRequest<HARD, A, AUTH_RESOURCE>>(),
                second.recv::<CommitRequest<HARD, A, DATABASE_RESOURCE>>()
            )
            .unwrap();
        }
        {
            let [mut first, mut second] = a.par().unwrap();
            tokio::try_join!(
                first.send::<CommitCompleted<'_, AUTH_RESOURCE>>(&result),
                second.send::<CommitCompleted<'_, DATABASE_RESOURCE>>(&result)
            )
            .unwrap();
        }
        tokio::try_join!(
            auth_resource.recv::<CommitCompleted<'_, AUTH_RESOURCE>>(),
            database_resource.recv::<CommitCompleted<'_, DATABASE_RESOURCE>>()
        )
        .unwrap();
        drop((a_auth, a_database));
        {
            let [mut first, mut second] = a_task.par().unwrap();
            tokio::try_join!(
                first.send::<Loan<'_, AUTH_RESOURCE>>(auth_resource),
                second.send::<Loan<'_, DATABASE_RESOURCE>>(database_resource)
            )
            .unwrap();
        }
        let _auth_resource = auth.recv::<Loan<'_, AUTH_RESOURCE>>().await.unwrap();
        let database_resource = database
            .recv::<Loan<'_, DATABASE_RESOURCE>>()
            .await
            .unwrap();
        database
            .send::<Loan<'_, DATABASE_RESOURCE>>(database_resource)
            .await
            .unwrap();
        let _b_resource = waiting.await.unwrap();
        assert!(runtime.enter(session, &programs.database_resource).is_err());
        let frames = wire.take_observations();
        let sends = |from: u8, to: u8| {
            frames
                .iter()
                .filter(|frame| {
                    frame["action"] == "send" && frame["source"] == from && frame["target"] == to
                })
                .collect::<Vec<_>>()
        };
        let first_loan = sends(DATABASE, A_WASI);
        let returned = sends(A_WASI, DATABASE);
        let next_loan = sends(DATABASE, B_WASI);
        assert_eq!(
            (first_loan.len(), returned.len(), next_loan.len()),
            (1, 1, 1)
        );
        assert!(
            first_loan[0]["frame_id"].as_u64().unwrap() < returned[0]["frame_id"].as_u64().unwrap()
        );
        assert!(
            returned[0]["frame_id"].as_u64().unwrap() < next_loan[0]["frame_id"].as_u64().unwrap()
        );
        eprintln!(
            "{}",
            serde_json::json!({
                "kind":"owned_1h_global_exclusion_prefix",
                "task":"1H",
                "source":"current choreofs global",
                "production_module_localside":false,
                "scored_benchmark":false,
                "frames":frames
            })
        );
    })
    .await
    .expect("the actual 1H loan/commit/return prefix stalled");
}
