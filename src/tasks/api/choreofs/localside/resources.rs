use super::super::global::*;
use crate::shell::ShellResult;
use hibana::Endpoint;
pub async fn run<const ROLE: u8>(endpoint: &mut Endpoint<'_, ROLE>) -> ShellResult<()> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
