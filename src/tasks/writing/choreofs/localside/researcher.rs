//! The four literal protocol lifetimes are visible here. No phase variable.
use super::super::global::*;
use crate::{
    shell::{
        ShellError, ShellResult,
        imp::service::{Domain, File, Validation},
    },
    tasks::writing::global::SectionMessages,
};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::json;

/// Original task input, not protocol progress or an authorization table.
pub(crate) struct Assignment {
    pub agent: &'static str,
    pub topic: &'static str,
    pub section: &'static str,
    pub sections: [&'static str; 2],
}

pub(crate) async fn run<const ROLE: u8, F: SectionMessages, S: SectionMessages, D: Domain>(
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
    validation: &Validation,
    assignment: &Assignment,
) -> ShellResult<()> {
    // read_literature: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/read_literature", SERVICE_FD);
    loop {
        endpoint.send::<ReadRequest>(&()).await?;
        endpoint.send::<ReadRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            100 => {
                let request = branch.recv::<ReadOpen>().await?;
                endpoint.send::<ReadOpened>(&file.open(request)).await?;
            }
            102 => {
                let request = branch.recv::<ReadWrite>().await?;
                endpoint.send::<ReadWritten>(&file.write(request)).await?;
            }
            104 => {
                let request = branch.recv::<ReadCommit>().await?;
                validation.record(file.arguments(request.0, "topic").and_then(|arguments| {
                    if arguments["topic"] == assignment.topic {
                        Ok(arguments)
                    } else {
                        Err(ShellError::message(
                            "arguments name a different task object",
                        ))
                    }
                }));
                let decision = endpoint.offer().await?;
                match decision.label() {
                    111 => {
                        decision.recv::<ReadValidate>().await?;
                        endpoint.send::<ReadAdmitted>(&()).await?;
                        break;
                    }
                    106 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<ReadRejected<'_>>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected read_literature request")),
        }
    }
    endpoint.send::<ReadEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "read_literature", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain read_literature failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<ReadCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<ReadServiceDone>(&()).await?;

    // draft_content: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/draft_content", SERVICE_FD);
    loop {
        endpoint.send::<DraftRequest>(&()).await?;
        endpoint.send::<DraftRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            125 => {
                let request = branch.recv::<DraftOpen>().await?;
                endpoint.send::<DraftOpened>(&file.open(request)).await?;
            }
            127 => {
                let request = branch.recv::<DraftWrite>().await?;
                endpoint.send::<DraftWritten>(&file.write(request)).await?;
            }
            129 => {
                let request = branch.recv::<DraftCommit>().await?;
                validation.record(file.arguments(request.0, "section_name").and_then(
                    |arguments| {
                        if arguments["section_name"] == assignment.section {
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
                        decision.recv::<DraftValidate>().await?;
                        endpoint.send::<DraftAdmitted>(&()).await?;
                        break;
                    }
                    131 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<DraftRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected draft_content request")),
        }
    }
    endpoint.send::<DraftEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "draft_content", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain draft_content failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<DraftCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<DraftServiceDone>(&()).await?;

    // write_to_document: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/write_to_document", SERVICE_FD);
    loop {
        endpoint.send::<WriteRequest>(&()).await?;
        endpoint.send::<WriteRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            150 => {
                let request = branch.recv::<WriteOpen>().await?;
                endpoint.send::<WriteOpened>(&file.open(request)).await?;
            }
            152 => {
                let request = branch.recv::<WriteWrite>().await?;
                endpoint.send::<WriteWritten>(&file.write(request)).await?;
            }
            154 => {
                let request = branch.recv::<WriteCommit>().await?;
                validation.record(file.arguments(request.0, "sections").and_then(|arguments| {
                    let actual: std::collections::BTreeSet<_> = arguments["sections"]
                        .as_str()
                        .unwrap()
                        .split(',')
                        .map(str::trim)
                        .collect();
                    if actual == assignment.sections.into_iter().collect() {
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
                        decision.recv::<WriteValidate>().await?;
                        endpoint.send::<WriteAdmitted>(&()).await?;
                        break;
                    }
                    156 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<WriteRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected write_to_document request")),
        }
    }
    endpoint.recv::<F::Available>().await?;
    endpoint.recv::<S::Available>().await?;
    endpoint.send::<F::Acquire>(&()).await?;
    endpoint.send::<S::Acquire>(&()).await?;
    endpoint.recv::<F::Granted>().await?;
    endpoint.recv::<S::Granted>().await?;
    endpoint.send::<WriteEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "write_to_document", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain write_to_document failed: {result}"
        )));
    }
    endpoint.send::<S::Return>(&()).await?;
    endpoint.send::<F::Return>(&()).await?;
    endpoint.recv::<S::Returned>().await?;
    endpoint.recv::<F::Returned>().await?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<WriteCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<WriteServiceDone>(&()).await?;

    // review_own_work: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/review_own_work", SERVICE_FD);
    loop {
        endpoint.send::<ReviewRequest>(&()).await?;
        endpoint.send::<ReviewRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            175 => {
                let request = branch.recv::<ReviewOpen>().await?;
                endpoint.send::<ReviewOpened>(&file.open(request)).await?;
            }
            177 => {
                let request = branch.recv::<ReviewWrite>().await?;
                endpoint.send::<ReviewWritten>(&file.write(request)).await?;
            }
            179 => {
                let request = branch.recv::<ReviewCommit>().await?;
                validation.record(file.arguments(request.0, "section_name").and_then(
                    |arguments| {
                        if arguments["section_name"] == assignment.section {
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
                        decision.recv::<ReviewValidate>().await?;
                        endpoint.send::<ReviewAdmitted>(&()).await?;
                        break;
                    }
                    181 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision
                            .send::<ReviewRejected<'_>>(&bytes.as_slice())
                            .await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected review_own_work request")),
        }
    }
    endpoint.send::<ReviewEffect>(&()).await?;
    let result = domain
        .execute(assignment.agent, "review_own_work", validation.take()?)
        .await
        .map_err(ShellError::message)?;
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    endpoint
        .send::<ReviewCompleted<'_>>(&bytes.as_slice())
        .await?;
    endpoint.send::<ReviewServiceDone>(&()).await?;

    // Review is terminal even when its domain verdict is false.
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
                    return Err(ShellError::message("researcher guest failed"));
                }
                return Ok(());
            }
            _ => return Err(ShellError::message("unexpected completed-service request")),
        }
    }
}
