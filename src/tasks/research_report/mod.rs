pub mod global;
mod imp;
mod localside;
use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
pub use imp::{Domain, Operation, Request};
use imp::{ReviewInput, decide};

pub async fn run_3e<D: Domain>(domain: &D) -> Result<Vec<serde_json::Value>, RunError<String>> {
    let p = global::project_3e();
    let review_a = ReviewInput::default();
    let review_b = ReviewInput::default();
    let mut slab = vec![0; 256 * 1024];
    let carrier = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit
        .rendezvous(&mut slab, &carrier)
        .map_err(RunError::Attach)?;
    rv.set_resolver(
        &p.editor,
        ResolverRef::<{ global::REVIEW_A }>::decision_state(&review_a, decide),
    )
    .map_err(RunError::Resolver)?;
    rv.set_resolver(
        &p.editor,
        ResolverRef::<{ global::REVIEW_B }>::decision_state(&review_b, decide),
    )
    .map_err(RunError::Resolver)?;
    let session = SessionId::new(47);
    let mut a = rv.enter(session, &p.a).map_err(RunError::Attach)?;
    let mut b = rv.enter(session, &p.b).map_err(RunError::Attach)?;
    let mut editor = rv.enter(session, &p.editor).map_err(RunError::Attach)?;
    let mut document = rv.enter(session, &p.document).map_err(RunError::Attach)?;
    let mut database = rv.enter(session, &p.database).map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::author::author_a(&mut a, domain),
        localside::author::author_b(&mut b, domain),
        localside::editor::run(&mut editor, domain, &review_a, &review_b),
        localside::resources::document(&mut document),
        localside::resources::database(&mut database)
    )?;
    Ok(carrier.take_observations())
}
