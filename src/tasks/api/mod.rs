pub mod global;
mod imp;
mod localside;
use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
pub use imp::{Domain, Operation, Request};
use imp::{TestInput, decide};

pub async fn run_8e<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    let programs = global::project_8e();
    let input = TestInput::default();
    let mut slab = vec![0; 64 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    rendezvous
        .set_resolver(
            &programs.tester,
            ResolverRef::<{ global::TEST_RESULT }>::decision_state(&input, decide),
        )
        .map_err(RunError::Resolver)?;
    let session = SessionId::new(1);
    let mut admin = rendezvous
        .enter(session, &programs.admin)
        .map_err(RunError::Attach)?;
    let mut backend = rendezvous
        .enter(session, &programs.backend)
        .map_err(RunError::Attach)?;
    let mut tester = rendezvous
        .enter(session, &programs.tester)
        .map_err(RunError::Attach)?;
    let mut database = rendezvous
        .enter(session, &programs.database)
        .map_err(RunError::Attach)?;
    let mut test_env = rendezvous
        .enter(session, &programs.test_env)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::admin::run(&mut admin, domain),
        localside::backend::run(&mut backend, domain),
        localside::tester::run(&mut tester, domain, &input),
        localside::resources::database(&mut database),
        localside::resources::test_env(&mut test_env)
    )?;
    Ok(())
}
