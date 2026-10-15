use super::super::global::*;
use crate::shell::{ShellResult, global::Exit as IoExit};
use hibana_wasip1_runtime::{
    exchange::{HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportRequest},
    protocol,
};

pub async fn run<const ROLE: u8>(
    guest: &mut HibanaWasiGuest<'_>,
    endpoint: &mut hibana::Endpoint<'_, ROLE>,
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
                    let completion = if request.0.fd() == REVIEW_B_FD {
                        endpoint.recv::<ReviewRequestReadyB>().await?;
                        endpoint.send::<WriteB>(&request).await?;
                        endpoint.recv::<WrittenB>().await?
                    } else {
                        endpoint.recv::<ReviewRequestReady>().await?;
                        endpoint.send::<Write>(&request).await?;
                        endpoint.recv::<Written>().await?
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
                    let path = request
                        .0
                        .path()
                        .strip_prefix(b"/")
                        .unwrap_or(request.0.path());
                    let completion = if path == b"service/code_review/notification_feature" {
                        endpoint.recv::<ReviewRequestReadyB>().await?;
                        endpoint.send::<OpenB>(&request).await?;
                        endpoint.recv::<OpenedB>().await.map_err(|e| {
                            crate::shell::ShellError::message(format!("review WASI OpenedB: {e:?}"))
                        })?
                    } else if path.starts_with(SERVICE_PREFIX) && path != RESULT_PATH {
                        endpoint.recv::<ReviewRequestReady>().await?;
                        endpoint.send::<Open>(&request).await?;
                        endpoint.recv::<Opened>().await.map_err(|e| {
                            crate::shell::ShellError::message(format!("review WASI Opened: {e:?}"))
                        })?
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
                    let completion = if request.0.fd() == REVIEW_B_FD {
                        endpoint.recv::<ReviewRequestReadyB>().await?;
                        endpoint.send::<CommitB>(&request).await?;
                        let branch = endpoint.offer().await?;
                        match branch.label() {
                            48 => {
                                branch.recv::<AdmittedB>().await?;
                                let bytes = endpoint.recv::<ResultB<'_>>().await?.to_vec();
                                endpoint.recv::<ServiceDoneB>().await?;
                                reply_bytes.replace(bytes);
                                protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd()))
                            }
                            49 => {
                                reply_bytes.replace(branch.recv::<RejectedB<'_>>().await?.to_vec());
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            66 => branch.recv::<UnavailableB>().await?,
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected review B completion",
                                ));
                            }
                        }
                    } else if request.0.fd() == SERVICE_FD {
                        endpoint.recv::<ReviewRequestReady>().await?;
                        endpoint.send::<Commit>(&request).await?;
                        let branch = endpoint.offer().await?;
                        match branch.label() {
                            8 => {
                                branch.recv::<Admitted>().await?;
                                let bytes = endpoint.recv::<Result<'_>>().await?.to_vec();
                                endpoint.recv::<ServiceDone>().await?;
                                reply_bytes.replace(bytes);
                                protocol::FdClosedRet(protocol::FdClosed::new(request.0.fd()))
                            }
                            9 => {
                                reply_bytes.replace(branch.recv::<Rejected<'_>>().await?.to_vec());
                                protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                    request.0.fd(),
                                    28,
                                ))
                            }
                            26 => branch.recv::<Unavailable>().await?,
                            _ => {
                                return Err(crate::shell::ShellError::message(
                                    "unexpected service completion",
                                ));
                            }
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
                endpoint.recv::<ReviewRequestReady>().await?;
                endpoint.send::<Exit>(&status).await?;
                endpoint.recv::<ReviewRequestReadyB>().await?;
                endpoint.send::<ExitB>(&()).await?;
                endpoint.recv::<ReviewExitAck>().await?;
                endpoint.recv::<ReviewExitAckB>().await?;
                endpoint.send::<IoExit>(&status).await?;
                return Ok(status);
            }
        }
    }
}
