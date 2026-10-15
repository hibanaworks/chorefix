use super::super::{
    global::*,
    imp::{Domain, Operation, Request},
};
use crate::RunError;
use hibana::{Endpoint, g::Message};

pub(crate) async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, DB_ADMIN>,
    domain: &D,
) -> Result<(), RunError<String>> {
    let call = domain.request("DB_ADMIN").await.map_err(RunError::Domain)?;
    endpoint.send::<Acquire>(&()).await?;
    endpoint.recv::<Granted>().await?;
    match call.operation() {
        Operation::Migrate => endpoint.send::<Migrate>(&()).await?,
        Operation::Implement => endpoint.send::<Implement>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
    }
    let result = domain
        .execute("DB_ADMIN", call)
        .await
        .map_err(RunError::Domain)?;
    if result["success"] != true {
        return Err(RunError::Domain(format!(
            "domain operation failed: {result}"
        )));
    }
    endpoint.send::<Return>(&()).await?;
    endpoint.recv::<Returned>().await?;
    endpoint.send::<MigrationDone>(&()).await?;
    let branch = endpoint.offer().await?;
    match branch.label() {
        PassedToAdmin::LOGICAL_LABEL => {
            branch.recv::<PassedToAdmin>().await?;
        }
        FailedToAdmin::LOGICAL_LABEL => {
            branch.recv::<FailedToAdmin>().await?;
        }
        _ => unreachable!("projected terminal test result"),
    }
    Ok(())
}
