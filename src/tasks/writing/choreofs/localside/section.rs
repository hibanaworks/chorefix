//! The same section endpoint returns before it is lent to its next writer.
use super::super::global::Loan;
use crate::shell::ShellResult;
use hibana::Endpoint;

#[hibana_runtime::localside]
pub(crate) async fn run<'r, const ROLE: u8, const RESOURCE: u8>(
    endpoint: &mut Endpoint<'r, ROLE>,
    resource: Endpoint<'r, RESOURCE>,
) -> ShellResult<()> {
    endpoint.send::<Loan<RESOURCE>>(resource).await?;
    let resource = endpoint.recv::<Loan<RESOURCE>>().await?;
    endpoint.send::<Loan<RESOURCE>>(resource).await?;
    let _resource = endpoint.recv::<Loan<RESOURCE>>().await?;
    Ok(())
}
