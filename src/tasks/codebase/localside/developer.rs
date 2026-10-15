//! Dispatch actual requests; the projected endpoint decides which can progress.
use super::super::{
    global::*,
    imp::{Domain, Operation, Request, TestInput, decide, response},
};
use crate::{RunError, resource::global::*};
use hibana::{Endpoint, runtime::resolver::DecisionArm};

/// 1H retries follow the declared resolver and rolled choice.
pub(crate) async fn run<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
    input: &TestInput,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        let operation = call.operation();
        match operation {
            Operation::Design => {
                endpoint.send::<Design>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                input.record(response(operation, result)?);
                endpoint.send::<Attempt>(&()).await?;
            }
            Operation::Implement => {
                endpoint.send::<Implement>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Commit => {
                endpoint.send::<AcquireFirst>(&()).await?;
                endpoint.send::<AcquireSecond>(&()).await?;
                endpoint.recv::<GrantedFirst>().await?;
                endpoint.recv::<GrantedSecond>().await?;
                endpoint.send::<Commit>(&()).await?;
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
            Operation::Test => {
                endpoint.send::<Test>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                input.record(response(operation, result)?);
                // Choosing the entry does not bypass the registered resolver:
                // Endpoint independently checks it before either send commits.
                match decide(input).map_err(RunError::Resolver)? {
                    DecisionArm::Left => {
                        endpoint.send::<Passed>(&()).await?;
                        endpoint.send::<ClosedFirst>(&()).await?;
                        endpoint.send::<ClosedSecond>(&()).await?;
                        endpoint.send::<Finished>(&()).await?;
                        return Ok(());
                    }
                    DecisionArm::Right => endpoint.send::<Attempt>(&()).await?,
                }
            }
        }
    }
}

/// 1E/1M tests are terminal, including a failed test result.
pub(crate) async fn terminal<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        let operation = call.operation();
        match operation {
            Operation::Design => {
                endpoint.send::<Design>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Implement => {
                endpoint.send::<Implement>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Commit => {
                endpoint.send::<AcquireFirst>(&()).await?;
                endpoint.send::<AcquireSecond>(&()).await?;
                endpoint.recv::<GrantedFirst>().await?;
                endpoint.recv::<GrantedSecond>().await?;
                endpoint.send::<Commit>(&()).await?;
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
            Operation::Test => {
                endpoint.send::<Test>(&()).await?;
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
