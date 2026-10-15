//! Received tool kinds select handlers, never a stored expected phase.
use super::super::{
    global::*,
    imp::{Domain, Operation, Request, response},
};
use crate::RunError;
use hibana::Endpoint;

pub(crate) async fn run<const ROLE: u8, F: SectionMessages, S: SectionMessages, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        let operation = call.operation();
        match operation {
            Operation::Literature => {
                endpoint.send::<Literature>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Draft => {
                endpoint.send::<Draft>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Write => {
                endpoint.recv::<F::Available>().await?;
                endpoint.send::<F::Acquire>(&()).await?;
                endpoint.recv::<F::Granted>().await?;
                endpoint.recv::<S::Available>().await?;
                endpoint.send::<S::Acquire>(&()).await?;
                endpoint.recv::<S::Granted>().await?;
                endpoint.send::<Write>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
                endpoint.send::<S::Return>(&()).await?;
                endpoint.send::<F::Return>(&()).await?;
                endpoint.recv::<S::Returned>().await?;
                endpoint.recv::<F::Returned>().await?;
            }
            Operation::Review => {
                endpoint.send::<Review>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
                endpoint.send::<Finished>(&()).await?;
                return Ok(());
            }
        }
    }
}
