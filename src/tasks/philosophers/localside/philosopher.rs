use crate::imp::domain::{Domain, RunError, Work, WorkRequest};
use crate::tasks::philosophers::global::{self as p, Eat, Finished, ForkA, ForkB, ForkC, Think};
use hibana::Endpoint;

async fn work<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<D::Error>> {
    let call = domain.request(agent).await.map_err(RunError::Domain)?;
    // Static message dispatch. The endpoint alone decides whether this call
    // is in order; no phase variable or expected-operation table is used.
    match call.work() {
        Work::Think => endpoint.send::<Think>(&()).await?,
        Work::Eat => endpoint.send::<Eat>(&()).await?,
    }
    domain.execute(agent, call).await.map_err(RunError::Domain)
}

pub(crate) async fn run<const ROLE: u8, L: p::ForkMessages, R: p::ForkMessages, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<D::Error>> {
    work(agent, endpoint, domain).await?;
    super::acquire::<ROLE, L>(endpoint).await?;
    super::acquire::<ROLE, R>(endpoint).await?;
    work(agent, endpoint, domain).await?;
    super::return_fork::<ROLE, R>(endpoint).await?;
    super::return_fork::<ROLE, L>(endpoint).await?;
    endpoint.send::<Finished>(&()).await?;
    Ok(())
}

pub(crate) async fn a<D: Domain>(
    endpoint: &mut Endpoint<'_, { p::A }>,
    domain: &D,
) -> Result<(), RunError<D::Error>> {
    run::<{ p::A }, ForkA, ForkC, D>("PHILOSOPHER_A", endpoint, domain).await
}

pub(crate) async fn b<D: Domain>(
    endpoint: &mut Endpoint<'_, { p::B }>,
    domain: &D,
) -> Result<(), RunError<D::Error>> {
    run::<{ p::B }, ForkB, ForkA, D>("PHILOSOPHER_B", endpoint, domain).await
}

pub(crate) async fn c<D: Domain>(
    endpoint: &mut Endpoint<'_, { p::C }>,
    domain: &D,
) -> Result<(), RunError<D::Error>> {
    run::<{ p::C }, ForkC, ForkB, D>("PHILOSOPHER_C", endpoint, domain).await
}
