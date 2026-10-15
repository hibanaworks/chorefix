use crate::tasks::philosophers::global::{self as p, ForkA, ForkB, ForkC, ForkMessages};
use hibana::{Endpoint, EndpointError};

async fn lend<const ROLE: u8, M: ForkMessages>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> Result<(), EndpointError> {
    endpoint.send::<M::Available>(&()).await?;
    endpoint.recv::<M::Acquire>().await?;
    endpoint.send::<M::Granted>(&()).await?;
    endpoint.recv::<M::Return>().await?;
    endpoint.send::<M::Returned>(&()).await
}

pub(crate) async fn run<const ROLE: u8, M: ForkMessages>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> Result<(), EndpointError> {
    lend::<ROLE, M>(endpoint).await?;
    lend::<ROLE, M>(endpoint).await
}

pub(crate) async fn a(endpoint: &mut Endpoint<'_, { p::FORK_A }>) -> Result<(), EndpointError> {
    run::<{ p::FORK_A }, ForkA>(endpoint).await
}
pub(crate) async fn b(endpoint: &mut Endpoint<'_, { p::FORK_B }>) -> Result<(), EndpointError> {
    run::<{ p::FORK_B }, ForkB>(endpoint).await
}
pub(crate) async fn c(endpoint: &mut Endpoint<'_, { p::FORK_C }>) -> Result<(), EndpointError> {
    run::<{ p::FORK_C }, ForkC>(endpoint).await
}
