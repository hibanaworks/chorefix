use super::super::{global::*, imp::PhilosopherArguments};
use crate::shell::{
    ShellError, ShellResult,
    global::Exit,
    imp::file_wire::{ClosedFileMsg, ResultChunk, ResultChunkMsg},
};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};

#[hibana_runtime::scope]
pub async fn run<const ROLE: u8, const LEFT: u8, const RIGHT: u8>(
    guest: &mut HibanaWasiGuest,
    endpoint: &mut hibana::Endpoint<ROLE>,
) -> ShellResult<i32> {
    let [mut task, mut private] = endpoint.par()?;
    let mut fuel_id = 0u16;

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

                    if request.0.fd() == SERVICE_FD {
                        match if closed.completion.0.errno() == 0 && closed.path == b"service/think"
                        {
                            PhilosopherArguments::from_bytes(&closed.bytes)
                        } else {
                            Err(ShellError::message(
                                "service file does not match the request",
                            ))
                        } {
                            Err(error) => {
                                let reply = serde_json::to_vec(&serde_json::json!({"status":"invalid_arguments","error":error.to_string()})).map_err(|error| ShellError::message(error.to_string()))?;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                        SERVICE_FD, 28,
                                    )),
                                ))?;
                            }
                            Ok(arguments) => {
                                task.send::<Think>(&arguments).await?;
                                let bytes = task.recv::<Thought>().await?.to_vec();
                                let reply = bytes;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(SERVICE_FD)),
                                ))?;
                                break;
                            }
                        }
                    } else {
                        pending.complete(WasiImportCompletion::FdClose(closed.completion))?;
                    }
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
                return Err(ShellError::message(format!(
                    "guest exited before think completed: {}",
                    exit.status()
                )));
            }
        }
    }

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

                    if request.0.fd() == SERVICE_FD {
                        match if closed.completion.0.errno() == 0 && closed.path == b"service/eat" {
                            PhilosopherArguments::from_bytes(&closed.bytes)
                        } else {
                            Err(ShellError::message(
                                "service file does not match the request",
                            ))
                        } {
                            Err(error) => {
                                let reply = serde_json::to_vec(&serde_json::json!({"status":"invalid_arguments","error":error.to_string()})).map_err(|error| ShellError::message(error.to_string()))?;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                        SERVICE_FD, 28,
                                    )),
                                ))?;
                            }
                            Ok(arguments) => {
                                let (mut left, mut right) = {
                                    let [mut left_loan, mut right_loan] = task.par()?;
                                    async {
                                        async {
                                            let mut resource =
                                                left_loan.recv::<Loan<LEFT>>().await?;
                                            resource.send::<Eat<LEFT>>(&arguments).await?;
                                            Ok(resource)
                                        };
                                        async {
                                            let mut resource =
                                                right_loan.recv::<Loan<RIGHT>>().await?;
                                            resource.send::<Eat<RIGHT>>(&arguments).await?;
                                            Ok(resource)
                                        };
                                    }
                                    .await?
                                };
                                let (left_result, right_result) = async {
                                    async { Ok(left.recv::<Meal<LEFT>>().await?.to_vec()) };
                                    async { Ok(right.recv::<Meal<RIGHT>>().await?.to_vec()) };
                                }
                                .await?;
                                if left_result != right_result {
                                    return Err(ShellError::message(
                                        "forks received different meal results",
                                    ));
                                }
                                {
                                    let [mut left_return, mut right_return] = task.par()?;
                                    async {
                                        async {
                                            left_return.send::<Loan<LEFT>>(left).await?;
                                        };
                                        async {
                                            right_return.send::<Loan<RIGHT>>(right).await?;
                                        };
                                        Ok(())
                                    }
                                    .await?;
                                }
                                let reply = left_result;
                                for (index, chunk) in
                                    reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                                {
                                    private
                                        .send::<ResultChunkMsg>(&ResultChunk::new(
                                            index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                            chunk,
                                        )?)
                                        .await?;
                                }
                                pending.complete(WasiImportCompletion::FdClose(
                                    protocol::FdClosedRet(protocol::FdClosed::new(SERVICE_FD)),
                                ))?;
                                break;
                            }
                        }
                    } else {
                        pending.complete(WasiImportCompletion::FdClose(closed.completion))?;
                    }
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
                return Err(ShellError::message(format!(
                    "guest exited before eat completed: {}",
                    exit.status()
                )));
            }
        }
    }
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

                    if request.0.fd() == SERVICE_FD {
                        let bytes=serde_json::to_vec(&serde_json::json!({"status":"operation_not_available","error":"task has no further shared operation"})).map_err(|error|ShellError::message(error.to_string()))?;
                        let reply = bytes;
                        for (index, chunk) in
                            reply.chunks(protocol::WASIP1_IO_CHUNK_CAPACITY).enumerate()
                        {
                            private
                                .send::<ResultChunkMsg>(&ResultChunk::new(
                                    index * protocol::WASIP1_IO_CHUNK_CAPACITY,
                                    chunk,
                                )?)
                                .await?;
                        }
                        pending.complete(WasiImportCompletion::FdClose(protocol::FdClosedRet(
                            protocol::FdClosed::new_with_errno(SERVICE_FD, 8),
                        )))?;
                    } else {
                        pending.complete(WasiImportCompletion::FdClose(closed.completion))?;
                    }
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
