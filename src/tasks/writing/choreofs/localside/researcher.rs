//! The four literal protocol lifetimes are visible here. No phase variable.
use super::super::global::*;
use crate::shell::{
    ShellError, ShellResult,
    imp::service::{Domain, File, Validation},
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

#[hibana_runtime::localside]
pub(crate) async fn run<const ROLE: u8, const FIRST: u8, const SECOND: u8>(
    endpoint: &mut Endpoint<ROLE>,
    domain: &impl Domain,
    (validation, assignment): (&Validation, &Assignment),
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
                let response = file.open(request);
                endpoint.send::<ReadOpened>(&response).await?;
            }
            102 => {
                let request = branch.recv::<ReadWrite>().await?;
                let response = file.write(request);
                endpoint.send::<ReadWritten>(&response).await?;
            }
            104 => {
                let request = branch.recv::<ReadCommit>().await?;
                validation.record(file.arguments(request, "topic").and_then(|arguments| {
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
                        decision.send::<ReadRejected>(&bytes.as_slice()).await?;
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
    let response = bytes.as_slice();
    endpoint.send::<ReadCompleted>(&response).await?;

    // draft_content: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/draft_content", SERVICE_FD);
    loop {
        endpoint.send::<DraftRequest>(&()).await?;
        endpoint.send::<DraftRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            125 => {
                let request = branch.recv::<DraftOpen>().await?;
                let response = file.open(request);
                endpoint.send::<DraftOpened>(&response).await?;
            }
            127 => {
                let request = branch.recv::<DraftWrite>().await?;
                let response = file.write(request);
                endpoint.send::<DraftWritten>(&response).await?;
            }
            129 => {
                let request = branch.recv::<DraftCommit>().await?;
                validation.record(
                    file.arguments(request, "section_name")
                        .and_then(|arguments| {
                            if arguments["section_name"] == assignment.section {
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
                        decision.send::<DraftRejected>(&bytes.as_slice()).await?;
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
    let response = bytes.as_slice();
    endpoint.send::<DraftCompleted>(&response).await?;

    // write_to_document: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/write_to_document", SERVICE_FD);
    let admitted = loop {
        endpoint.send::<WriteRequest>(&()).await?;
        endpoint.send::<WriteRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            150 => {
                let request = branch.recv::<WriteOpen>().await?;
                let response = file.open(request);
                endpoint.send::<WriteOpened>(&response).await?;
            }
            152 => {
                let request = branch.recv::<WriteWrite>().await?;
                let response = file.write(request);
                endpoint.send::<WriteWritten>(&response).await?;
            }
            154 => {
                let request = branch.recv::<WriteCommit>().await?;
                validation.record(file.arguments(request, "sections").and_then(|arguments| {
                    let actual: Vec<_> = arguments["sections"]
                        .as_str()
                        .unwrap()
                        .split(',')
                        .map(str::trim)
                        .collect();
                    if actual.len() == 2
                        && actual
                            .into_iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            == assignment.sections.into_iter().collect()
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
                        decision.recv::<WriteValidate>().await?;
                        let arguments =
                            super::super::imp::WriteArguments::from_value(validation.take()?)?;
                        let mut bytes = vec![0; file.bytes.len()];
                        let count =
                            hibana::runtime::wire::WireEncode::encode_into(&arguments, &mut bytes)
                                .map_err(|error| ShellError::message(format!("{error:?}")))?;
                        let payload = &bytes[..count];
                        endpoint.send::<WriteAdmitted>(&payload).await?;
                        break arguments;
                    }
                    156 => {
                        let bytes = serde_json::to_vec(
                            &json!({"status":"invalid_arguments", "error":validation.rejection()?}),
                        )
                        .map_err(|error| ShellError::message(error.to_string()))?;
                        decision.send::<WriteRejected>(&bytes.as_slice()).await?;
                    }
                    _ => return Err(ShellError::message("unexpected validation branch")),
                }
            }
            _ => return Err(ShellError::message("unexpected write_to_document request")),
        }
    };
    let (first, second) = {
        let [mut first, mut second] = endpoint.par()?;
        async {
            #[localside]
            let (first, second) = (
                first.recv::<SectionRequest<FIRST>>(),
                second.recv::<SectionRequest<SECOND>>(),
            );
            Ok((first, second))
        }
        .await?
    };
    if first != second || first != admitted {
        return Err(ShellError::message(
            "section arguments differ from the admitted request",
        ));
    }
    let result = domain
        .execute(assignment.agent, "write_to_document", first.into_value())
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain write_to_document failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    {
        let [mut first, mut second] = endpoint.par()?;
        let payload = bytes.as_slice();
        async {
            async {
                first.send::<SectionWritten<FIRST>>(&payload).await?;
            };
            async {
                second.send::<SectionWritten<SECOND>>(&payload).await?;
            };
            Ok(())
        }
        .await?;
    }

    // review_own_work: the Endpoint admits this lifetime, including reconsideration.
    let mut file = File::new(b"service/review_own_work", SERVICE_FD);
    loop {
        endpoint.send::<ReviewRequest>(&()).await?;
        endpoint.send::<ReviewRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            175 => {
                let request = branch.recv::<ReviewOpen>().await?;
                let response = file.open(request);
                endpoint.send::<ReviewOpened>(&response).await?;
            }
            177 => {
                let request = branch.recv::<ReviewWrite>().await?;
                let response = file.write(request);
                endpoint.send::<ReviewWritten>(&response).await?;
            }
            179 => {
                let request = branch.recv::<ReviewCommit>().await?;
                validation.record(
                    file.arguments(request, "section_name")
                        .and_then(|arguments| {
                            if arguments["section_name"] == assignment.section {
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
                        decision.send::<ReviewRejected>(&bytes.as_slice()).await?;
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
    let response = bytes.as_slice();
    endpoint.send::<ReviewCompleted>(&response).await?;

    // Review is terminal even when its domain verdict is false.
    loop {
        endpoint.send::<RejectRequest>(&()).await?;
        endpoint.send::<ClosedRequestReady>(&()).await?;
        let branch = endpoint.offer().await?;
        match branch.label() {
            200 => {
                branch.recv::<ClosedOpen>().await?;
                let response = protocol::PathOpenedRet(protocol::PathOpened::new(0, 8));
                endpoint.send::<ClosedOpened>(&response).await?;
            }
            202 => {
                let request = branch.recv::<ClosedWrite>().await?;
                let response = protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(
                    hibana_wasip1_runtime::choreofs::ChoreoFsRequest::request(&request)
                        .0
                        .fd(),
                    0,
                    8,
                ));
                endpoint.send::<ClosedWritten>(&response).await?;
            }
            204 => {
                let request = branch.recv::<ClosedCommit>().await?;
                let response = protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                    hibana_wasip1_runtime::choreofs::ChoreoFsRequest::request(&request)
                        .0
                        .fd(),
                    8,
                ));
                endpoint.send::<ClosedUnavailable>(&response).await?;
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
