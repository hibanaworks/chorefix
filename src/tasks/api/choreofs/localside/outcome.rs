pub mod admin {
    use super::super::super::{
        global::*,
        imp::{FileReply, OutcomeFile},
    };
    use crate::shell::{ShellError, ShellResult};
    use hibana::Endpoint;
    use serde_json::Value;
    pub async fn admin(
        endpoint: &mut Endpoint<'_, ADMIN_OUTCOME>,
        file: &mut OutcomeFile,
    ) -> ShellResult<()> {
        let [mut results, mut metadata] = endpoint.par()?;
        file.bytes = loop {
            let received = {
                let offered = tokio::select! {
                    branch = results.offer() => Ok(branch),
                    branch = metadata.offer() => Err(branch),
                };
                match offered {
                    Ok(branch) => {
                        let branch = branch?;
                        let bytes = match branch.label() {
                            50 => {
                                let bytes = branch.recv::<PassedAdmin<'_>>().await?.to_vec();
                                let result: Value = serde_json::from_slice(&bytes)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                if result["success"] != true {
                                    return Err(ShellError::message(
                                        "pass label differs from test result",
                                    ));
                                }
                                bytes
                            }
                            60 => {
                                let bytes = branch.recv::<FailedAdmin<'_>>().await?.to_vec();
                                let result: Value = serde_json::from_slice(&bytes)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                if result["success"] != false {
                                    return Err(ShellError::message(
                                        "fail label differs from test result",
                                    ));
                                }
                                bytes
                            }
                            _ => return Err(ShellError::message("unexpected result notification")),
                        };
                        Ok(bytes)
                    }
                    Err(branch) => {
                        let branch = branch?;
                        if branch.label() != 160 {
                            return Err(ShellError::message(
                                "guest exited before receiving its result",
                            ));
                        }
                        Err(file.path_stat(branch.recv::<OutcomePathStat>().await?))
                    }
                }
            };
            match received {
                Ok(bytes) => break bytes,
                Err(response) => metadata.send::<OutcomePathStatDone>(&response).await?,
            }
        };
        let opened = loop {
            let received = {
                let offered = tokio::select! {
                    request = results.recv::<OutcomeOpen>() => Ok(request),
                    branch = metadata.offer() => Err(branch),
                };
                match offered {
                    Ok(request) => Ok(file.open(request?)?),
                    Err(branch) => {
                        let branch = branch?;
                        if branch.label() != 160 {
                            return Err(ShellError::message(
                                "guest exited before opening its result",
                            ));
                        }
                        Err(file.path_stat(branch.recv::<OutcomePathStat>().await?))
                    }
                }
            };
            match received {
                Ok(response) => break response,
                Err(response) => metadata.send::<OutcomePathStatDone>(&response).await?,
            }
        };
        results.send::<OutcomeOpened>(&opened).await?;
        loop {
            let response = {
                let offered = tokio::select! {
                    branch = results.offer() => Ok(branch),
                    branch = metadata.offer() => Err(branch),
                };
                match offered {
                    Ok(branch) => {
                        let branch = branch?;
                        match branch.label() {
                            150 => {
                                FileReply::Opened(file.open(branch.recv::<OutcomeOpen>().await?)?)
                            }
                            152 => FileReply::Read(file.read(branch.recv::<OutcomeRead>().await?)?),
                            154 => FileReply::Stat(file.stat(branch.recv::<OutcomeStat>().await?)),
                            156 => FileReply::FileStat(
                                file.file_stat(branch.recv::<OutcomeFileStat>().await?),
                            ),
                            158 => {
                                FileReply::Closed(file.close(branch.recv::<OutcomeClose>().await?))
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected result-file operation",
                                ));
                            }
                        }
                    }
                    Err(branch) => {
                        let branch = branch?;
                        match branch.label() {
                            160 => FileReply::PathStat(
                                file.path_stat(branch.recv::<OutcomePathStat>().await?),
                            ),
                            242 => FileReply::Exit(branch.recv::<OutcomeExit>().await?),
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected file metadata operation",
                                ));
                            }
                        }
                    }
                }
            };
            match response {
                FileReply::Opened(response) => results.send::<OutcomeOpened>(&response).await?,
                FileReply::Read(response) => results.send::<OutcomeReadDone>(&response).await?,
                FileReply::Stat(response) => results.send::<OutcomeStatDone>(&response).await?,
                FileReply::FileStat(response) => {
                    results.send::<OutcomeFileStatDone>(&response).await?
                }
                FileReply::Closed(response) => results.send::<OutcomeClosed>(&response).await?,
                FileReply::PathStat(response) => {
                    metadata.send::<OutcomePathStatDone>(&response).await?
                }
                FileReply::Exit(status) => {
                    if status != 0 {
                        return Err(ShellError::message("result recipient guest failed"));
                    }
                    return Ok(());
                }
            }
        }
    }
}
pub mod backend {
    use super::super::super::{
        global::*,
        imp::{FileReply, OutcomeFile},
    };
    use crate::shell::{ShellError, ShellResult};
    use hibana::Endpoint;
    use serde_json::Value;
    pub async fn backend(
        endpoint: &mut Endpoint<'_, BACKEND_OUTCOME>,
        file: &mut OutcomeFile,
    ) -> ShellResult<()> {
        let [mut results, mut metadata] = endpoint.par()?;
        file.bytes = loop {
            let received = {
                let offered = tokio::select! {
                    branch = results.offer() => Ok(branch),
                    branch = metadata.offer() => Err(branch),
                };
                match offered {
                    Ok(branch) => {
                        let branch = branch?;
                        let bytes = match branch.label() {
                            51 => {
                                let bytes = branch.recv::<PassedBackend<'_>>().await?.to_vec();
                                let result: Value = serde_json::from_slice(&bytes)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                if result["success"] != true {
                                    return Err(ShellError::message(
                                        "pass label differs from test result",
                                    ));
                                }
                                bytes
                            }
                            61 => {
                                let bytes = branch.recv::<FailedBackend<'_>>().await?.to_vec();
                                let result: Value = serde_json::from_slice(&bytes)
                                    .map_err(|e| ShellError::message(e.to_string()))?;
                                if result["success"] != false {
                                    return Err(ShellError::message(
                                        "fail label differs from test result",
                                    ));
                                }
                                bytes
                            }
                            _ => return Err(ShellError::message("unexpected result notification")),
                        };
                        Ok(bytes)
                    }
                    Err(branch) => {
                        let branch = branch?;
                        if branch.label() != 160 {
                            return Err(ShellError::message(
                                "guest exited before receiving its result",
                            ));
                        }
                        Err(file.path_stat(branch.recv::<OutcomePathStat>().await?))
                    }
                }
            };
            match received {
                Ok(bytes) => break bytes,
                Err(response) => metadata.send::<OutcomePathStatDone>(&response).await?,
            }
        };
        let opened = loop {
            let received = {
                let offered = tokio::select! {
                    request = results.recv::<OutcomeOpen>() => Ok(request),
                    branch = metadata.offer() => Err(branch),
                };
                match offered {
                    Ok(request) => Ok(file.open(request?)?),
                    Err(branch) => {
                        let branch = branch?;
                        if branch.label() != 160 {
                            return Err(ShellError::message(
                                "guest exited before opening its result",
                            ));
                        }
                        Err(file.path_stat(branch.recv::<OutcomePathStat>().await?))
                    }
                }
            };
            match received {
                Ok(response) => break response,
                Err(response) => metadata.send::<OutcomePathStatDone>(&response).await?,
            }
        };
        results.send::<OutcomeOpened>(&opened).await?;
        loop {
            let response = {
                let offered = tokio::select! {
                    branch = results.offer() => Ok(branch),
                    branch = metadata.offer() => Err(branch),
                };
                match offered {
                    Ok(branch) => {
                        let branch = branch?;
                        match branch.label() {
                            150 => {
                                FileReply::Opened(file.open(branch.recv::<OutcomeOpen>().await?)?)
                            }
                            152 => FileReply::Read(file.read(branch.recv::<OutcomeRead>().await?)?),
                            154 => FileReply::Stat(file.stat(branch.recv::<OutcomeStat>().await?)),
                            156 => FileReply::FileStat(
                                file.file_stat(branch.recv::<OutcomeFileStat>().await?),
                            ),
                            158 => {
                                FileReply::Closed(file.close(branch.recv::<OutcomeClose>().await?))
                            }
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected result-file operation",
                                ));
                            }
                        }
                    }
                    Err(branch) => {
                        let branch = branch?;
                        match branch.label() {
                            160 => FileReply::PathStat(
                                file.path_stat(branch.recv::<OutcomePathStat>().await?),
                            ),
                            242 => FileReply::Exit(branch.recv::<OutcomeExit>().await?),
                            _ => {
                                return Err(ShellError::message(
                                    "unexpected file metadata operation",
                                ));
                            }
                        }
                    }
                }
            };
            match response {
                FileReply::Opened(response) => results.send::<OutcomeOpened>(&response).await?,
                FileReply::Read(response) => results.send::<OutcomeReadDone>(&response).await?,
                FileReply::Stat(response) => results.send::<OutcomeStatDone>(&response).await?,
                FileReply::FileStat(response) => {
                    results.send::<OutcomeFileStatDone>(&response).await?
                }
                FileReply::Closed(response) => results.send::<OutcomeClosed>(&response).await?,
                FileReply::PathStat(response) => {
                    metadata.send::<OutcomePathStatDone>(&response).await?
                }
                FileReply::Exit(status) => {
                    if status != 0 {
                        return Err(ShellError::message("result recipient guest failed"));
                    }
                    return Ok(());
                }
            }
        }
    }
}
