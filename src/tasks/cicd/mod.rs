pub mod choreofs;
pub mod global;
mod imp;
mod localside;
pub use imp::{Domain, Operation, Request};

use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId};

pub async fn run_16e<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    let programs = global::project_16e();
    let mut slab = vec![0; 32 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, &transport_storage)
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut developer = rendezvous
        .enter(session, &programs.developer)
        .map_err(RunError::Attach)?;
    let mut tester = rendezvous
        .enter(session, &programs.tester)
        .map_err(RunError::Attach)?;
    let mut deployer = rendezvous
        .enter(session, &programs.deployer)
        .map_err(RunError::Attach)?;
    let mut build_server = rendezvous
        .enter(session, &programs.build_server)
        .map_err(RunError::Attach)?;
    let mut deploy_env = rendezvous
        .enter(session, &programs.deploy_env)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::developer::run(&mut developer, domain),
        localside::tester::run(&mut tester, domain),
        localside::deployer::run(&mut deployer, domain),
        localside::infrastructure::build_server(&mut build_server),
        localside::infrastructure::deploy_env(&mut deploy_env),
    )?;
    Ok(())
}
