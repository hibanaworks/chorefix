use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;
pub async fn run<'r, const ROLE: u8, const RESOURCE: u8>(
    endpoint: &mut Endpoint<'r, ROLE>,
    resource: Endpoint<'r, RESOURCE>,
) -> ShellResult<()> {
    endpoint.send::<Loan<'_, RESOURCE>>(resource).await?;
    let _resource = endpoint.recv::<Loan<'_, RESOURCE>>().await?;
    Ok(())
}
