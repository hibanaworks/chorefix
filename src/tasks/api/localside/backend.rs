use super::super::{
    global::*,
    imp::{Domain, Operation, Request},
};
use crate::RunError;
use hibana::{Endpoint, g::Message};

pub(crate) async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, BACKEND_DEV>,
    domain: &D,
) -> Result<(), RunError<String>> {
    endpoint.recv::<MigrationDone>().await?;
    let call = domain
        .request("BACKEND_DEV")
        .await
        .map_err(RunError::Domain)?;
    match call.operation() {
        Operation::Migrate => endpoint.send::<Migrate>(&()).await?,
        Operation::Implement => endpoint.send::<Implement>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
    }
    let result = domain
        .execute("BACKEND_DEV", call)
        .await
        .map_err(RunError::Domain)?;
    if result["success"] != true {
        return Err(RunError::Domain(format!(
            "domain operation failed: {result}"
        )));
    }
    endpoint.send::<BackendReady>(&()).await?;
    let branch = endpoint.offer().await?;
    match branch.label() {
        PassedToBackend::LOGICAL_LABEL => {
            branch.recv::<PassedToBackend>().await?;
        }
        FailedToBackend::LOGICAL_LABEL => {
            branch.recv::<FailedToBackend>().await?;
        }
        _ => unreachable!("projected terminal test result"),
    }
    Ok(())
}
