use super::super::{
    global::*,
    imp::{Domain, Operation, Request, ReviewInput, decide, succeeded},
};
use crate::RunError;
use hibana::{Endpoint, runtime::resolver::DecisionArm};

#[hibana_runtime::scope]
pub(crate) async fn run<D: Domain>(
    endpoint: &mut Endpoint<EDITOR>,
    domain: &D,
    input_a: &ReviewInput,
    input_b: &ReviewInput,
) -> Result<(), RunError<String>> {
    let call = domain.request("EDITOR").await.map_err(RunError::Domain)?;
    endpoint.recv::<author_a::Ready>().await?;
    endpoint.send::<author_a::BeginReview>(&()).await?;
    match call.operation() {
        Operation::ReviewA => {
            endpoint.send::<ReviewA>(&()).await?;
            input_a.record(
                domain
                    .execute("EDITOR", call)
                    .await
                    .map_err(RunError::Domain)?,
            );
        }
        Operation::ReviewB => {
            endpoint.send::<ReviewB>(&()).await?;
            return Err(RunError::Domain(
                "unexpected admission of section B review".into(),
            ));
        }
        Operation::Combine => {
            endpoint.send::<CombineRequest>(&()).await?;
            return Err(RunError::Domain(
                "unexpected admission of early combine".into(),
            ));
        }
        _ => {
            return Err(RunError::Domain(
                "tool does not belong to the editor".into(),
            ));
        }
    }
    endpoint.recv::<author_b::ReadyForFeedback>().await?;
    endpoint.send::<author_a::BeginDocumentReview>(&()).await?;
    loop {
        match decide(input_a).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<author_a::Accepted>(&()).await?;
                endpoint.send::<author_b::BeginReview>(&()).await?;
                endpoint.send::<author_b::BeginDocumentReview>(&()).await?;
                break;
            }
            DecisionArm::Right => {
                endpoint.send::<author_b::WaitForARevision>(&()).await?;
                endpoint.send::<author_a::NeedsRevision>(&()).await?;
                endpoint.recv::<author_a::RevisionReady>().await?;
                endpoint
                    .send::<author_a::NeedsDocumentRevision>(&())
                    .await?;
                endpoint.recv::<author_a::Resubmitted>().await?;
            }
        }
        let call = domain.request("EDITOR").await.map_err(RunError::Domain)?;
        match call.operation() {
            Operation::ReviewA => {
                endpoint.send::<ReviewA>(&()).await?;
                input_a.record(
                    domain
                        .execute("EDITOR", call)
                        .await
                        .map_err(RunError::Domain)?,
                );
            }
            Operation::ReviewB => {
                endpoint.send::<ReviewB>(&()).await?;
                return Err(RunError::Domain(
                    "unexpected admission of section B review".into(),
                ));
            }
            Operation::Combine => {
                endpoint.send::<CombineRequest>(&()).await?;
                return Err(RunError::Domain(
                    "unexpected admission of early combine".into(),
                ));
            }
            _ => {
                return Err(RunError::Domain(
                    "tool does not belong to the editor".into(),
                ));
            }
        }
    }
    endpoint.recv::<author_b::Ready>().await?;
    loop {
        let call = domain.request("EDITOR").await.map_err(RunError::Domain)?;
        match call.operation() {
            Operation::ReviewB => {
                endpoint.send::<ReviewB>(&()).await?;
                input_b.record(
                    domain
                        .execute("EDITOR", call)
                        .await
                        .map_err(RunError::Domain)?,
                );
                match decide(input_b).map_err(RunError::Resolver)? {
                    DecisionArm::Left => {
                        endpoint.send::<author_b::Accepted>(&()).await?;
                        endpoint.send::<BeginCombine>(&()).await?;
                        break;
                    }
                    DecisionArm::Right => {
                        endpoint.send::<author_b::NeedsRevision>(&()).await?;
                        endpoint.recv::<author_b::RevisionReady>().await?;
                        endpoint
                            .send::<author_b::NeedsDocumentRevision>(&())
                            .await?;
                        endpoint.recv::<author_b::Resubmitted>().await?;
                    }
                }
            }
            Operation::ReviewA => {
                endpoint.send::<ReviewA>(&()).await?;
                return Err(RunError::Domain(
                    "unexpected admission of a completed section review".into(),
                ));
            }
            Operation::Combine => {
                endpoint.send::<CombineRequest>(&()).await?;
                return Err(RunError::Domain(
                    "unexpected admission of early combine".into(),
                ));
            }
            _ => {
                return Err(RunError::Domain(
                    "tool does not belong to the editor".into(),
                ));
            }
        }
    }
    let call = domain.request("EDITOR").await.map_err(RunError::Domain)?;
    match call.operation() {
        Operation::Combine => {
            endpoint.send::<CombineRequest>(&()).await?;
            endpoint.recv::<CombineGranted>().await?;
            endpoint.send::<Combine>(&()).await?;
            succeeded(
                &domain
                    .execute("EDITOR", call)
                    .await
                    .map_err(RunError::Domain)?,
            )?;
            endpoint.send::<CombineReturn>(&()).await?;
            endpoint.recv::<CombineReturned>().await?;
            Ok(())
        }
        Operation::ReviewA => {
            endpoint.send::<ReviewA>(&()).await?;
            Err(RunError::Domain(
                "unexpected repeated review admission".into(),
            ))
        }
        Operation::ReviewB => {
            endpoint.send::<ReviewB>(&()).await?;
            Err(RunError::Domain(
                "unexpected repeated review admission".into(),
            ))
        }
        _ => Err(RunError::Domain(
            "tool does not belong to the editor".into(),
        )),
    }
}
