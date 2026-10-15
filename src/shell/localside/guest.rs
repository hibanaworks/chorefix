use crate::shell::{
    ShellResult,
    global::{Exit, GUEST},
};
use hibana_wasip1_runtime::{
    exchange::{
        HibanaWasiGuest, WasiBoundaryStep, WasiImportCompletion, WasiImportPending,
        WasiImportRequest,
    },
    protocol,
};

pub async fn run(
    guest: &mut HibanaWasiGuest<'_>,
    endpoint: &mut hibana::Endpoint<'_, GUEST>,
) -> ShellResult<i32> {
    let mut fuel_id = 0u16;
    loop {
        match guest.resume_wasi_boundary(protocol::BudgetRun::new(fuel_id, 0, 100_000))? {
            WasiBoundaryStep::ImportPending(pending) => complete_import(endpoint, pending).await?,
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

async fn complete_import(
    endpoint: &mut hibana::Endpoint<'_, GUEST>,
    pending: WasiImportPending<'_, '_>,
) -> ShellResult<()> {
    match pending.request()? {
        WasiImportRequest::FdWrite(request) => {
            endpoint.send::<protocol::FdWriteReqMsg>(&request).await?;
            let completion = endpoint.recv::<protocol::FdWriteRetMsg>().await?;
            pending.complete(WasiImportCompletion::FdWrite(completion))?;
        }
        WasiImportRequest::FdWriteObject(request) => {
            endpoint
                .send::<protocol::FdWriteObjectReqMsg>(&request)
                .await?;
            let completion = endpoint.recv::<protocol::FdWriteObjectRetMsg>().await?;
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
            endpoint.send::<protocol::PathOpenReqMsg>(&request).await?;
            let completion = endpoint.recv::<protocol::PathOpenRetMsg>().await?;
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
            endpoint.send::<protocol::FdCloseReqMsg>(&request).await?;
            let completion = endpoint.recv::<protocol::FdCloseRetMsg>().await?;
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
    }
    Ok(())
}
