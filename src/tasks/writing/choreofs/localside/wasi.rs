//! Five explicit projected lifetimes, without a stored phase selector.
use super::super::global::*;
use crate::shell::imp::file_wire::{ClosedFileMsg, ResultChunk, ResultChunkMsg};
use crate::shell::{ShellResult, global::Exit};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8, const FIRST: u8, const SECOND: u8>(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<ROLE>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut fuel_id = 0u16;

    // Read: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    task.recv::<ReadRequestReady>().await?;
                    task.send::<ReadWrite>(&request).await?;
                    let completion = task.recv::<ReadWritten>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    private.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    private.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        task.recv::<ReadRequestReady>().await?;
                        task.send::<ReadOpen>(&request).await?;
                        task.recv::<ReadOpened>().await?
                    } else {
                        private.send::<protocol::PathOpenReqMsg>(&request).await?;
                        private.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    private
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    private
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    private
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    private
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    private.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    private
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    private.send::<protocol::EnvironGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    private
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    private
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == SERVICE_FD {
                        task.recv::<ReadRequestReady>().await?;
                        task.send::<ReadCommit>(&request).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            105 => {
                                branch.recv::<ReadAdmitted>().await?;
                                let bytes = task.recv::<ReadCompleted>().await?.to_vec();
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            106 => {
                                let reply = branch.recv::<ReadRejected>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        private.send::<protocol::FdCloseReqMsg>(&request).await?;
                        private.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    private
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    private
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    private.send::<protocol::PollOneoffReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    private.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                private
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = private.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                hibana_runtime::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                return Err(crate::shell::ShellError::message(format!(
                    "guest exited before read completed: {}",
                    exit.status()
                )));
            }
        }
    }

    // Draft: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    task.recv::<DraftRequestReady>().await?;
                    task.send::<DraftWrite>(&request).await?;
                    let completion = task.recv::<DraftWritten>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    private.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    private.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        task.recv::<DraftRequestReady>().await?;
                        task.send::<DraftOpen>(&request).await?;
                        task.recv::<DraftOpened>().await?
                    } else {
                        private.send::<protocol::PathOpenReqMsg>(&request).await?;
                        private.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    private
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    private
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    private
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    private
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    private.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    private
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    private.send::<protocol::EnvironGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    private
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    private
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == SERVICE_FD {
                        task.recv::<DraftRequestReady>().await?;
                        task.send::<DraftCommit>(&request).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            130 => {
                                branch.recv::<DraftAdmitted>().await?;
                                let bytes = task.recv::<DraftCompleted>().await?.to_vec();
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            131 => {
                                let reply = branch.recv::<DraftRejected>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        private.send::<protocol::FdCloseReqMsg>(&request).await?;
                        private.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    private
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    private
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    private.send::<protocol::PollOneoffReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    private.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                private
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = private.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                hibana_runtime::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                return Err(crate::shell::ShellError::message(format!(
                    "guest exited before draft completed: {}",
                    exit.status()
                )));
            }
        }
    }

    // Write: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    task.recv::<WriteRequestReady>().await?;
                    task.send::<WriteWrite>(&request).await?;
                    let completion = task.recv::<WriteWritten>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    private.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    private.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        task.recv::<WriteRequestReady>().await?;
                        task.send::<WriteOpen>(&request).await?;
                        task.recv::<WriteOpened>().await?
                    } else {
                        private.send::<protocol::PathOpenReqMsg>(&request).await?;
                        private.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    private
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    private
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    private
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    private
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    private.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    private
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    private.send::<protocol::EnvironGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    private
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    private
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == SERVICE_FD {
                        task.recv::<WriteRequestReady>().await?;
                        task.send::<WriteCommit>(&request).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            155 => {
                                let arguments = {
                                    let bytes = branch.recv::<WriteAdmitted>().await?;
                                    super::super::imp::WriteArguments::from_bytes(bytes)?
                                };
                                let (mut first, mut second) = {
                                    let [mut first_loan, mut second_loan] = task.par()?;
                                    async {
                                        async {
                                            let mut section =
                                                first_loan.recv::<Loan<FIRST>>().await?;
                                            section
                                                .send::<SectionRequest<FIRST>>(&arguments)
                                                .await?;
                                            Ok(section)
                                        };
                                        async {
                                            let mut section =
                                                second_loan.recv::<Loan<SECOND>>().await?;
                                            section
                                                .send::<SectionRequest<SECOND>>(&arguments)
                                                .await?;
                                            Ok(section)
                                        };
                                    }
                                    .await?
                                };
                                let (first_result, second_result) = async {
                                    async {
                                        Ok(first.recv::<SectionWritten<FIRST>>().await?.to_vec())
                                    };
                                    async {
                                        Ok(second.recv::<SectionWritten<SECOND>>().await?.to_vec())
                                    };
                                }
                                .await?;
                                if first_result != second_result {
                                    return Err(crate::shell::ShellError::message(
                                        "sections received different results",
                                    ));
                                }
                                {
                                    let [mut first_return, mut second_return] = task.par()?;
                                    async {
                                        async {
                                            first_return.send::<Loan<FIRST>>(first).await?;
                                        };
                                        async {
                                            second_return.send::<Loan<SECOND>>(second).await?;
                                        };
                                        Ok(())
                                    }
                                    .await?;
                                }
                                let bytes = first_result;
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            156 => {
                                let reply = branch.recv::<WriteRejected>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        private.send::<protocol::FdCloseReqMsg>(&request).await?;
                        private.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    private
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    private
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    private.send::<protocol::PollOneoffReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    private.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                private
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = private.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                hibana_runtime::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                return Err(crate::shell::ShellError::message(format!(
                    "guest exited before write completed: {}",
                    exit.status()
                )));
            }
        }
    }

    // Review: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    task.recv::<ReviewRequestReady>().await?;
                    task.send::<ReviewWrite>(&request).await?;
                    let completion = task.recv::<ReviewWritten>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    private.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    private.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        task.recv::<ReviewRequestReady>().await?;
                        task.send::<ReviewOpen>(&request).await?;
                        task.recv::<ReviewOpened>().await?
                    } else {
                        private.send::<protocol::PathOpenReqMsg>(&request).await?;
                        private.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    private
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    private
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    private
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    private
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    private.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    private
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    private.send::<protocol::EnvironGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    private
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    private
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == SERVICE_FD {
                        task.recv::<ReviewRequestReady>().await?;
                        task.send::<ReviewCommit>(&request).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            180 => {
                                branch.recv::<ReviewAdmitted>().await?;
                                let bytes = task.recv::<ReviewCompleted>().await?.to_vec();
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            181 => {
                                let reply = branch.recv::<ReviewRejected>().await?.to_vec();
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    let response = ResultChunk::new(
                                        index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                        chunk,
                                    )?;
                                    private.send::<ResultChunkMsg>(&response).await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        private.send::<protocol::FdCloseReqMsg>(&request).await?;
                        private.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    private
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    private
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    private.send::<protocol::PollOneoffReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    private.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                private
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = private.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                hibana_runtime::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                return Err(crate::shell::ShellError::message(format!(
                    "guest exited before review completed: {}",
                    exit.status()
                )));
            }
        }
    }

    // Closed: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    task.recv::<ClosedRequestReady>().await?;
                    task.send::<ClosedWrite>(&request).await?;
                    let completion = task.recv::<ClosedWritten>().await?;
                    pending.complete(WasiImportCompletion::FdWriteObject(completion))?;
                }
                WasiImportRequest::FdRead(request) => {
                    private.send::<protocol::FdReadReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReadRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdRead(completion))?;
                }
                WasiImportRequest::FdReaddir(request) => {
                    private.send::<protocol::FdReaddirReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdReaddirRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdReaddir(completion))?;
                }
                WasiImportRequest::PathOpen(request) => {
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        task.recv::<ClosedRequestReady>().await?;
                        task.send::<ClosedOpen>(&request).await?;
                        task.recv::<ClosedOpened>().await?
                    } else {
                        private.send::<protocol::PathOpenReqMsg>(&request).await?;
                        private.recv::<protocol::PathOpenRetMsg>().await?
                    };
                    pending.complete(WasiImportCompletion::PathOpen(completion))?;
                }
                WasiImportRequest::FdPrestatGet(request) => {
                    private
                        .send::<protocol::FdPrestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatGet(completion))?;
                }
                WasiImportRequest::FdPrestatDirName(request) => {
                    private
                        .send::<protocol::FdPrestatDirNameReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdPrestatDirNameRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdPrestatDirName(completion))?;
                }
                WasiImportRequest::FdFilestatGet(request) => {
                    private
                        .send::<protocol::FdFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFilestatGet(completion))?;
                }
                WasiImportRequest::ArgsSizesGet(request) => {
                    private
                        .send::<protocol::ArgsSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ArgsSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsSizesGet(completion))?;
                }
                WasiImportRequest::ArgsGet(request) => {
                    private.send::<protocol::ArgsGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::ArgsGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ArgsGet(completion))?;
                }
                WasiImportRequest::EnvironSizesGet(request) => {
                    private
                        .send::<protocol::EnvironSizesGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::EnvironSizesGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironSizesGet(completion))?;
                }
                WasiImportRequest::EnvironGet(request) => {
                    private.send::<protocol::EnvironGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::EnvironGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::EnvironGet(completion))?;
                }
                WasiImportRequest::FdFdstatGet(request) => {
                    private
                        .send::<protocol::FdFdstatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdFdstatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdFdstatGet(completion))?;
                }
                WasiImportRequest::PathFilestatGet(request) => {
                    private
                        .send::<protocol::PathFilestatGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::PathFilestatGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PathFilestatGet(completion))?;
                }
                WasiImportRequest::FdClose(request) => {
                    let completion = if request.0.fd() == SERVICE_FD {
                        task.recv::<ClosedRequestReady>().await?;
                        task.send::<ClosedCommit>(&request).await?;
                        let branch = task.offer().await?;
                        match branch.label() {
                            209 => branch.recv::<ClosedUnavailable>().await?,
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
                        }
                    } else {
                        private.send::<protocol::FdCloseReqMsg>(&request).await?;
                        private.recv::<ClosedFileMsg>().await?.completion
                    };
                    pending.complete(WasiImportCompletion::FdClose(completion))?;
                }
                WasiImportRequest::ClockResGet(request) => {
                    private
                        .send::<protocol::ClockResGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockResGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockResGet(completion))?;
                }
                WasiImportRequest::ClockTimeGet(request) => {
                    private
                        .send::<protocol::ClockTimeGetReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::ClockTimeGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::ClockTimeGet(completion))?;
                }
                WasiImportRequest::PollOneoff(request) => {
                    private.send::<protocol::PollOneoffReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PollOneoffRetMsg>().await?;
                    pending.complete(WasiImportCompletion::PollOneoff(completion))?;
                }
                WasiImportRequest::RandomGet(request) => {
                    private.send::<protocol::RandomGetReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::RandomGetRetMsg>().await?;
                    pending.complete(WasiImportCompletion::RandomGet(completion))?;
                }
            },
            WasiBoundaryStep::MemoryGrowPending(pending) => {
                let response = pending.request();
                private
                    .send::<protocol::MemoryGrowReqMsg>(&response)
                    .await?;
                let response = private.recv::<protocol::MemoryGrowRetMsg>().await?;
                pending.complete(response)?;
            }
            WasiBoundaryStep::BudgetExpired(_) => {
                fuel_id = fuel_id.wrapping_add(1);
                hibana_runtime::yield_now().await;
            }
            WasiBoundaryStep::Exit(exit) => {
                let status = exit.status() as i32;
                task.recv::<ClosedRequestReady>().await?;
                task.send::<ServiceExit>(&status).await?;
                private.send::<Exit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
