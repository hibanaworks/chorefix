use super::super::global::*;
use crate::shell::{
    ShellError, ShellResult,
    imp::service::{Domain, File, Validation},
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;

pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, DEPLOYER>,
    domain: &D,
    deploy: &Validation,
) -> ShellResult<()> {
    endpoint.recv::<TestsPassed>().await?;
    endpoint.send::<RequestReady>(&()).await?;
    let request = endpoint.recv::<Open>().await?;
    endpoint.send::<Acquire>(&()).await?;
    endpoint.recv::<Granted>().await?;
    let mut file = File::new(b"service/deploy_application", SERVICE_FD);
    endpoint.send::<Opened>(&file.open(request)).await?;
    loop {
        endpoint.send::<ServeRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
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
                deploy.record(file.arguments(request.0, "target"));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    12 => {
                        decision.recv::<Materialize>().await?;
                        let result = domain
                            .execute("DEPLOYER", "deploy_application", deploy.take()?)
                            .await
                            .map_err(ShellError::message)?;
                        let bytes = serde_json::to_vec(&result)
                            .map_err(|e| ShellError::message(e.to_string()))?;
                        endpoint.send::<Committed<'_>>(&bytes.as_slice()).await?;
                        if result["success"] != true {
                            return Err(ShellError::message(format!(
                                "domain operation failed: {result}"
                            )));
                        }
                        break;
                    }
                    9 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":deploy.rejection()?}),
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
    endpoint.send::<Return>(&()).await?;
    endpoint.recv::<Returned>().await?;
    endpoint.send::<ServiceDone>(&()).await?;
    endpoint.send::<DeployedToDeveloper>(&()).await?;
    endpoint.send::<DeployedToTester>(&()).await?;

    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
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
                let status = branch.recv::<ServiceExit>().await?;
                if status != 0 {
                    return Err(ShellError::message("guest failed"));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected closed-service request")),
        }
    }
}
