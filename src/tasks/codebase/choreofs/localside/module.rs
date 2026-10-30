//! The same module endpoint returns before its next borrower can use it.
use super::super::global::Loan;
use crate::shell::ShellResult;
use hibana::Endpoint;

pub(crate) async fn run<'r, const ROLE: u8, const RESOURCE: u8>(
    endpoint: &mut Endpoint<'r, ROLE>,
    resource: Endpoint<'r, RESOURCE>,
) -> ShellResult<()> {
    endpoint.send::<Loan<'_, RESOURCE>>(resource).await?;
    let resource = endpoint.recv::<Loan<'_, RESOURCE>>().await?;
    endpoint.send::<Loan<'_, RESOURCE>>(resource).await?;
    let _resource = endpoint.recv::<Loan<'_, RESOURCE>>().await?;
    Ok(())
}
