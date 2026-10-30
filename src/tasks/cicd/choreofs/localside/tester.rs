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
    test: &Validation,
) -> ShellResult<()> {
    endpoint.recv::<BuildComplete>().await?;
    endpoint.send::<RequestReady>(&()).await?;
    let request = endpoint.recv::<Open>().await?;
    let mut file = File::new(b"service/run_integration_tests", SERVICE_FD);
    let response = file.open(request);
    endpoint.send::<Opened>(&response).await?;
    loop {
        endpoint.send::<ServeRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            3 => {
                let request = branch.recv::<Open>().await?;
                let response = file.open(request);
                endpoint.send::<Opened>(&response).await?;
            }
            5 => {
                let request = branch.recv::<Write>().await?;
                let response = file.write(request);
                endpoint.send::<Written>(&response).await?;
            }
            7 => {
                let request = branch.recv::<Commit>().await?;
                test.record(file.arguments(request, "suite"));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    12 => {
                        decision.recv::<Validate>().await?;
                        let arguments = serde_json::to_vec(&test.take()?)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        endpoint.send::<Admitted<'_>>(&arguments.as_slice()).await?;
                        break;
                    }
                    9 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":test.rejection()?}),
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
    let arguments = endpoint.recv::<TestRequest>().await?.into_value();
    let result = domain
        .execute("TESTER", "run_integration_tests", arguments)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint.send::<TestResult<'_>>(&bytes.as_slice()).await?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain operation failed: {result}"
        )));
    }
    endpoint.send::<TestsPassed>(&()).await?;
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<RequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            3 => {
                branch.recv::<Open>().await?;
                let response = protocol::PathOpenedRet(protocol::PathOpened::new(0, 8));
                endpoint.send::<Opened>(&response).await?;
            }
            5 => {
                let request = branch.recv::<Write>().await?;
                let response = protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(
                    hibana_wasip1_runtime::choreofs::ChoreoFsRequest::request(&request)
                        .0
                        .fd(),
                    0,
                    8,
                ));
                endpoint.send::<Written>(&response).await?;
            }
            7 => {
                let request = branch.recv::<Commit>().await?;
                let response = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                    hibana_wasip1_runtime::choreofs::ChoreoFsRequest::request(&request)
                        .0
                        .fd(),
                    8,
                ));
                endpoint.send::<Unavailable>(&response).await?;
            }
            241 => {
                let status = branch.recv::<ServiceExit>().await?;
                if status != 0 {
                    return Err(ShellError::message("guest failed"));
                }
                break;
            }
            _ => return Err(ShellError::message("unexpected closed-service request")),
        }
    }
    endpoint.recv::<DeployedToTester>().await?;
    Ok(())
}
