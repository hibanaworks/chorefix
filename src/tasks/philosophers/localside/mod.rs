pub(crate) mod fork;
pub(crate) mod philosopher;

use super::global::ForkMessages;
use hibana::{Endpoint, EndpointError};

async fn acquire<const ROLE: u8, M: ForkMessages>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> Result<(), EndpointError> {
    endpoint.recv::<M::Available>().await?;
    endpoint.send::<M::Acquire>(&()).await?;
    endpoint.recv::<M::Granted>().await?;
    Ok(())
}

async fn return_fork<const ROLE: u8, M: ForkMessages>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> Result<(), EndpointError> {
    endpoint.send::<M::Return>(&()).await?;
    endpoint.recv::<M::Returned>().await?;
    Ok(())
}
