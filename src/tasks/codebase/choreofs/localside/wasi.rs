//! The literal request sequence follows the global; argument bytes are private.
use super::super::{
    global::*,
    wire::{COMMIT, Close, DESIGN, IMPLEMENT, TEST},
};
use crate::shell::{
    ShellResult,
    global::Exit,
    imp::file_wire::{ClosedFileMsg, ResultChunk, ResultChunkMsg},
};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};

#[hibana_runtime::localside]
pub async fn run<
    const ROLE: u8,
    const SUBJECT: u8,
    const TASK: u8,
    const FIRST: u8,
    const SECOND: u8,
>(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<ROLE>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut fuel_id = 0u16;

    // Design: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    private
                        .send::<protocol::FdWriteObjectReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdWriteObjectRetMsg>().await?;
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
                    private.send::<protocol::PathOpenReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PathOpenRetMsg>().await?;
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
                    private.send::<protocol::FdCloseReqMsg>(&request).await?;
                    let closed = private.recv::<ClosedFileMsg>().await?;

                    let completion = if request.0.fd() == SERVICE_FD {
                        let arguments = closed
                            .arguments(
                                Close::<TASK, SUBJECT, DESIGN>::PATH,
                                Close::<TASK, SUBJECT, DESIGN>::KEY,
                            )
                            .and_then(|arguments| {
                                Close::<TASK, SUBJECT, DESIGN>::new(request, arguments)
                            });
                        match arguments {
                            Ok(arguments) => {
                                task.send::<DesignRequest<TASK, SUBJECT>>(&arguments)
                                    .await?;
                                let bytes = task.recv::<DesignCompleted>().await?.to_vec();
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            Err(error) => {
                                let bytes = serde_json::to_vec(&serde_json::json!({"status":"invalid_arguments","error":error.to_string()}))
                .map_err(|error| crate::shell::ShellError::message(error.to_string()))?;
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                        }
                    } else {
                        closed.completion
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
                private
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
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

    // Implement: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    private
                        .send::<protocol::FdWriteObjectReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdWriteObjectRetMsg>().await?;
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
                    private.send::<protocol::PathOpenReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PathOpenRetMsg>().await?;
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
                    private.send::<protocol::FdCloseReqMsg>(&request).await?;
                    let closed = private.recv::<ClosedFileMsg>().await?;

                    let completion = if request.0.fd() == SERVICE_FD {
                        let arguments = closed
                            .arguments(
                                Close::<TASK, SUBJECT, IMPLEMENT>::PATH,
                                Close::<TASK, SUBJECT, IMPLEMENT>::KEY,
                            )
                            .and_then(|arguments| {
                                Close::<TASK, SUBJECT, IMPLEMENT>::new(request, arguments)
                            });
                        match arguments {
                            Ok(arguments) => {
                                task.send::<ImplementRequest<TASK, SUBJECT>>(&arguments)
                                    .await?;
                                let bytes = task.recv::<ImplementCompleted>().await?.to_vec();
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            Err(error) => {
                                let bytes = serde_json::to_vec(&serde_json::json!({"status":"invalid_arguments","error":error.to_string()}))
                .map_err(|error| crate::shell::ShellError::message(error.to_string()))?;
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                        }
                    } else {
                        closed.completion
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
                private
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
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

    // Commit: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    private
                        .send::<protocol::FdWriteObjectReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdWriteObjectRetMsg>().await?;
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
                    private.send::<protocol::PathOpenReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PathOpenRetMsg>().await?;
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
                    private.send::<protocol::FdCloseReqMsg>(&request).await?;
                    let closed = private.recv::<ClosedFileMsg>().await?;

                    let completion = if request.0.fd() == SERVICE_FD {
                        let arguments = closed
                            .arguments(
                                Close::<TASK, SUBJECT, COMMIT>::PATH,
                                Close::<TASK, SUBJECT, COMMIT>::KEY,
                            )
                            .and_then(|arguments| {
                                Close::<TASK, SUBJECT, COMMIT>::new(request, arguments)
                            });
                        match arguments {
                            Ok(arguments) => {
                                let (mut first, mut second) = {
                                    let [mut first_loan, mut second_loan] = task.par()?;
                                    async {
                                        #[localside]
                                        let first = async {
                                            let mut resource =
                                                first_loan.recv::<Loan<FIRST>>().await?;
                                            resource
                                                .send::<CommitRequest<TASK, SUBJECT, FIRST>>(
                                                    &arguments,
                                                )
                                                .await?;
                                            Ok(resource)
                                        };
                                        #[localside]
                                        let second = async {
                                            let mut resource =
                                                second_loan.recv::<Loan<SECOND>>().await?;
                                            resource
                                                .send::<CommitRequest<TASK, SUBJECT, SECOND>>(
                                                    &arguments,
                                                )
                                                .await?;
                                            Ok(resource)
                                        };
                                        Ok((first, second))
                                    }
                                    .await?
                                };
                                let (bytes, second_bytes) = async {
                                    #[localside]
                                    let bytes = async {
                                        Ok(first.recv::<CommitCompleted<FIRST>>().await?.to_vec())
                                    };
                                    #[localside]
                                    let second_bytes = async {
                                        Ok(second.recv::<CommitCompleted<SECOND>>().await?.to_vec())
                                    };
                                    Ok((bytes, second_bytes))
                                }
                                .await?;
                                if bytes != second_bytes {
                                    return Err(crate::shell::ShellError::message(
                                        "modules received different commit results",
                                    ));
                                }
                                {
                                    let [mut first_return, mut second_return] = task.par()?;
                                    async {
                                        #[localside]
                                        first_return.send::<Loan<FIRST>>(first);
                                        #[localside]
                                        second_return.send::<Loan<SECOND>>(second);
                                        Ok(())
                                    }
                                    .await?;
                                }
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            Err(error) => {
                                let bytes = serde_json::to_vec(&serde_json::json!({"status":"invalid_arguments","error":error.to_string()}))
                .map_err(|error| crate::shell::ShellError::message(error.to_string()))?;
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                        }
                    } else {
                        closed.completion
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
                private
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
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

    // Test: directly consumes this projected lifetime.
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => match pending.request()? {
                WasiImportRequest::FdWrite(request) => {
                    private.send::<protocol::FdWriteReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::FdWriteRetMsg>().await?;
                    pending.complete(WasiImportCompletion::FdWrite(completion))?;
                }
                WasiImportRequest::FdWriteObject(request) => {
                    private
                        .send::<protocol::FdWriteObjectReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdWriteObjectRetMsg>().await?;
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
                    private.send::<protocol::PathOpenReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PathOpenRetMsg>().await?;
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
                    private.send::<protocol::FdCloseReqMsg>(&request).await?;
                    let closed = private.recv::<ClosedFileMsg>().await?;

                    let completion = if request.0.fd() == SERVICE_FD {
                        let arguments = closed
                            .arguments(
                                Close::<TASK, SUBJECT, TEST>::PATH,
                                Close::<TASK, SUBJECT, TEST>::KEY,
                            )
                            .and_then(|arguments| {
                                Close::<TASK, SUBJECT, TEST>::new(request, arguments)
                            });
                        match arguments {
                            Ok(arguments) => {
                                task.send::<TestRequest<TASK, SUBJECT>>(&arguments).await?;
                                let bytes = task.recv::<TestCompleted>().await?.to_vec();
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd())),
                                ))?;
                                break;
                            }
                            Err(error) => {
                                let bytes = serde_json::to_vec(&serde_json::json!({"status":"invalid_arguments","error":error.to_string()}))
                .map_err(|error| crate::shell::ShellError::message(error.to_string()))?;
                                for (index, chunk) in
                                    bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                        }
                    } else {
                        closed.completion
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
                private
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
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
                    private
                        .send::<protocol::FdWriteObjectReqMsg>(&request)
                        .await?;
                    let completion = private.recv::<protocol::FdWriteObjectRetMsg>().await?;
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
                    private.send::<protocol::PathOpenReqMsg>(&request).await?;
                    let completion = private.recv::<protocol::PathOpenRetMsg>().await?;
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
                    private.send::<protocol::FdCloseReqMsg>(&request).await?;
                    let closed = private.recv::<ClosedFileMsg>().await?;

                    let completion = if request.0.fd() == SERVICE_FD {
                        let bytes = serde_json::to_vec(&serde_json::json!({"status":"operation_not_available","error":"task has no further shared operation"})).map_err(|error|crate::shell::ShellError::message(error.to_string()))?;
                        for (index, chunk) in
                            bytes.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                        {
                            private
                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                    chunk,
                                )?)
                                .await?;
                        }
                        protocol::FdClosedRet(protocol::FdClosed::new_with_errno(request.0.fd(), 8))
                    } else {
                        closed.completion
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
                private
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
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
                task.send::<ServiceExit>(&status).await?;
                private.send::<Exit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
