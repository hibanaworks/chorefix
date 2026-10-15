pub mod choreofs;
pub mod global;
mod imp;
mod localside;
use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
pub use imp::{Domain, Operation, Request};
use imp::{TestInput, decide};

/// Seven concurrent workflows and physical owners, all projected from one global.
pub async fn run_1h<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_1h();
    let inputs: [TestInput; 7] = std::array::from_fn(|_| TestInput::default());
    let mut slab = vec![0; 512 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<TEST_A>::decision_state(&inputs[0], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<TEST_B>::decision_state(&inputs[1], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<TEST_C>::decision_state(&inputs[2], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<TEST_D>::decision_state(&inputs[3], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<TEST_E>::decision_state(&inputs[4], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.f,
        ResolverRef::<TEST_F>::decision_state(&inputs[5], decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &programs.g,
        ResolverRef::<TEST_G>::decision_state(&inputs[6], decide),
    )
    .map_err(RunError::Resolver)?;
    let session = SessionId::new(1);
    let mut a = rv.enter(session, &programs.a).map_err(RunError::Attach)?;
    let mut b = rv.enter(session, &programs.b).map_err(RunError::Attach)?;
    let mut c = rv.enter(session, &programs.c).map_err(RunError::Attach)?;
    let mut d = rv.enter(session, &programs.d).map_err(RunError::Attach)?;
    let mut e = rv.enter(session, &programs.e).map_err(RunError::Attach)?;
    let mut f = rv.enter(session, &programs.f).map_err(RunError::Attach)?;
    let mut g = rv.enter(session, &programs.g).map_err(RunError::Attach)?;
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
    let mut analytics = rv
        .enter(session, &programs.analytics)
        .map_err(RunError::Attach)?;
    let mut search = rv
        .enter(session, &programs.search)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::developer::run::<_, _, global::reconsider::LoanA>(
            "DEVELOPER_A",
            &mut a,
            domain,
            &inputs[0]
        ),
        localside::developer::run::<_, _, global::reconsider::LoanB>(
            "DEVELOPER_B",
            &mut b,
            domain,
            &inputs[1]
        ),
        localside::developer::run::<_, _, global::reconsider::LoanC>(
            "DEVELOPER_C",
            &mut c,
            domain,
            &inputs[2]
        ),
        localside::developer::run::<_, _, global::reconsider::LoanD>(
            "DEVELOPER_D",
            &mut d,
            domain,
            &inputs[3]
        ),
        localside::developer::run::<_, _, global::reconsider::LoanE>(
            "DEVELOPER_E",
            &mut e,
            domain,
            &inputs[4]
        ),
        localside::developer::run::<_, _, global::reconsider::LoanF>(
            "DEVELOPER_F",
            &mut f,
            domain,
            &inputs[5]
        ),
        localside::developer::run::<_, _, global::reconsider::LoanG>(
            "DEVELOPER_G",
            &mut g,
            domain,
            &inputs[6]
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
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
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
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
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
