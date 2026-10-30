use super::super::global::*;
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

pub(crate) async fn run<const ROLE: u8, const LEFT: u8, const RIGHT: u8>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &impl Domain,
) -> ShellResult<()> {
    let arguments = endpoint.recv::<Think>().await?;
    let result = domain
        .execute(agent, "think", arguments.into_value())
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain think failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    let response = bytes.as_slice();
    endpoint.send::<Thought<'_>>(&response).await?;
    let (left, right) = {
        let [mut left, mut right] = endpoint.par()?;
        tokio::try_join!(left.recv::<Eat<LEFT>>(), right.recv::<Eat<RIGHT>>())?
    };
    if left != right {
        return Err(ShellError::message("fork requests name different meals"));
    }
    let result = domain
        .execute(agent, "eat", left.into_value())
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!("domain eat failed: {result}")));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    {
        let [mut left, mut right] = endpoint.par()?;
        let payload = bytes.as_slice();
        tokio::try_join!(
            left.send::<Meal<'_, LEFT>>(&payload),
            right.send::<Meal<'_, RIGHT>>(&payload)
        )?;
    }
    let status = endpoint.recv::<ServiceExit>().await?;
    if status != 0 {
        return Err(ShellError::message(format!(
            "philosopher guest exited with {status}"
        )));
    }
    Ok(())
}
