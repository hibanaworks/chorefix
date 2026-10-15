use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;

pub async fn build_server(endpoint: &mut Endpoint<'_, BUILD_SERVER>) -> ShellResult<()> {
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
pub async fn deploy_env(endpoint: &mut Endpoint<'_, DEPLOY_ENV>) -> ShellResult<()> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
