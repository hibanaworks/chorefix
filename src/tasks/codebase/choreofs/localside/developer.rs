//! The four literal protocol lifetimes are visible here. No phase variable.
use super::super::global::*;
use crate::{
    shell::{
        ShellError, ShellResult,
        imp::service::{Domain, File, Validation},
    },
    tasks::codebase::choreofs::global::ModuleMessages,
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;

/// Original task input, not protocol progress or an authorization table.
pub(crate) struct Assignment {
    pub agent: &'static str,
    pub feature: &'static str,
    pub modules: [&'static str; 2],
}

pub(crate) async fn run<const ROLE: u8, F: ModuleMessages, S: ModuleMessages, D: Domain>(
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
    validation: &Validation,
    assignment: &Assignment,
) -> ShellResult<()> {
    // design_feature: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/design_feature", SERVICE_FD);
    loop {
        endpoint.send::<DesignRequest>(&()).await?;
        endpoint.send::<DesignRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                let request = branch.recv::<DesignOpen>().await?;
                endpoint.send::<DesignOpened>(&file.open(request)).await?;
            }
            102 => {
                let request = branch.recv::<DesignWrite>().await?;
                endpoint.send::<DesignWritten>(&file.write(request)).await?;
            }
            104 => {
                let request = branch.recv::<DesignCommit>().await?;
                validation.record(file.arguments(request.0, "feature_name").and_then(
                    |arguments| {
                        if arguments["feature_name"] == assignment.feature {
                            Ok(arguments)
                        } else {
                            Err(ShellError::message(
                                "arguments name a different task object",
                            ))
                        }
                    },
                ));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    111 => {
                        decision.recv::<DesignValidate>().await?;
                        endpoint.send::<DesignAdmitted>(&()).await?;
                        break;
                    }
                    106 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<DesignRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected design_feature request")),
        }
    }
    endpoint.send::<DesignEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "design_feature", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain design_feature failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<DesignCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<DesignServiceDone>(&()).await?;

    // implement_code: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/implement_code", SERVICE_FD);
    loop {
        endpoint.send::<ImplementRequest>(&()).await?;
        endpoint.send::<ImplementRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            125 => {
                let request = branch.recv::<ImplementOpen>().await?;
                endpoint
                    .send::<ImplementOpened>(&file.open(request))
                    .await?;
            }
            127 => {
                let request = branch.recv::<ImplementWrite>().await?;
                endpoint
                    .send::<ImplementWritten>(&file.write(request))
                    .await?;
            }
            129 => {
                let request = branch.recv::<ImplementCommit>().await?;
                validation.record(file.arguments(request.0, "feature_name").and_then(
                    |arguments| {
                        if arguments["feature_name"] == assignment.feature {
                            Ok(arguments)
                        } else {
                            Err(ShellError::message(
                                "arguments name a different task object",
                            ))
                        }
                    },
                ));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    136 => {
                        decision.recv::<ImplementValidate>().await?;
                        endpoint.send::<ImplementAdmitted>(&()).await?;
                        break;
                    }
                    131 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<ImplementRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected implement_code request")),
        }
    }
    endpoint.send::<ImplementEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "implement_code", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain implement_code failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<ImplementCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<ImplementServiceDone>(&()).await?;

    // commit_changes: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/commit_changes", SERVICE_FD);
    loop {
        endpoint.send::<CommitRequest>(&()).await?;
        endpoint.send::<CommitRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            150 => {
                let request = branch.recv::<CommitOpen>().await?;
                endpoint.send::<CommitOpened>(&file.open(request)).await?;
            }
            152 => {
                let request = branch.recv::<CommitWrite>().await?;
                endpoint.send::<CommitWritten>(&file.write(request)).await?;
            }
            154 => {
                let request = branch.recv::<CommitCommit>().await?;
                validation.record(file.arguments(request.0, "modules").and_then(|arguments| {
                    let actual: Vec<_> = arguments["modules"]
                        .as_str()
                        .unwrap()
                        .split(',')
                        .map(str::trim)
                        .collect();
                    if actual.len() == 2
                        && actual
                            .into_iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            == assignment.modules.into_iter().collect()
                    {
                        Ok(arguments)
                    } else {
                        Err(ShellError::message(
                            "arguments name a different task object",
                        ))
                    }
                }));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    161 => {
                        decision.recv::<CommitValidate>().await?;
                        endpoint.send::<CommitAdmitted>(&()).await?;
                        break;
                    }
                    156 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<CommitRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected commit_changes request")),
        }
    }
    endpoint.recv::<F::Available>().await?;
    endpoint.recv::<S::Available>().await?;
    endpoint.send::<F::Acquire>(&()).await?;
    endpoint.send::<S::Acquire>(&()).await?;
    endpoint.recv::<F::Granted>().await?;
    endpoint.recv::<S::Granted>().await?;
    endpoint.send::<CommitEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "commit_changes", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain commit_changes failed: {result}"
        )));
    }
    endpoint.send::<S::Return>(&()).await?;
    endpoint.send::<F::Return>(&()).await?;
    endpoint.recv::<S::Returned>().await?;
    endpoint.recv::<F::Returned>().await?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<CommitCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<CommitServiceDone>(&()).await?;

    // run_local_tests: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/run_local_tests", SERVICE_FD);
    loop {
        endpoint.send::<TestRequest>(&()).await?;
        endpoint.send::<TestRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            175 => {
                let request = branch.recv::<TestOpen>().await?;
                endpoint.send::<TestOpened>(&file.open(request)).await?;
            }
            177 => {
                let request = branch.recv::<TestWrite>().await?;
                endpoint.send::<TestWritten>(&file.write(request)).await?;
            }
            179 => {
                let request = branch.recv::<TestCommit>().await?;
                validation.record(file.arguments(request.0, "feature_name").and_then(
                    |arguments| {
                        if arguments["feature_name"] == assignment.feature {
                            Ok(arguments)
                        } else {
                            Err(ShellError::message(
                                "arguments name a different task object",
                            ))
                        }
                    },
                ));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    186 => {
                        decision.recv::<TestValidate>().await?;
                        endpoint.send::<TestAdmitted>(&()).await?;
                        break;
                    }
                    181 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<TestRejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected run_local_tests request")),
        }
    }
    endpoint.send::<TestEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "run_local_tests", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<TestCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<TestServiceDone>(&()).await?;

    // Test is terminal even when its domain verdict is false.
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
                    return Err(ShellError::message("developer guest failed"));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected completed-service request")),
        }
    }
}
