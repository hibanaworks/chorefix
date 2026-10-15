//! service/code_review/user_authentication: actual review outcomes drive rejected revision cycles.
use super::super::global::*;
use crate::shell::{
    ShellError, ShellResult,
    imp::service::{Domain, File, Validation},
};
use crate::tasks::review::imp::ReviewInput;
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;

pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEW_A>,
    domain: &D,
    arguments: &Validation,
    review_result: &ReviewInput,
) -> ShellResult<()> {
    let mut file = File::new(b"service/code_review/user_authentication", SERVICE_FD);
    loop {
        endpoint.send::<BeginReviewVisit>(&()).await?;
        endpoint.send::<ReviewRequestReady>(&()).await?;
        let request = endpoint.recv::<Open>().await?;
        endpoint.send::<ReviewRequested>(&()).await?;
        endpoint.recv::<Ready>().await?;
        endpoint.send::<Opened>(&file.open(request)).await?;
        loop {
            endpoint.send::<ReviewRequest>(&()).await?;
            endpoint.send::<ReviewRequestReady>(&()).await?;
            let branch = endpoint
                .offer()
                .await
                .map_err(|e| ShellError::message(format!("reviewer_a offer: {e:?}")))?;
            match branch.label() {
                3 => {
                    let request = branch.recv::<Open>().await?;
                    endpoint.send::<Opened>(&file.open(request)).await?;
                }
                5 => {
                    let request = branch.recv::<Write>().await?;
                    endpoint.send::<Written>(&file.write(request)).await?;
                }
                7 => {
                    let request = branch.recv::<Commit>().await?;
                    arguments.record(file.arguments(request.0, "pr_id"));
                    let decision = endpoint
                        .offer()
                        .await
                        .map_err(|e| ShellError::message(format!("reviewer_a offer: {e:?}")))?;
                    match decision.label() {
                        12 => {
                            decision.recv::<Validate>().await?;
                            endpoint.send::<Admitted>(&()).await?;
                            break;
                        }
                        9 => {
                            let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments","error":arguments.rejection()?}),
                        )
                        .map_err(|e| ShellError::message(e.to_string()))?;
                            decision.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                        }
                        _ => {
                            return Err(ShellError::message("unexpected review argument decision"));
                        }
                    }
                }
                _ => return Err(ShellError::message("unexpected review request")),
            }
        }
        endpoint.send::<Review>(&()).await?;
        let result = domain
            .execute("REVIEWER", "code_review", arguments.take()?)
            .await
            .map_err(ShellError::message)?;
        let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
        review_result.record(result.clone());
        let decision = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("reviewer_a offer: {e:?}")))?;
        match decision.label() {
            180 => {
                decision.send::<ApproveRepoA>(&()).await?;
                endpoint.recv::<DecisionRecordedA>().await?;
                endpoint.send::<Approved<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<Result<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ServiceDone>(&()).await?;
                break;
            }
            181 => {
                decision.send::<ReworkRepoA>(&()).await?;
                endpoint.recv::<DecisionRecordedA>().await?;
                endpoint.send::<Revise<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<Result<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ServiceDone>(&()).await?;
            }
            _ => return Err(ShellError::message("unexpected review decision")),
        }
    }
    let status = loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<ReviewRequestReady>(&()).await?;
        let branch = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("reviewer_a offer: {e:?}")))?;
        match branch.label() {
            3 => {
                branch.recv::<Open>().await?;
                endpoint
                    .send::<Opened>(&protocol::PathOpenedRet(protocol::PathOpened::new(0, 8)))
                    .await?;
            }
            5 => {
                let r = branch.recv::<Write>().await?;
                endpoint
                    .send::<Written>(&protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(r.0.fd(), 0, 8),
                    ))
                    .await?;
            }
            7 => {
                let r = branch.recv::<Commit>().await?;
                endpoint
                    .send::<Unavailable>(&protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(r.0.fd(), 8),
                    ))
                    .await?;
            }
            241 => {
                let status = branch.recv::<Exit>().await?;
                break status;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected closed review service request: {label}"
                )));
            }
        }
    };
    endpoint.send::<ReviewerClosed>(&()).await?;
    endpoint.recv::<Merged>().await?;
    endpoint.send::<ReviewExitAck>(&()).await?;
    endpoint.recv::<BothMerged>().await?;
    if status == 0 {
        endpoint.send::<RepoExit>(&()).await?;
    } else {
        endpoint.send::<RepoFailed>(&status).await?;
    }
    Ok(())
}
