use super::super::global::*;
use crate::shell::imp::file_wire::{ClosedFileMsg, ResultChunk, ResultChunkMsg};
use crate::shell::{ShellResult, global::Exit};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};

pub async fn tester(
    guest: &mut HibanaWasiGuest<'_>,
    endpoint: &mut hibana::Endpoint<'_, TESTER_VM>,
) -> ShellResult<i32> {
    let mut fuel_id = 0u16;
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    endpoint.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    endpoint.recv::<RequestReady>().await?;
                    endpoint.send::<Write>(&request).await?;
                    let completion = endpoint.recv::<Written>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    endpoint.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    endpoint.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        endpoint.recv::<RequestReady>().await?;
                        endpoint.send::<Open>(&request).await?;
                        endpoint.recv::<Opened>().await?
                    } else {
                        endpoint.send::<protocol::PathOpenReqMsg>(&request).await?;
                        endpoint.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    endpoint
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    endpoint
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    endpoint
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    endpoint
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    endpoint.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    endpoint
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    endpoint
                        .send::<protocol::EnvironGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    endpoint
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    endpoint
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == SERVICE_FD {
                        endpoint.recv::<RequestReady>().await?;
                        endpoint.send::<Commit>(&request).await?;
                        let branch = endpoint.offer().await?;
                        match branch.label() {
                            105 => {
                                let arguments = {
                                    let bytes = branch.recv::<Admitted<'_>>().await?;
                                    super::super::imp::ApiArguments::<{super::super::imp::TEST}>::from_bytes(bytes)?
                                };
                                let mut environment =
                                    endpoint.recv::<Loan<'_, TEST_RESOURCE>>().await?;
                                environment.send::<TestRequest>(&arguments).await?;
                                let bytes = environment.recv::<TestCompleted<'_>>().await?.to_vec();
                                endpoint
                                    .send::<Loan<'_, TEST_RESOURCE>>(environment)
                                    .await?;
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    endpoint.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd()))
                            }
                            106 => {
                                let reply = branch.recv::<Rejected<'_>>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    endpoint.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            109 => branch.recv::<Unavailable>().await?,
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        endpoint.send::<protocol::FdCloseReqMsg>(&request).await?;
                        endpoint.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    endpoint
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    endpoint
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    endpoint
                        .send::<protocol::PollOneoffReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    endpoint.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                endpoint
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = endpoint.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                tokio::task::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                let status = exit.status() as i32;
                endpoint.recv::<RequestReady>().await?;
                endpoint.send::<ServiceExit>(&status).await?;
                endpoint.send::<Exit>(&status).await?;
                return Ok(status);
            }
        }
    }
}

pub async fn admin(
    guest: &mut HibanaWasiGuest<'_>,
    endpoint: &mut hibana::Endpoint<'_, ADMIN_VM>,
    outcome: &mut hibana::Endpoint<'_, ADMIN_OUTCOME_VM>,
) -> ShellResult<i32> {
    let mut fuel_id = 0u16;
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    endpoint.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    endpoint.recv::<RequestReady>().await?;
                    endpoint.send::<Write>(&request).await?;
                    let completion = endpoint.recv::<Written>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeRead>(&request).await?;
                        outcome.recv::<OutcomeReadDone>().await?
                    } else {
                        endpoint.send::<protocol::FdReadReqMsg>(&request).await?;
                        endpoint.recv::<protocol::FdReadRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    endpoint.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path == OUTCOME_PATH {
                        outcome.send::<OutcomeOpen>(&request).await?;
                        outcome.recv::<OutcomeOpened>().await?
                    } else if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        endpoint.recv::<RequestReady>().await?;
                        endpoint.send::<Open>(&request).await?;
                        endpoint.recv::<Opened>().await?
                    } else {
                        endpoint.send::<protocol::PathOpenReqMsg>(&request).await?;
                        endpoint.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    endpoint
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    endpoint
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeFileStat>(&request).await?;
                        outcome.recv::<OutcomeFileStatDone>().await?
                    } else {
                        endpoint
                            .send::<protocol::FdFilestatGetReqMsg>(&request)
                            .await?;
                        endpoint.recv::<protocol::FdFilestatGetRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    endpoint
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    endpoint.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    endpoint
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    endpoint
                        .send::<protocol::EnvironGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeStat>(&request).await?;
                        outcome.recv::<OutcomeStatDone>().await?
                    } else {
                        endpoint
                            .send::<protocol::FdFdstatGetReqMsg>(&request)
                            .await?;
                        endpoint.recv::<protocol::FdFdstatGetRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    endpoint
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeClose>(&request).await?;
                        outcome.recv::<OutcomeClosed>().await?
                    } else if request.0.fd() == SERVICE_FD {
                        endpoint.recv::<RequestReady>().await?;
                        endpoint.send::<Commit>(&request).await?;
                        let branch = endpoint.offer().await?;
                        match branch.label() {
                            105 => {
                                let arguments = {
                                    let bytes = branch.recv::<Admitted<'_>>().await?;
                                    super::super::imp::ApiArguments::<{super::super::imp::MIGRATE}>::from_bytes(bytes)?
                                };
                                let mut database =
                                    endpoint.recv::<Loan<'_, DATABASE_RESOURCE>>().await?;
                                database.send::<MigrationRequest>(&arguments).await?;
                                let bytes =
                                    database.recv::<MigrationCompleted<'_>>().await?.to_vec();
                                endpoint
                                    .send::<Loan<'_, DATABASE_RESOURCE>>(database)
                                    .await?;
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    endpoint.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd()))
                            }
                            106 => {
                                let reply = branch.recv::<Rejected<'_>>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    endpoint.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            109 => branch.recv::<Unavailable>().await?,
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        endpoint.send::<protocol::FdCloseReqMsg>(&request).await?;
                        endpoint.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    endpoint
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    endpoint
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    endpoint
                        .send::<protocol::PollOneoffReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    endpoint.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                endpoint
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = endpoint.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                tokio::task::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                let status = exit.status() as i32;
                endpoint.recv::<RequestReady>().await?;
                endpoint.send::<ServiceExit>(&status).await?;
                endpoint.send::<Exit>(&status).await?;
                return Ok(status);
            }
        }
    }
}

