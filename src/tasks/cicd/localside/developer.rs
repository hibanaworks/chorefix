use super::super::{
    global::*,
    imp::{Domain, Operation, Request},
};
use crate::RunError;
use hibana::Endpoint;

pub(crate) async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, DEVELOPER>,
    domain: &D,
) -> Result<(), RunError<String>> {
    endpoint.send::<Acquire>(&()).await?;
    endpoint.recv::<Granted>().await?;
    let call = domain
        .request("DEVELOPER")
        .await
        .map_err(RunError::Domain)?;
    match call.operation() {
        Operation::Compile => endpoint.send::<Compile>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
        Operation::Deploy => endpoint.send::<Deploy>(&()).await?,
    }
    let result = domain
        .execute("DEVELOPER", call)
        .await
        .map_err(RunError::Domain)?;
    if result["success"] != true {
        return Err(RunError::Domain(format!(
            "domain operation failed: {result}"
        )));
    }
    endpoint.send::<Return>(&()).await?;
    endpoint.recv::<Returned>().await?;
    endpoint.send::<BuildComplete>(&()).await?;
    endpoint.recv::<DeployedToDeveloper>().await?;
    Ok(())
}
