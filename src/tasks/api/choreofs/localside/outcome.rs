pub mod admin {
    use super::super::super::{global::*, imp::OutcomeFile};
    use crate::shell::{ShellError, ShellResult};
    use hibana::Endpoint;
    use serde_json::Value;
    pub async fn admin(
        endpoint: &mut Endpoint<'_, ADMIN_OUTCOME>,
        file: &mut OutcomeFile,
    ) -> ShellResult<()> {
        let notification = endpoint.offer().await?;
        let bytes = match notification.label() {
            50 => {
                let b = notification.recv::<PassedAdmin<'_>>().await?.to_vec();
                let r: Value =
                    serde_json::from_slice(&b).map_err(|e| ShellError::message(e.to_string()))?;
                if r["success"] != true {
                    return Err(ShellError::message("pass label differs from test result"));
                }
                b
            }
            60 => {
                let b = notification.recv::<FailedAdmin<'_>>().await?.to_vec();
                let r: Value =
                    serde_json::from_slice(&b).map_err(|e| ShellError::message(e.to_string()))?;
                if r["success"] != false {
                    return Err(ShellError::message("fail label differs from test result"));
                }
                b
            }
            _ => return Err(ShellError::message("unexpected result notification")),
        };
        file.bytes.replace(bytes);
        // The first open cannot be answered until the projected notification arrives.
        let request = endpoint.recv::<OutcomeOpen>().await?;
        endpoint.send::<OutcomeOpened>(&file.open(request)?).await?;
        loop {
            let branch = endpoint.offer().await?;
            match branch.label() {
                150 => {
                    let r = branch.recv::<OutcomeOpen>().await?;
                    endpoint.send::<OutcomeOpened>(&file.open(r)?).await?;
                }
                152 => {
                    let r = branch.recv::<OutcomeRead>().await?;
                    endpoint.send::<OutcomeReadDone>(&file.read(r)?).await?;
                }
                154 => {
                    let r = branch.recv::<OutcomeStat>().await?;
                    endpoint.send::<OutcomeStatDone>(&file.stat(r)).await?;
                }
                156 => {
                    let r = branch.recv::<OutcomeFileStat>().await?;
                    endpoint
                        .send::<OutcomeFileStatDone>(&file.file_stat(r))
                        .await?;
                }
                158 => {
                    let r = branch.recv::<OutcomeClose>().await?;
                    endpoint.send::<OutcomeClosed>(&file.close(r)).await?;
                }
                242 => {
                    let status = branch.recv::<OutcomeExit>().await?;
                    if status != 0 {
                        return Err(ShellError::message("result recipient guest failed"));
                    }
                    return Ok(());
                }
                _ => return Err(ShellError::message("unexpected result-file operation")),
            }
        }
    }
}
pub mod backend {
    use super::super::super::{global::*, imp::OutcomeFile};
    use crate::shell::{ShellError, ShellResult};
    use hibana::Endpoint;
    use serde_json::Value;
    pub async fn backend(
        endpoint: &mut Endpoint<'_, BACKEND_OUTCOME>,
        file: &mut OutcomeFile,
    ) -> ShellResult<()> {
        let notification = endpoint.offer().await?;
        let bytes = match notification.label() {
            51 => {
                let b = notification.recv::<PassedBackend<'_>>().await?.to_vec();
                let r: Value =
                    serde_json::from_slice(&b).map_err(|e| ShellError::message(e.to_string()))?;
                if r["success"] != true {
                    return Err(ShellError::message("pass label differs from test result"));
                }
                b
            }
            61 => {
                let b = notification.recv::<FailedBackend<'_>>().await?.to_vec();
                let r: Value =
                    serde_json::from_slice(&b).map_err(|e| ShellError::message(e.to_string()))?;
                if r["success"] != false {
                    return Err(ShellError::message("fail label differs from test result"));
                }
                b
            }
            _ => return Err(ShellError::message("unexpected result notification")),
        };
        file.bytes.replace(bytes);
        // The first open cannot be answered until the projected notification arrives.
        let request = endpoint.recv::<OutcomeOpen>().await?;
        endpoint.send::<OutcomeOpened>(&file.open(request)?).await?;
        loop {
            let branch = endpoint.offer().await?;
            match branch.label() {
                150 => {
                    let r = branch.recv::<OutcomeOpen>().await?;
                    endpoint.send::<OutcomeOpened>(&file.open(r)?).await?;
                }
                152 => {
                    let r = branch.recv::<OutcomeRead>().await?;
                    endpoint.send::<OutcomeReadDone>(&file.read(r)?).await?;
                }
                154 => {
                    let r = branch.recv::<OutcomeStat>().await?;
                    endpoint.send::<OutcomeStatDone>(&file.stat(r)).await?;
                }
                156 => {
                    let r = branch.recv::<OutcomeFileStat>().await?;
                    endpoint
                        .send::<OutcomeFileStatDone>(&file.file_stat(r))
                        .await?;
                }
                158 => {
                    let r = branch.recv::<OutcomeClose>().await?;
                    endpoint.send::<OutcomeClosed>(&file.close(r)).await?;
                }
                242 => {
                    let status = branch.recv::<OutcomeExit>().await?;
                    if status != 0 {
                        return Err(ShellError::message("result recipient guest failed"));
                    }
                    return Ok(());
                }
                _ => return Err(ShellError::message("unexpected result-file operation")),
            }
        }
    }
}
