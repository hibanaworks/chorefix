//! service/code_review/notification_feature: actual review outcomes drive rejected revision cycles.
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
    endpoint: &mut Endpoint<'_, REVIEW_B>,
    domain: &D,
    arguments: &Validation,
    review_result: &ReviewInput,
) -> ShellResult<()> {
    let mut file = File::new(b"service/code_review/notification_feature", REVIEW_B_FD);
    loop {
        endpoint.send::<BeginReviewVisit>(&()).await?;
        endpoint.send::<ReviewRequestReadyB>(&()).await?;
        let request = endpoint.recv::<OpenB>().await?;
        endpoint.send::<ReviewRequestedB>(&()).await?;
        endpoint.recv::<Ready>().await?;
        endpoint.send::<OpenedB>(&file.open(request)).await?;
        loop {
            endpoint.send::<ReviewRequest>(&()).await?;
            endpoint.send::<ReviewRequestReadyB>(&()).await?;
            let branch = endpoint
                .offer()
                .await
                .map_err(|e| ShellError::message(format!("reviewer_b offer: {e:?}")))?;
            match branch.label() {
                43 => {
                    let request = branch.recv::<OpenB>().await?;
                    endpoint.send::<OpenedB>(&file.open(request)).await?;
                }
                45 => {
                    let request = branch.recv::<WriteB>().await?;
                    endpoint.send::<WrittenB>(&file.write(request)).await?;
                }
                47 => {
                    let request = branch.recv::<CommitB>().await?;
                    arguments.record(file.arguments(request.0, "pr_id"));
                    let decision = endpoint
                        .offer()
                        .await
                        .map_err(|e| ShellError::message(format!("reviewer_b offer: {e:?}")))?;
                    match decision.label() {
                        52 => {
                            decision.recv::<ValidateB>().await?;
                            endpoint.send::<AdmittedB>(&()).await?;
                            break;
                        }
                        49 => {
                            let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments","error":arguments.rejection()?}),
                        )
                        .map_err(|e| ShellError::message(e.to_string()))?;
                            decision.send::<RejectedB<'_>>(&bytes.as_slice()).await?;
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
            .map_err(|e| ShellError::message(format!("reviewer_b offer: {e:?}")))?;
        match decision.label() {
            182 => {
                decision.send::<ApproveRepoB>(&()).await?;
                endpoint.recv::<DecisionRecordedB>().await?;
                endpoint.send::<Approved<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ResultB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ServiceDoneB>(&()).await?;
                break;
            }
            183 => {
                decision.send::<ReworkRepoB>(&()).await?;
                endpoint.recv::<DecisionRecordedB>().await?;
                endpoint.send::<Revise<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ResultB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ServiceDoneB>(&()).await?;
            }
            _ => return Err(ShellError::message("unexpected review decision")),
        }
    }
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<ReviewRequestReadyB>(&()).await?;
        let branch = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("reviewer_b offer: {e:?}")))?;
        match branch.label() {
            43 => {
                branch.recv::<OpenB>().await?;
                endpoint
                    .send::<OpenedB>(&protocol::PathOpenedRet(protocol::PathOpened::new(0, 8)))
                    .await?;
            }
            45 => {
                let r = branch.recv::<WriteB>().await?;
                endpoint
                    .send::<WrittenB>(&protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(r.0.fd(), 0, 8),
                    ))
                    .await?;
            }
            47 => {
                let r = branch.recv::<CommitB>().await?;
                endpoint
                    .send::<UnavailableB>(&protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(r.0.fd(), 8),
                    ))
                    .await?;
            }
            242 => {
                branch.recv::<ExitB>().await?;
                break;
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected closed review service request: {label}"
                )));
            }
        }
    }
    endpoint.send::<ReviewerClosed>(&()).await?;
    endpoint.recv::<Merged>().await?;
    endpoint.send::<ReviewExitAckB>(&()).await?;
    endpoint.send::<BothMerged>(&()).await?;
    Ok(())
}
