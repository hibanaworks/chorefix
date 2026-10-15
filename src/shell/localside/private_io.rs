use crate::shell::{ShellError, ShellResult, global::Exit, imp::private_fs::PrivateFs};
use hibana_wasip1_runtime::protocol;

pub async fn run<const ROLE: u8>(
    endpoint: &mut hibana::Endpoint<'_, ROLE>,
    io: &mut PrivateFs<'_>,
) -> ShellResult<i32> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            240 => return Ok(branch.recv::<Exit>().await?),
            protocol::LABEL_WASI_FD_READDIR => {
                let request = branch.recv::<protocol::FdReaddirReqMsg>().await?;
                let response = io.read_dir(request.0)?;
                endpoint
                    .send::<protocol::FdReaddirRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_READ => {
                let request = branch.recv::<protocol::FdReadReqMsg>().await?;
                let response = io.read(request.0).await?;
                endpoint.send::<protocol::FdReadRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_FD_WRITE => {
                let request = branch.recv::<protocol::FdWriteReqMsg>().await?;
                let response = io.write(request.0).await?;
                endpoint.send::<protocol::FdWriteRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_PATH_OPEN => {
                let request = branch.recv::<protocol::PathOpenReqMsg>().await?;
                let response = io.open(request.0)?;
                endpoint.send::<protocol::PathOpenRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                let response = io.stat(request.0);
                endpoint
                    .send::<protocol::FdFdstatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_FILESTAT_GET => {
                let request = branch.recv::<protocol::FdFilestatGetReqMsg>().await?;
                let response = io.file_stat(request.0);
                endpoint
                    .send::<protocol::FdFilestatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                let request = branch.recv::<protocol::FdPrestatGetReqMsg>().await?;
                let fd = request.0.fd();
                let response = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                    fd,
                    1,
                    if fd == 3 { 0 } else { 8 },
                ));
                endpoint
                    .send::<protocol::FdPrestatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                let request = branch.recv::<protocol::FdPrestatDirNameReqMsg>().await?;
                let fd = request.0.fd();
                let response = protocol::FdPrestatDirNameRet(protocol::FdPrestatDirNameDone::new(
                    fd,
                    b"/",
                    if fd == 3 { 0 } else { 8 },
                )?);
                endpoint
                    .send::<protocol::FdPrestatDirNameRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_WASI_FD_CLOSE => {
                let request = branch.recv::<protocol::FdCloseReqMsg>().await?;
                let response = io.close(request.0);
                endpoint.send::<protocol::FdCloseRetMsg>(&response).await?;
            }
            protocol::LABEL_WASI_PATH_FILESTAT_GET => {
                let request = branch.recv::<protocol::PathFilestatGetReqMsg>().await?;
                let response = io.path_stat(request.0)?;
                endpoint
                    .send::<protocol::PathFilestatGetRetMsg>(&response)
                    .await?;
            }
            protocol::LABEL_ENGINE_MEMORY_GROW => {
                let _request = branch.recv::<protocol::MemoryGrowReqMsg>().await?;
                let response = protocol::MemoryGrowRet(protocol::MemoryGrowDecision::reject());
                endpoint
                    .send::<protocol::MemoryGrowRetMsg>(&response)
                    .await?;
            }
            label => return Err(ShellError::message(format!("unexpected WASI row {label}"))),
        }
    }
}
