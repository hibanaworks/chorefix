//! Native session attachment and direct localside scheduling.
use super::{
    global,
    imp::{Domain, Requests, complete, decide, ready},
    localside,
};
use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};

/// Seven developers and physical owners share one repeated loan round.
pub async fn run_1h<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::{reconsider::*, *};
    let programs = project_1h();
    let requests = Requests::<D::Call>::new();
    let mut slab = vec![0; 512 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let rv = storage
        .init()
        .rendezvous(&mut slab, &transport_storage)
        .map_err(RunError::Attach)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_A>::decision_state(&requests, ready::<0, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_B>::decision_state(&requests, ready::<1, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_C>::decision_state(&requests, ready::<2, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_D>::decision_state(&requests, ready::<3, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_E>::decision_state(&requests, ready::<4, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_F>::decision_state(&requests, ready::<5, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<READY_G>::decision_state(&requests, ready::<6, D::Call>),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.dispatch,
        ResolverRef::<ALL_TESTS>::decision_state(&requests, complete),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<TEST_A>::decision_state(&requests.results[0], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<TEST_B>::decision_state(&requests.results[1], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<TEST_C>::decision_state(&requests.results[2], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<TEST_D>::decision_state(&requests.results[3], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<TEST_E>::decision_state(&requests.results[4], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.f,
        ResolverRef::<TEST_F>::decision_state(&requests.results[5], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.g,
        ResolverRef::<TEST_G>::decision_state(&requests.results[6], decide),
    )
    .map_err(RunError::Resolver)?;
    let sid = SessionId::new(1);
    let mut dispatch = rv
        .enter(sid, &programs.dispatch)
        .map_err(RunError::Attach)?;
    let mut a = rv.enter(sid, &programs.a).map_err(RunError::Attach)?;
    let mut b = rv.enter(sid, &programs.b).map_err(RunError::Attach)?;
    let mut c = rv.enter(sid, &programs.c).map_err(RunError::Attach)?;
    let mut d = rv.enter(sid, &programs.d).map_err(RunError::Attach)?;
    let mut e = rv.enter(sid, &programs.e).map_err(RunError::Attach)?;
    let mut f = rv.enter(sid, &programs.f).map_err(RunError::Attach)?;
    let mut g = rv.enter(sid, &programs.g).map_err(RunError::Attach)?;
    let mut auth = rv.enter(sid, &programs.auth).map_err(RunError::Attach)?;
    let mut database = rv
        .enter(sid, &programs.database)
        .map_err(RunError::Attach)?;
    let mut api = rv.enter(sid, &programs.api).map_err(RunError::Attach)?;
    let mut payment = rv.enter(sid, &programs.payment).map_err(RunError::Attach)?;
    let mut notify = rv.enter(sid, &programs.notify).map_err(RunError::Attach)?;
    let mut analytics = rv
        .enter(sid, &programs.analytics)
        .map_err(RunError::Attach)?;
    let mut search = rv.enter(sid, &programs.search).map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::dispatch::run(&mut dispatch, &requests),
        localside::developer::run::<_, _, ChooseA, SkipA, SubmittedA, FinishA, DoneA>(
            "DEVELOPER_A",
            &mut a,
            domain,
            &requests
        ),
        localside::developer::run::<_, _, ChooseB, SkipB, SubmittedB, FinishB, DoneB>(
            "DEVELOPER_B",
            &mut b,
            domain,
            &requests
        ),
        localside::developer::run::<_, _, ChooseC, SkipC, SubmittedC, FinishC, DoneC>(
            "DEVELOPER_C",
            &mut c,
            domain,
            &requests
        ),
        localside::developer::run::<_, _, ChooseD, SkipD, SubmittedD, FinishD, DoneD>(
            "DEVELOPER_D",
            &mut d,
            domain,
            &requests
        ),
        localside::developer::run::<_, _, ChooseE, SkipE, SubmittedE, FinishE, DoneE>(
            "DEVELOPER_E",
            &mut e,
            domain,
            &requests
        ),
        localside::developer::run::<_, _, ChooseF, SkipF, SubmittedF, FinishF, DoneF>(
            "DEVELOPER_F",
            &mut f,
            domain,
            &requests
        ),
        localside::developer::run::<_, _, ChooseG, SkipG, SubmittedG, FinishG, DoneG>(
            "DEVELOPER_G",
            &mut g,
            domain,
            &requests
        ),
        localside::reconsider_module::auth(&mut auth),
        localside::reconsider_module::database(&mut database),
        localside::reconsider_module::api(&mut api),
        localside::reconsider_module::payment(&mut payment),
        localside::reconsider_module::notify(&mut notify),
        localside::reconsider_module::analytics(&mut analytics),
        localside::reconsider_module::search(&mut search),
    )?;
    Ok(())
}
/// Independent preparation and tests, with physical module handoffs in one global.
pub async fn run_1e<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_1e();
    let mut slab = vec![0; 192 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit
        .rendezvous(&mut slab, &transport_storage)
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rv.enter(session, &programs.a).map_err(RunError::Attach)?;
    let mut b = rv.enter(session, &programs.b).map_err(RunError::Attach)?;
    let mut c = rv.enter(session, &programs.c).map_err(RunError::Attach)?;
    let mut auth = rv
        .enter(session, &programs.auth)
        .map_err(RunError::Attach)?;
    let mut database = rv
        .enter(session, &programs.database)
        .map_err(RunError::Attach)?;
    let mut api = rv.enter(session, &programs.api).map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::developer::terminal("DEVELOPER_A", &mut a, domain),
        localside::developer::terminal("DEVELOPER_B", &mut b, domain),
        localside::developer::terminal("DEVELOPER_C", &mut c, domain),
        localside::module::auth_1e(&mut auth),
        localside::module::database_1e(&mut database),
        localside::module::api_1e(&mut api),
    )?;
    Ok(())
}
/// Independent preparation and tests, with physical module handoffs in one global.
pub async fn run_1m<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_1m();
    let mut slab = vec![0; 192 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit
        .rendezvous(&mut slab, &transport_storage)
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rv.enter(session, &programs.a).map_err(RunError::Attach)?;
    let mut b = rv.enter(session, &programs.b).map_err(RunError::Attach)?;
    let mut c = rv.enter(session, &programs.c).map_err(RunError::Attach)?;
    let mut d = rv.enter(session, &programs.d).map_err(RunError::Attach)?;
    let mut e = rv.enter(session, &programs.e).map_err(RunError::Attach)?;
    let mut auth = rv
        .enter(session, &programs.auth)
        .map_err(RunError::Attach)?;
    let mut database = rv
        .enter(session, &programs.database)
        .map_err(RunError::Attach)?;
    let mut api = rv.enter(session, &programs.api).map_err(RunError::Attach)?;
    let mut payment = rv
        .enter(session, &programs.payment)
        .map_err(RunError::Attach)?;
    let mut notify = rv
        .enter(session, &programs.notify)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::developer::terminal("DEVELOPER_A", &mut a, domain),
        localside::developer::terminal("DEVELOPER_B", &mut b, domain),
        localside::developer::terminal("DEVELOPER_C", &mut c, domain),
        localside::developer::terminal("DEVELOPER_D", &mut d, domain),
        localside::developer::terminal("DEVELOPER_E", &mut e, domain),
        localside::module::auth_1m(&mut auth),
        localside::module::database_1m(&mut database),
        localside::module::api_1m(&mut api),
        localside::module::payment_1m(&mut payment),
        localside::module::notify_1m(&mut notify),
    )?;
    Ok(())
}
