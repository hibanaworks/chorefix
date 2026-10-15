//! The feature owner's service requests and review continuation are explicit.
use super::super::global::*;
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};

pub async fn run(endpoint: &mut Endpoint<'_, B>) -> ShellResult<()> {
    loop {
        endpoint.send::<BeginFeature>(&()).await?;
        let mut result: Value = loop {
            endpoint.send::<ImplementRequest>(&()).await?;
            endpoint.send::<RequestReady>(&()).await?;
            let branch = endpoint
                .offer()
                .await
                .map_err(|e| ShellError::message(format!("developer_b offer: {e:?}")))?;
            match branch.label() {
                3 => {
                    let request = branch.recv::<Open>().await?;
                    endpoint.send::<OpenImplementB>(&request).await?;
                    let response = endpoint.recv::<OpenedImplementB>().await?;
                    endpoint.send::<Opened>(&response).await?;
                }
                5 => {
                    let request = branch.recv::<Write>().await?;
                    endpoint.send::<WriteImplementB>(&request).await?;
                    let response = endpoint.recv::<WrittenImplementB>().await?;
                    endpoint.send::<Written>(&response).await?;
                }
                7 => {
                    let request = branch.recv::<Commit>().await?;
                    endpoint.send::<CommitImplementB>(&request).await?;
                    let decision = endpoint
                        .offer()
                        .await
                        .map_err(|e| ShellError::message(format!("developer_b offer: {e:?}")))?;
                    match decision.label() {
                        156 => {
                            let value = serde_json::from_slice(
                                decision.recv::<CompletedImplementB<'_>>().await?,
                            )
                            .map_err(|e| ShellError::message(e.to_string()))?;
                            endpoint.send::<Admitted>(&()).await?;
                            break value;
                        }
                        157 => {
                            let bytes = decision.recv::<RejectedImplementB<'_>>().await?.to_vec();
                            endpoint.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                        }
                        _ => {
                            return Err(ShellError::message(
                                "unexpected REPO implementation response",
                            ));
                        }
                    }
                }
                _ => return Err(ShellError::message("unexpected implementation request")),
            }
        };
        endpoint.recv::<ReviewRequestedB>().await?;
        endpoint.send::<Ready>(&()).await?;
        let feedback = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("developer_b offer: {e:?}")))?;
        match feedback.label() {
            30 => {
                let value: Value = serde_json::from_slice(feedback.recv::<Approved<'_>>().await?)
                    .map_err(|e| ShellError::message(e.to_string()))?;
                result["coordination"] = json!({"code_review":value});
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                endpoint.send::<Result<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ServiceDone>(&()).await?;
                break;
            }
            31 => {
                let value: Value = serde_json::from_slice(feedback.recv::<Revise<'_>>().await?)
                    .map_err(|e| ShellError::message(e.to_string()))?;
                result["coordination"] = json!({"code_review":value});
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                endpoint.send::<Result<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ServiceDone>(&()).await?;
            }
            _ => return Err(ShellError::message("unexpected review feedback")),
        }
    }
    let bytes = loop {
        endpoint.send::<MergeRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("developer_b offer: {e:?}")))?;
        match branch.label() {
            3 => {
                let request = branch.recv::<Open>().await?;
                endpoint.send::<OpenMergeB>(&request).await?;
                let response = endpoint.recv::<OpenedMergeB>().await?;
                endpoint.send::<Opened>(&response).await?;
            }
            5 => {
                let request = branch.recv::<Write>().await?;
                endpoint.send::<WriteMergeB>(&request).await?;
                let response = endpoint.recv::<WrittenMergeB>().await?;
                endpoint.send::<Written>(&response).await?;
            }
            7 => {
                let request = branch.recv::<Commit>().await?;
                endpoint.send::<CommitMergeB>(&request).await?;
                let decision = endpoint
                    .offer()
                    .await
                    .map_err(|e| ShellError::message(format!("developer_b offer: {e:?}")))?;
                match decision.label() {
                    176 => {
                        let bytes = decision.recv::<CompletedMergeB<'_>>().await?.to_vec();
                        endpoint.send::<Admitted>(&()).await?;
                        break bytes;
                    }
                    177 => {
                        let bytes = decision.recv::<RejectedMergeB<'_>>().await?.to_vec();
                        endpoint.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected REPO merge response")),
                }
            }
            _ => return Err(ShellError::message("unexpected merge request")),
        }
    };
    endpoint.send::<Result<'_>>(&bytes.as_slice()).await?;
    endpoint.send::<ServiceDone>(&()).await?;
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("developer_b offer: {e:?}")))?;
        match branch.label() {
            3 => {
                branch.recv::<Open>().await?;
                endpoint
                    .send::<Opened>(&protocol::PathOpenedRet(protocol::PathOpened::new(0, 8)))
                    .await?;
            }
            5 => {
                let request = branch.recv::<Write>().await?;
                endpoint
                    .send::<Written>(&protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(request.0.fd(), 0, 8),
                    ))
                    .await?;
            }
            7 => {
                let request = branch.recv::<Commit>().await?;
                endpoint
                    .send::<Unavailable>(&protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(request.0.fd(), 8),
                    ))
                    .await?;
            }
            241 => {
                let status = branch.recv::<Exit>().await?;
                if status != 0 {
                    return Err(ShellError::message("developer guest failed"));
                }
                endpoint.recv::<ReviewerClosed>().await?;
                endpoint.send::<Merged>(&()).await?;
                return Ok(());
            }
            _ => {
                return Err(ShellError::message(
                    "unexpected closed developer service request",
                ));
            }
        }
    }
}
