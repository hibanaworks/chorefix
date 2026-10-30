//! Both module endpoints carry the commit request before the shared effect.
use super::super::{
    global::*,
    input::assignment,
    wire::{COMMIT, Close, DESIGN, IMPLEMENT, TEST},
};
use crate::shell::{ShellError, ShellResult, imp::service::Domain};
use hibana::Endpoint;

pub(crate) async fn run<
    const TASK: u8,
    const ROLE: u8,
    const FIRST: u8,
    const SECOND: u8,
    const FIRST_OWNER: u8,
    const SECOND_OWNER: u8,
>(
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &impl Domain,
    test_result: &super::super::input::TestResult,
) -> ShellResult<()> {
    let input = assignment::<TASK, ROLE>();

    let request = endpoint.recv::<DesignRequest<TASK, ROLE>>().await?;
    let arguments = serde_json::from_slice(&request.as_bytes()[1..])
        .map_err(|error| ShellError::message(error.to_string()))?;
    let result = Close::<TASK, ROLE, DESIGN>::objects()
        .choreofs()
        .fd_close(request);
    if result.0.errno() != 0 {
        return Err(ShellError::message("design object could not materialize"));
    }
    let result = domain
        .execute(input.agent, "design_feature", arguments)
        .await
        .map_err(ShellError::message)?;
    if result["success"] != true {
        return Err(ShellError::message(format!(
            "domain design_feature failed: {result}"
        )));
    }
    let bytes =
        serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
    let response = bytes.as_slice();
    endpoint.send::<DesignCompleted<'_>>(&response).await?;

    loop {
        let request = endpoint.recv::<ImplementRequest<TASK, ROLE>>().await?;
        let arguments = serde_json::from_slice(&request.as_bytes()[1..])
            .map_err(|error| ShellError::message(error.to_string()))?;
        let result = Close::<TASK, ROLE, IMPLEMENT>::objects()
            .choreofs()
            .fd_close(request);
        if result.0.errno() != 0 {
            return Err(ShellError::message(
                "implementation object could not materialize",
            ));
        }
        let result = domain
            .execute(input.agent, "implement_code", arguments)
            .await
            .map_err(ShellError::message)?;
        if result["success"] != true {
            return Err(ShellError::message(format!(
                "domain implement_code failed: {result}"
            )));
        }
        let bytes =
            serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
        let response = bytes.as_slice();
        endpoint.send::<ImplementCompleted<'_>>(&response).await?;

        let arguments = {
            let [mut first, mut second] = endpoint.par()?;
            let (first, second) = tokio::try_join!(
                first.recv::<CommitRequest<TASK, ROLE, FIRST>>(),
                second.recv::<CommitRequest<TASK, ROLE, SECOND>>()
            )?;
            if first.as_bytes() != second.as_bytes() {
                return Err(ShellError::message(
                    "module requests name different commits",
                ));
            }
            let arguments = serde_json::from_slice(&first.as_bytes()[1..])
                .map_err(|error| ShellError::message(error.to_string()))?;
            let objects = Close::<TASK, ROLE, COMMIT>::objects();
            let fs = objects.choreofs();
            let first = fs.fd_close(first);
            let second = fs.fd_close(second);
            if first.0.errno() != 0 || second.0.errno() != 0 {
                return Err(ShellError::message("commit object could not materialize"));
            }
            arguments
        };
        let result = domain
            .execute(input.agent, "commit_changes", arguments)
            .await
            .map_err(ShellError::message)?;
        if result["success"] != true {
            return Err(ShellError::message(format!(
                "domain commit_changes failed: {result}"
            )));
        }
        let bytes =
            serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
        {
            let [mut first, mut second] = endpoint.par()?;
            let payload = bytes.as_slice();
            tokio::try_join!(
                first.send::<CommitCompleted<'_, FIRST>>(&payload),
                second.send::<CommitCompleted<'_, SECOND>>(&payload)
            )?;
        }

        let request = endpoint.recv::<TestRequest<TASK, ROLE>>().await?;
        let arguments = serde_json::from_slice(&request.as_bytes()[1..])
            .map_err(|error| ShellError::message(error.to_string()))?;
        let result = Close::<TASK, ROLE, TEST>::objects()
            .choreofs()
            .fd_close(request);
        if result.0.errno() != 0 {
            return Err(ShellError::message("test object could not materialize"));
        }
        let result = domain
            .execute(input.agent, "run_local_tests", arguments)
            .await
            .map_err(ShellError::message)?;

        let bytes =
            serde_json::to_vec(&result).map_err(|error| ShellError::message(error.to_string()))?;
        test_result.record(result);
        let branch = endpoint.offer().await?;
        match branch.label() {
            183 => {
                drop(branch);
                let [mut vm, mut first_owner, mut second_owner] = endpoint.par()?;
                let payload = bytes.as_slice();
                tokio::try_join!(
                    vm.send::<TestPassed<'_>>(&payload),
                    first_owner.send::<ModuleTestResult<'_, SECOND>>(&payload),
                    second_owner.send::<ModuleTestResult<'_, FIRST>>(&payload)
                )?;
            }
            184 => {
                drop(branch);
                let [mut vm, mut first_owner, mut second_owner] = endpoint.par()?;
                let payload = bytes.as_slice();
                tokio::try_join!(
                    vm.send::<TestFailed<'_>>(&payload),
                    first_owner.send::<ModuleTestResult<'_, SECOND_OWNER>>(&payload),
                    second_owner.send::<ModuleTestResult<'_, FIRST_OWNER>>(&payload)
                )?;
                continue;
            }
            _ => return Err(ShellError::message("unexpected test result branch")),
        }
        let status = endpoint.recv::<ServiceExit>().await?;
        if status != 0 {
            return Err(ShellError::message("developer guest failed"));
        }
        return Ok(());
    }
}
