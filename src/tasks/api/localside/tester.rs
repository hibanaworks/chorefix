use super::super::{
    global::*,
    imp::{Domain, Operation, Request, TestInput, decide},
};
use crate::RunError;
use hibana::{Endpoint, runtime::resolver::DecisionArm};

pub(crate) async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, TESTER>,
    domain: &D,
    input: &TestInput,
) -> Result<(), RunError<String>> {
    endpoint.recv::<BackendReady>().await?;
    let call = domain.request("TESTER").await.map_err(RunError::Domain)?;
    endpoint.send::<Acquire>(&()).await?;
    endpoint.recv::<Granted>().await?;
    match call.operation() {
        Operation::Migrate => endpoint.send::<Migrate>(&()).await?,
        Operation::Implement => endpoint.send::<Implement>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
    }
    let result = domain
        .execute("TESTER", call)
        .await
        .map_err(RunError::Domain)?;
    input.record(result);
    endpoint.send::<Return>(&()).await?;
    endpoint.recv::<Returned>().await?;
    match decide(input).map_err(RunError::Resolver)? {
        DecisionArm::Left => {
            endpoint.send::<PassedToBackend>(&()).await?;
            endpoint.send::<PassedToAdmin>(&()).await?;
        }
        DecisionArm::Right => {
            endpoint.send::<FailedToBackend>(&()).await?;
            endpoint.send::<FailedToAdmin>(&()).await?;
        }
    }
    Ok(())
}
