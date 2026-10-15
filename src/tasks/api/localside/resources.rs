use super::super::global::*;
use crate::RunError;
use hibana::Endpoint;

pub(crate) async fn database(
    endpoint: &mut Endpoint<'_, DATABASE>,
) -> Result<(), RunError<String>> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
pub(crate) async fn test_env(
    endpoint: &mut Endpoint<'_, TEST_ENV>,
) -> Result<(), RunError<String>> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
