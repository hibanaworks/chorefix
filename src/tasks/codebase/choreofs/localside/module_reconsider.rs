//! Return each Endpoint before observing and serving the next enabled choice.
use super::super::global::{Loan, ModuleTestResult};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;

#[hibana_runtime::localside]
pub(crate) async fn run<
    'r,
    const ROLE: u8,
    const RESOURCE: u8,
    const FIRST_OTHER_OWNER: u8,
    const FIRST_OTHER_RESOURCE: u8,
    const SECOND_OTHER_OWNER: u8,
    const SECOND_OTHER_RESOURCE: u8,
>(
    endpoint: &mut Endpoint<'r, ROLE>,
    mut resource: Endpoint<'r, RESOURCE>,
) -> ShellResult<()> {
    // Both assigned developers get their first commit before results are read.
    for _ in 0..2 {
        endpoint.send::<Loan<RESOURCE>>(resource).await?;
        resource = endpoint.recv::<Loan<RESOURCE>>().await?;
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == FIRST_OTHER_OWNER => {
                let _result = branch.recv::<ModuleTestResult<FIRST_OTHER_OWNER>>().await?;
                endpoint.send::<Loan<RESOURCE>>(resource).await?;
                resource = endpoint.recv::<Loan<RESOURCE>>().await?;
            }
            label if label == SECOND_OTHER_OWNER => {
                let _result = branch
                    .recv::<ModuleTestResult<SECOND_OTHER_OWNER>>()
                    .await?;
                endpoint.send::<Loan<RESOURCE>>(resource).await?;
                resource = endpoint.recv::<Loan<RESOURCE>>().await?;
            }
            label if label == FIRST_OTHER_RESOURCE => {
                let _result = branch
                    .recv::<ModuleTestResult<FIRST_OTHER_RESOURCE>>()
                    .await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        label if label == SECOND_OTHER_OWNER => {
                            let _result = branch
                                .recv::<ModuleTestResult<SECOND_OTHER_OWNER>>()
                                .await?;
                            endpoint.send::<Loan<RESOURCE>>(resource).await?;
                            resource = endpoint.recv::<Loan<RESOURCE>>().await?;
                        }
                        label if label == SECOND_OTHER_RESOURCE => {
                            let _result = branch
                                .recv::<ModuleTestResult<SECOND_OTHER_RESOURCE>>()
                                .await?;
                            return Ok(());
                        }
                        _ => return Err(ShellError::message("unexpected second developer result")),
                    }
                }
            }
            label if label == SECOND_OTHER_RESOURCE => {
                let _result = branch
                    .recv::<ModuleTestResult<SECOND_OTHER_RESOURCE>>()
                    .await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        label if label == FIRST_OTHER_OWNER => {
                            let _result =
                                branch.recv::<ModuleTestResult<FIRST_OTHER_OWNER>>().await?;
                            endpoint.send::<Loan<RESOURCE>>(resource).await?;
                            resource = endpoint.recv::<Loan<RESOURCE>>().await?;
                        }
                        label if label == FIRST_OTHER_RESOURCE => {
                            let _result = branch
                                .recv::<ModuleTestResult<FIRST_OTHER_RESOURCE>>()
                                .await?;
                            return Ok(());
                        }
                        _ => return Err(ShellError::message("unexpected first developer result")),
                    }
                }
            }
            _ => return Err(ShellError::message("unexpected developer test result")),
        }
    }
}
