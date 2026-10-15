use super::super::{
    global::*,
    imp::{Domain, File, Validation},
};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use serde_json::json;

pub async fn build_server<D: Domain>(
    endpoint: &mut Endpoint<'_, BUILD_SERVER>,
    domain: &D,
    compile: &Validation,
    test: &Validation,
) -> ShellResult<()> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    let mut file = File::new(b"service/compile_and_test");
    loop {
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
                compile.record(file.arguments(request.0, "component"));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    12 => {
                        decision.recv::<Materialize>().await?;
                        let result = domain
                            .execute("DEVELOPER", "compile_and_test", compile.take()?)
                            .await
                            .map_err(ShellError::message)?;
                        let bytes = serde_json::to_vec(&result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        endpoint.send::<Committed<'_>>(&bytes.as_slice()).await?;
                        break;
                    }
                    9 => {
                        let error = compile.rejection()?;
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":error.to_string()}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected close decision")),
                }
            }
            _ => return Err(ShellError::message("unexpected service operation")),
        }
    }
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    let mut file = File::new(b"service/run_integration_tests");
    loop {
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
                test.record(file.arguments(request.0, "suite"));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    12 => {
                        decision.recv::<Materialize>().await?;
                        let result = domain
                            .execute("TESTER", "run_integration_tests", test.take()?)
                            .await
                            .map_err(ShellError::message)?;
                        let bytes = serde_json::to_vec(&result)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        endpoint.send::<Committed<'_>>(&bytes.as_slice()).await?;
                        break;
                    }
                    9 => {
                        let error = test.rejection()?;
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":error.to_string()}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected close decision")),
                }
            }
            _ => return Err(ShellError::message("unexpected service operation")),
        }
    }
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}

pub async fn deploy_env<D: Domain>(
    endpoint: &mut Endpoint<'_, DEPLOY_ENV>,
    domain: &D,
    deploy: &Validation,
) -> ShellResult<()> {
    endpoint.recv::<Acquire>().await?;
    endpoint.send::<Granted>(&()).await?;
    let mut file = File::new(b"service/deploy_application");
    loop {
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
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        endpoint.send::<Committed<'_>>(&bytes.as_slice()).await?;
                        break;
                    }
                    9 => {
                        let error = deploy.rejection()?;
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":error.to_string()}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<Rejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected close decision")),
                }
            }
            _ => return Err(ShellError::message("unexpected service operation")),
        }
    }
    endpoint.recv::<Return>().await?;
    endpoint.send::<Returned>(&()).await?;
    Ok(())
}
