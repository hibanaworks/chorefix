//! Think and eat are two explicit protocol lifetimes. Endpoints own progress.
use super::super::global::*;
use crate::{
    shell::{
        ShellError, ShellResult,
        imp::service::{Domain, File, Validation},
    },
    tasks::philosophers::global::ForkMessages,
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;
pub(crate) async fn run<const ROLE: u8, F: ForkMessages, S: ForkMessages, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
    validation: &Validation,
) -> ShellResult<()> {
    // think: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/think", SERVICE_FD);
    loop {
        endpoint.send::<ThinkRequest>(&()).await?;
        endpoint.send::<ThinkRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                let request = branch.recv::<ThinkOpen>().await?;
                endpoint.send::<ThinkOpened>(&file.open(request)).await?;
            }
            102 => {
                let request = branch.recv::<ThinkWrite>().await?;
                endpoint.send::<ThinkWritten>(&file.write(request)).await?;
            }
            104 => {
                let request = branch.recv::<ThinkCommit>().await?;
                validation.record(
                    file.arguments(request.0, "philosopher")
                        .and_then(|arguments| {
                            if arguments["philosopher"] == agent {
                                Ok(arguments)
                            } else {
                                Err(ShellError::message(
                                    "arguments name a different task object",
                                ))
                            }
                        }),
                );
                let decision = endpoint.offer().await?;
                match decision.label() {
                    111 => {
                        decision.recv::<ThinkValidate>().await?;
                        endpoint.send::<ThinkAdmitted>(&()).await?;
                        break;
                    }
                    106 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<ThinkRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected think request")),
        }
    }
    endpoint.send::<ThinkEffect>(&()).await?;
    let result = domain
        .execute(agent, "think", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain think failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<ThinkCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<ThinkServiceDone>(&()).await?;

    // eat: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/eat", SERVICE_FD);
    loop {
        endpoint.send::<EatRequest>(&()).await?;
        endpoint.send::<EatRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            150 => {
                let request = branch.recv::<EatOpen>().await?;
                endpoint.send::<EatOpened>(&file.open(request)).await?;
            }
            152 => {
                let request = branch.recv::<EatWrite>().await?;
                endpoint.send::<EatWritten>(&file.write(request)).await?;
            }
            154 => {
                let request = branch.recv::<EatCommit>().await?;
                validation.record(
                    file.arguments(request.0, "philosopher")
                        .and_then(|arguments| {
                            if arguments["philosopher"] == agent {
                                Ok(arguments)
                            } else {
                                Err(ShellError::message(
                                    "arguments name a different philosopher",
                                ))
                            }
                        }),
                );
                let decision = endpoint.offer().await?;
                match decision.label() {
                    161 => {
                        decision.recv::<EatValidate>().await?;
                        endpoint.send::<EatAdmitted>(&()).await?;
                        break;
                    }
                    156 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<EatRejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected eat request")),
        }
    }
    endpoint.recv::<F::Available>().await?;
    endpoint.recv::<S::Available>().await?;
    endpoint.send::<F::Acquire>(&()).await?;
    endpoint.send::<S::Acquire>(&()).await?;
    endpoint.recv::<F::Granted>().await?;
    endpoint.recv::<S::Granted>().await?;
    endpoint.send::<EatEffect>(&()).await?;
    let result = domain
        .execute(agent, "eat", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!("domain eat failed: {result}")));
    }
    endpoint.send::<S::Return>(&()).await?;
    endpoint.send::<F::Return>(&()).await?;
    endpoint.recv::<S::Returned>().await?;
    endpoint.recv::<F::Returned>().await?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<EatCompleted<'_>>(&bytes.as_slice()).await?;
    endpoint.send::<EatServiceDone>(&()).await?;

    // Completion is a projected lifetime. Subsequent effects are unavailable.
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<ClosedRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            200 => {
                branch.recv::<ClosedOpen>().await?;
                endpoint
                    .send::<ClosedOpened>(&protocol::PathOpenedRet(protocol::PathOpened::new(0, 8)))
                    .await?;
            }
            202 => {
                let request = branch.recv::<ClosedWrite>().await?;
                endpoint
                    .send::<ClosedWritten>(&protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(request.0.fd(), 0, 8),
                    ))
                    .await?;
            }
            204 => {
                let request = branch.recv::<ClosedCommit>().await?;
                endpoint
                    .send::<ClosedUnavailable>(&protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(request.0.fd(), 8),
                    ))
                    .await?;
            }
            241 => {
                let status = branch.recv::<ServiceExit>().await?;
                if status != 0 {
                    return Err(ShellError::message("philosopher guest failed"));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected completed-service request")),
        }
    }
}
