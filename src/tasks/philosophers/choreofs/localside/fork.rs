//! A physical fork has exactly its two projected loans, in global order.
use crate::shell::ShellResult;
use crate::tasks::philosophers::global::ForkMessages;
use hibana::Endpoint;

pub(crate) async fn run<const ROLE: u8, M: ForkMessages>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> ShellResult<()> {
    endpoint.send::<M::Available>(&()).await?;
    endpoint.recv::<M::Acquire>().await?;
    endpoint.send::<M::Granted>(&()).await?;
    endpoint.recv::<M::Return>().await?;
    endpoint.send::<M::Returned>(&()).await?;
    endpoint.send::<M::Available>(&()).await?;
    endpoint.recv::<M::Acquire>().await?;
    endpoint.send::<M::Granted>(&()).await?;
    endpoint.recv::<M::Return>().await?;
    endpoint.send::<M::Returned>(&()).await?;
    Ok(())
}
