//! Received tool kinds select handlers, never a stored expected phase.
use super::super::{
    global::*,
    imp::{Domain, Operation, Request, response},
};
use crate::{RunError, resource::global::*};
use hibana::Endpoint;

pub(crate) async fn run<const ROLE: u8, D: Domain>(
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
                endpoint.send::<AcquireFirst>(&()).await?;
                endpoint.recv::<GrantedFirst>().await?;
                endpoint.send::<AcquireSecond>(&()).await?;
                endpoint.recv::<GrantedSecond>().await?;
                endpoint.send::<Write>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
                endpoint.send::<ReturnSecond>(&()).await?;
                endpoint.send::<ReturnFirst>(&()).await?;
                endpoint.recv::<ReturnedSecond>().await?;
                endpoint.recv::<ReturnedFirst>().await?;
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
