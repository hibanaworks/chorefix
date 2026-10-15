use super::super::global::*;
use crate::shell::{
    ShellError, ShellResult,
    imp::service::{Domain, File, Validation},
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;
pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, TESTER>,
    domain: &D,
    validation: &Validation,
    result_input: &crate::tasks::api::imp::TestInput,
) -> ShellResult<()> {
    endpoint.recv::<BackendReady>().await?;
    let mut file = File::new(b"service/run_integration_tests", SERVICE_FD);
    loop {
        endpoint.send::<Request>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                let request = branch.recv::<Open>().await?;
                endpoint.send::<Opened>(&file.open(request)).await?;
            }
            102 => {
                let request = branch.recv::<Write>().await?;
                endpoint.send::<Written>(&file.write(request)).await?;
            }
            104 => {
                let request = branch.recv::<Commit>().await?;
                validation.record(file.arguments(request.0, "suite").and_then(|v| {
                    if v["suite"] == "backend_schema_integration" {
                        Ok(v)
                    } else {
                        Err(ShellError::message(
                            "arguments name a different task object",
                        ))
                    }
                }));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    111 => {
                        decision.recv::<Validate>().await?;
                        endpoint.send::<Admitted>(&()).await?;
                        break;
                    }
                    106 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments","error":validation.rejection()?}),
                        )
                        .map_err(|e| ShellError::message(e.to_string()))?;
                        decision.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected argument decision")),
                }
            }
            _ => return Err(ShellError::message("unexpected service request")),
        }
    }
    endpoint.send::<Acquire>(&()).await?;
    endpoint.recv::<Granted>().await?;
    endpoint.send::<Effect>(&()).await?;
    let result = domain
        .execute("TESTER", "run_integration_tests", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    endpoint.send::<Return>(&()).await?;
    endpoint.recv::<Returned>().await?;
    let bytes = serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
    endpoint.send::<ResultBytes<'_>>(&bytes.as_slice()).await?;
    result_input.record(result);
    let decision = endpoint.offer().await?;
    match decision.label() {
        50 => {
            decision.send::<PassedAdmin<'_>>(&bytes.as_slice()).await?;
            endpoint
                .send::<PassedBackend<'_>>(&bytes.as_slice())
                .await?;
        }
        51 => {
            decision
                .send::<PassedBackend<'_>>(&bytes.as_slice())
                .await?;
            endpoint.send::<PassedAdmin<'_>>(&bytes.as_slice()).await?;
        }
        60 => {
            decision.send::<FailedAdmin<'_>>(&bytes.as_slice()).await?;
            endpoint
                .send::<FailedBackend<'_>>(&bytes.as_slice())
                .await?;
        }
        61 => {
            decision
                .send::<FailedBackend<'_>>(&bytes.as_slice())
                .await?;
            endpoint.send::<FailedAdmin<'_>>(&bytes.as_slice()).await?;
        }
        _ => return Err(ShellError::message("unexpected test outcome")),
    }
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                branch.recv::<Open>().await?;
                endpoint
                    .send::<Opened>(&protocol::PathOpenedRet(protocol::PathOpened::new(0, 8)))
                    .await?;
            }
            102 => {
                let r = branch.recv::<Write>().await?;
                endpoint
                    .send::<Written>(&protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(r.0.fd(), 0, 8),
                    ))
                    .await?;
            }
            104 => {
                let r = branch.recv::<Commit>().await?;
                endpoint
                    .send::<Unavailable>(&protocol::FdClosedRet(
                        protocol::FdClosed::new_with_errno(r.0.fd(), 8),
                    ))
                    .await?;
            }
            241 => {
                let status = branch.recv::<ServiceExit>().await?;
                if status != 0 {
                    return Err(ShellError::message("guest failed"));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected completed service request")),
        }
    }
}
