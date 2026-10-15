use crate::shell::{ShellResult, global::Exit};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};

pub async fn run<const ROLE: u8>(
    guest: &mut HibanaWasiGuest<'_>,
    endpoint: &mut hibana::Endpoint<'_, ROLE>,
    service: tokio::sync::mpsc::UnboundedSender<crate::tasks::cicd::choreofs::imp::Import>,
    reply_bytes: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
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
                    let (reply, response) = tokio::sync::oneshot::channel();
                    let completion = if service
                        .send(crate::tasks::cicd::choreofs::imp::Import::Write(
                            request, reply,
                        ))
                        .is_ok()
                    {
                        response.await.map_err(|_| {
                            crate::shell::ShellError::message("service write owner closed")
                        })?
                    } else {
                        protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(
                            request.0.fd(),
                            0,
                            8,
                        ))
                    };
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
                    // Static object binding identifies a service request; it
                    // grants no phase. Only the task Endpoint can admit it.
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path.starts_with(b"service/") && path != b"service/result" {
                        let (reply, response) = tokio::sync::oneshot::channel();
                        if service
                            .send(crate::tasks::cicd::choreofs::imp::Import::Open(
                                request, reply,
                            ))
                            .is_ok()
                        {
                            response.await.map_err(|_| {
                                crate::shell::ShellError::message("service open owner closed")
                            })?
                        } else {
                            protocol::PathOpenedRet(protocol::PathOpened::new(0, 8))
                        }
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
                    let completion = if request.0.fd() == 4 {
                        let (reply, response) = tokio::sync::oneshot::channel();
                        if service
                            .send(crate::tasks::cicd::choreofs::imp::Import::Close(
                                request.0, reply,
                            ))
                            .is_ok()
                        {
                            let (completion, bytes) = response.await.map_err(|_| {
                                crate::shell::ShellError::message("service close owner closed")
                            })?;
                            reply_bytes.replace(bytes);
                            completion
                        } else {
                            protocol::FdClosedRet(protocol::FdClosed::new_with_errno(4, 8))
                        }
                    } else {
                        endpoint.send::<protocol::FdCloseReqMsg>(&request).await?;
                        endpoint.recv::<protocol::FdCloseRetMsg>().await?
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
                endpoint
                    .send::<protocol::MemoryGrowReqMsg>(&pending.request())
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
                endpoint.send::<Exit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
