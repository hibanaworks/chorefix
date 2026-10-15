use super::super::{
    global::*,
    imp::{Import, Imports},
};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;
use hibana_wasip1_runtime::protocol;
use serde_json::Value;

pub async fn run(endpoint: &mut Endpoint<'_, DEPLOYER>, mut imports: Imports) -> ShellResult<()> {
    // Wait for a real service open. Private shell work has no request here.
    let (open, opened) = loop {
        match imports
            .recv()
            .await
            .ok_or_else(|| ShellError::message("guest closed before service open"))?
        {
            Import::Open(request, reply) => break (request, reply),
            Import::Write(request, reply) => {
                let _ = reply.send(protocol::FdWriteDoneRet(
                    protocol::FdWriteDone::new_with_errno(request.0.fd(), 0, 8),
                ));
            }
            Import::Close(request, reply) => {
                let _ = reply.send((
                    protocol::FdClosedRet(protocol::FdClosed::new_with_errno(request.fd(), 8)),
                    Vec::new(),
                ));
            }
        }
    };
    endpoint.recv::<TestsPassed>().await?;
    endpoint.send::<Acquire>(&()).await?;
    endpoint.recv::<Granted>().await?;
    endpoint.send::<Open>(&open).await?;
    let response = endpoint.recv::<Opened>().await?;
    let _ = opened.send(response);
    loop {
        match imports
            .recv()
            .await
            .ok_or_else(|| ShellError::message("guest closed before service commit"))?
        {
            Import::Open(request, reply) => {
                endpoint.send::<Open>(&request).await?;
                let response = endpoint.recv::<Opened>().await?;
                let _ = reply.send(response);
            }
            Import::Write(request, reply) => {
                endpoint.send::<Write>(&request).await?;
                let response = endpoint.recv::<Written>().await?;
                let _ = reply.send(response);
            }
            Import::Close(request, reply) => {
                endpoint
                    .send::<Commit>(&protocol::FdCloseReq(request))
                    .await?;
                let branch = endpoint.offer().await?;
                match branch.label() {
                    8 => {
                        let bytes = branch.recv::<Committed<'_>>().await?.to_vec();
                        let value: Value = serde_json::from_slice(&bytes)
                            .map_err(|error| ShellError::message(error.to_string()))?;
                        let _ = reply.send((
                            protocol::FdClosedRet(protocol::FdClosed::new(request.fd())),
                            bytes,
                        ));
                        if value["success"] != true {
                            return Err(ShellError::message(format!(
                                "domain operation failed: {value}"
                            )));
                        }
                        break;
                    }
                    9 => {
                        let bytes = branch.recv::<Rejected<'_>>().await?.to_vec();
                        let _ = reply.send((
                            protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                                request.fd(),
                                28,
                            )),
                            bytes,
                        ));
                    }
                    _ => return Err(ShellError::message("unexpected service close reply")),
                }
            }
        }
    }
    // The affine service receiver is consumed. A later syscall receives a
    // closed-direction error, rather than another permission from a done flag.
    drop(imports);
    endpoint.send::<Return>(&()).await?;
    endpoint.recv::<Returned>().await?;
    endpoint.send::<DeployedToDeveloper>(&()).await?;
    endpoint.send::<DeployedToTester>(&()).await?;
    Ok(())
}