pub async fn backend(
    guest: &mut HibanaWasiGuest<'_>,
    endpoint: &mut hibana::Endpoint<'_, BACKEND_VM>,
    outcome: &mut hibana::Endpoint<'_, BACKEND_OUTCOME_VM>,
) -> ShellResult<i32> {
    let mut fuel_id = 0u16;
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    endpoint.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    endpoint.recv::<RequestReady>().await?;
                    endpoint.send::<Write>(&request).await?;
                    let completion = endpoint.recv::<Written>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeRead>(&request).await?;
                        outcome.recv::<OutcomeReadDone>().await?
                    } else {
                        endpoint.send::<protocol::FdReadReqMsg>(&request).await?;
                        endpoint.recv::<protocol::FdReadRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    endpoint.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path == OUTCOME_PATH {
                        outcome.send::<OutcomeOpen>(&request).await?;
                        outcome.recv::<OutcomeOpened>().await?
                    } else if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        endpoint.recv::<RequestReady>().await?;
                        endpoint.send::<Open>(&request).await?;
                        endpoint.recv::<Opened>().await?
                    } else {
                        endpoint.send::<protocol::PathOpenReqMsg>(&request).await?;
                        endpoint.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    endpoint
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    endpoint
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeFileStat>(&request).await?;
                        outcome.recv::<OutcomeFileStatDone>().await?
                    } else {
                        endpoint
                            .send::<protocol::FdFilestatGetReqMsg>(&request)
                            .await?;
                        endpoint.recv::<protocol::FdFilestatGetRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    endpoint
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    endpoint.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    endpoint
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    endpoint
                        .send::<protocol::EnvironGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeStat>(&request).await?;
                        outcome.recv::<OutcomeStatDone>().await?
                    } else {
                        endpoint
                            .send::<protocol::FdFdstatGetReqMsg>(&request)
                            .await?;
                        endpoint.recv::<protocol::FdFdstatGetRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    endpoint
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == OUTCOME_FD {
                        outcome.send::<OutcomeClose>(&request).await?;
                        outcome.recv::<OutcomeClosed>().await?
                    } else if request.0.fd() == SERVICE_FD {
                        endpoint.recv::<RequestReady>().await?;
                        endpoint.send::<Commit>(&request).await?;
                        let branch = endpoint.offer().await?;
                        match branch.label() {
                            105 => {
                                branch.recv::<Admitted<'_>>().await?;
                                let bytes = endpoint.recv::<ResultBytes<'_>>().await?.to_vec();
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    endpoint.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd()))
                            }
                            106 => {
                                let reply = branch.recv::<Rejected<'_>>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    endpoint.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            109 => branch.recv::<Unavailable>().await?,
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        endpoint.send::<protocol::FdCloseReqMsg>(&request).await?;
                        endpoint.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    endpoint
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    endpoint
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    endpoint
                        .send::<protocol::PollOneoffReqMsg>(&request)
                        .await?;
                    let completion = endpoint.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    endpoint.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = endpoint.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                endpoint
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = endpoint.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                tokio::task::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                let status = exit.status() as i32;
                endpoint.recv::<RequestReady>().await?;
                endpoint.send::<ServiceExit>(&status).await?;
                endpoint.send::<Exit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
