use super::super::{
    global::*,
    imp::{Domain, Operation, Request},
};
use crate::RunError;
use hibana::{Endpoint, g};
use serde_json::Value;

pub(crate) async fn run<
    const ROLE: u8,
    const READY: u8,
    const MERGED: u8,
    const APPROVED: u8,
    const REVISE: u8,
    D: Domain,
>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        match call.operation() {
            Operation::Implement => {
                endpoint.send::<Acquire>(&()).await?;
                endpoint.recv::<Granted>().await?;
                endpoint.send::<Implement>(&()).await?;
                let (receipt, mut result) = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                if result["success"] != true {
                    return Err(RunError::Domain(format!("implementation failed: {result}")));
                }
                endpoint.send::<Return>(&()).await?;
                endpoint.recv::<Returned>().await?;
                endpoint.send::<g::Msg<READY, ()>>(&()).await?;
                let branch = endpoint.offer().await?;
                let feedback: Value = match branch.label() {
                    label if label == APPROVED => {
                        serde_json::from_slice(branch.recv::<g::Msg<APPROVED, &[u8]>>().await?)
                            .map_err(|e| RunError::Domain(e.to_string()))?
                    }
                    label if label == REVISE => {
                        serde_json::from_slice(branch.recv::<g::Msg<REVISE, &[u8]>>().await?)
                            .map_err(|e| RunError::Domain(e.to_string()))?
                    }
                    _ => unreachable!("projected review feedback"),
                };
                result["coordination"] = serde_json::json!({"code_review":feedback});
                domain
                    .reply(receipt, result)
                    .await
                    .map_err(RunError::Domain)?;
            }
            Operation::Merge => {
                endpoint.send::<Acquire>(&()).await?;
                endpoint.recv::<Granted>().await?;
                endpoint.send::<Merge>(&()).await?;
                let (receipt, result) = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                if result["success"] != true {
                    return Err(RunError::Domain(format!("merge failed: {result}")));
                }
                endpoint.send::<Return>(&()).await?;
                endpoint.recv::<Returned>().await?;
                endpoint.send::<g::Msg<MERGED, ()>>(&()).await?;
                domain
                    .reply(receipt, result)
                    .await
                    .map_err(RunError::Domain)?;
                return Ok(());
            }
            Operation::ReviewA => endpoint.send::<ReviewA>(&()).await?,
            Operation::ReviewB => endpoint.send::<ReviewB>(&()).await?,
            Operation::ModelFinished => {
                return Err(RunError::Domain(
                    "developer model direction closed before merge".into(),
                ));
            }
        }
    }
}
