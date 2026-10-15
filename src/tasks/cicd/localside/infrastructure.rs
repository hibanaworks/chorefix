use super::super::global::*;
use crate::RunError;
use hibana::Endpoint;

pub(crate) async fn build_server(
    endpoint: &mut Endpoint<'_, BUILD_SERVER>,
) -> Result<(), RunError<String>> {
    // Global: the developer returns before the tester can borrow.
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
pub(crate) async fn deploy_env(
    endpoint: &mut Endpoint<'_, DEPLOY_ENV>,
) -> Result<(), RunError<String>> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
