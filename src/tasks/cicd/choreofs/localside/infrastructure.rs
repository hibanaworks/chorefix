use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;

#[hibana_runtime::localside]
pub async fn build_server<'r>(
    endpoint: &mut Endpoint<'r, BUILD_OWNER>,
    build: Endpoint<'r, BUILD_SERVER>,
) -> ShellResult<()> {
    endpoint.send::<Loan<BUILD_SERVER>>(build).await?;
    let build = endpoint.recv::<Loan<BUILD_SERVER>>().await?;
    endpoint.send::<Loan<BUILD_SERVER>>(build).await?;
    let _build = endpoint.recv::<Loan<BUILD_SERVER>>().await?;
    Ok(())
}

#[hibana_runtime::localside]
pub async fn deploy_env<'r>(
    endpoint: &mut Endpoint<'r, DEPLOY_OWNER>,
    environment: Endpoint<'r, DEPLOY_ENV>,
) -> ShellResult<()> {
    endpoint.send::<Loan<DEPLOY_ENV>>(environment).await?;
    let _environment = endpoint.recv::<Loan<DEPLOY_ENV>>().await?;
    Ok(())
}
