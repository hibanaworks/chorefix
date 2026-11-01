use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;
#[hibana_runtime::scope]
pub async fn run<'r, const ROLE: u8, const RESOURCE: u8>(
    endpoint: &mut Endpoint<'r, ROLE>,
    resource: Endpoint<'r, RESOURCE>,
) -> ShellResult<()> {
    endpoint.send::<Loan<RESOURCE>>(resource).await?;
    let _resource = endpoint.recv::<Loan<RESOURCE>>().await?;
    Ok(())
}
