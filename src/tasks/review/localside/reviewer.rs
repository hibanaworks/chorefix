use super::super::{
    global::*,
    imp::{Domain, Operation, Request, ReviewInput, decide},
};
use crate::RunError;
use hibana::{Endpoint, runtime::resolver::DecisionArm};

pub(crate) async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEWER>,
    domain: &D,
    a: &ReviewInput,
    b: &ReviewInput,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain.request("REVIEWER").await.map_err(RunError::Domain)?;
        match call.operation() {
            Operation::ReviewA => {
                endpoint.recv::<ReadyA>().await?;
                endpoint.send::<ReviewA>(&()).await?;
                let (receipt, result) = domain
                    .execute("REVIEWER", call)
                    .await
                    .map_err(RunError::Domain)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| RunError::Domain(e.to_string()))?;
                a.record(result.clone());
                match decide(a).map_err(RunError::Resolver)? {
                    DecisionArm::Left => {
                        endpoint.send::<ApprovedA<'_>>(&bytes.as_slice()).await?;
                        endpoint.send::<ApprovedPortA>(&()).await?;
                    }
                    DecisionArm::Right => {
                        endpoint.send::<ReviseA<'_>>(&bytes.as_slice()).await?;
                        endpoint.send::<RevisePortA>(&()).await?;
                    }
                }
                domain
                    .reply(receipt, result)
                    .await
                    .map_err(RunError::Domain)?;
            }
            Operation::ReviewB => {
                endpoint.recv::<ReadyB>().await?;
                endpoint.send::<ReviewB>(&()).await?;
                let (receipt, result) = domain
                    .execute("REVIEWER", call)
                    .await
                    .map_err(RunError::Domain)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| RunError::Domain(e.to_string()))?;
                b.record(result.clone());
                match decide(b).map_err(RunError::Resolver)? {
                    DecisionArm::Left => {
                        endpoint.send::<ApprovedB<'_>>(&bytes.as_slice()).await?;
                        endpoint.send::<ApprovedPortB>(&()).await?;
                    }
                    DecisionArm::Right => {
                        endpoint.send::<ReviseB<'_>>(&bytes.as_slice()).await?;
                        endpoint.send::<RevisePortB>(&()).await?;
                    }
                }
                domain
                    .reply(receipt, result)
                    .await
                    .map_err(RunError::Domain)?;
            }
            Operation::ModelFinished => {
                endpoint.recv::<MergedA>().await?;
                endpoint.recv::<MergedB>().await?;
                return Ok(());
            }
            Operation::Implement => endpoint.send::<Implement>(&()).await?,
            Operation::Merge => endpoint.send::<Merge>(&()).await?,
        }
    }
}
