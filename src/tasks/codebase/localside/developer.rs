use super::super::{
    global::*,
    imp::{Domain, Operation, Request, TestInput, ToolResponse, decide},
};
use crate::RunError;
use hibana::{Endpoint, runtime::resolver::DecisionArm};

async fn execute_requested_tool<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<ToolResponse, RunError<String>> {
    let call = domain.request(agent).await.map_err(RunError::Domain)?;
    let operation = call.operation();
    match operation {
        Operation::Design => endpoint.send::<Design>(&()).await?,
        Operation::Implement => endpoint.send::<Implement>(&()).await?,
        Operation::Commit => endpoint.send::<Commit>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
    }
    let result = domain
        .execute(agent, call)
        .await
        .map_err(RunError::Domain)?;
    match result["success"].as_bool() {
        Some(true) => Ok(ToolResponse { operation, result }),
        Some(false) if operation == Operation::Test => Ok(ToolResponse { operation, result }),
        Some(false) => Err(RunError::Domain(format!(
            "domain operation failed: {result}"
        ))),
        None => Err(RunError::Domain(
            "domain response has no success outcome".into(),
        )),
    }
}

pub(crate) async fn run<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
    input: &TestInput,
) -> Result<(), RunError<String>> {
    input.record(execute_requested_tool(agent, endpoint, domain).await?); // global: design, once
    loop {
        // A controller sends the selected entry. The Endpoint independently
        // checks the declared resolver before that send can commit.
        match decide(input).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<Passed>(&()).await?;
                endpoint.send::<ClosedFirst>(&()).await?;
                endpoint.send::<ClosedSecond>(&()).await?;
                endpoint.send::<Finished>(&()).await?;
                return Ok(());
            }
            DecisionArm::Right => {
                endpoint.send::<Attempt>(&()).await?;
            }
        }
        execute_requested_tool(agent, endpoint, domain).await?; // global: implement
        endpoint.send::<AcquireFirst>(&()).await?;
        endpoint.recv::<GrantedFirst>().await?;
        endpoint.send::<AcquireSecond>(&()).await?;
        endpoint.recv::<GrantedSecond>().await?;
        execute_requested_tool(agent, endpoint, domain).await?; // global: commit with both modules
        // global: the two independent returns are in parallel.
        endpoint.send::<ReturnSecond>(&()).await?;
        endpoint.send::<ReturnFirst>(&()).await?;
        endpoint.recv::<ReturnedSecond>().await?;
        endpoint.recv::<ReturnedFirst>().await?;
        input.record(execute_requested_tool(agent, endpoint, domain).await?); // global: test after release
    }
}

/// 1E/1M tests are terminal. Neither task repeats implementation after a failure.
pub(crate) async fn terminal<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<String>> {
    execute_requested_tool(agent, endpoint, domain).await?; // design
    execute_requested_tool(agent, endpoint, domain).await?; // implement
    endpoint.send::<AcquireFirst>(&()).await?;
    endpoint.recv::<GrantedFirst>().await?;
    endpoint.send::<AcquireSecond>(&()).await?;
    endpoint.recv::<GrantedSecond>().await?;
    execute_requested_tool(agent, endpoint, domain).await?; // commit
    endpoint.send::<ReturnSecond>(&()).await?;
    endpoint.send::<ReturnFirst>(&()).await?;
    endpoint.recv::<ReturnedSecond>().await?;
    endpoint.recv::<ReturnedFirst>().await?;
    execute_requested_tool(agent, endpoint, domain).await?; // terminal test result
    endpoint.send::<Finished>(&()).await?;
    Ok(())
}
