//! The single REPO owner materializes every admitted shared operation.
use super::super::global::*;
use crate::shell::{
    ShellError, ShellResult,
    imp::service::{Domain, File, Validation},
};
use hibana::Endpoint;
use serde_json::json;

pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, REPO>,
    domain: &D,
    implement_a: &Validation,
    merge_a: &Validation,
    implement_b: &Validation,
    merge_b: &Validation,
) -> ShellResult<()> {
    let mut a_implementation = File::new(b"service/implement_feature", SERVICE_FD);
    let mut a_merge = File::new(b"service/merge_code", SERVICE_FD);
    let mut b_implementation = File::new(b"service/implement_feature", SERVICE_FD);
    let mut b_merge = File::new(b"service/merge_code", SERVICE_FD);
    loop {
        let branch = endpoint
            .offer()
            .await
            .map_err(|e| ShellError::message(format!("repo offer: {e:?}")))?;
        match branch.label() {
            110 => {
                let request = branch.recv::<OpenImplementA>().await?;
                endpoint
                    .send::<OpenedImplementA>(&a_implementation.open(request))
                    .await?;
            }
            112 => {
                let request = branch.recv::<WriteImplementA>().await?;
                endpoint
                    .send::<WrittenImplementA>(&a_implementation.write(request))
                    .await?;
            }
            114 => {
                let request = branch.recv::<CommitImplementA>().await?;
                implement_a.record(
                    a_implementation
                        .arguments(request.0, "feature_name")
                        .and_then(|value| {
                            if value["feature_name"] == FEATURE_A {
                                Ok(value)
                            } else {
                                Err(ShellError::message(
                                    "operation target differs from this service",
                                ))
                            }
                        }),
                );
            }
            115 => {
                branch.recv::<MaterializeImplementA>().await?;
                let result = domain
                    .execute("DEVELOPER_A", "implement_feature", implement_a.take()?)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                endpoint
                    .send::<CompletedImplementA<'_>>(&bytes.as_slice())
                    .await?;
                if result["success"] != true {
                    return Err(ShellError::message(format!(
                        "terminal REPO operation failure: {result}"
                    )));
                }
            }
            117 => {
                let bytes = serde_json::to_vec(
                    &json!({"status":"invalid_arguments", "error":implement_a.rejection()?}),
                )
                .map_err(|e| ShellError::message(e.to_string()))?;
                branch
                    .send::<RejectedImplementA<'_>>(&bytes.as_slice())
                    .await?;
            }
            130 => {
                let request = branch.recv::<OpenMergeA>().await?;
                endpoint
                    .send::<OpenedMergeA>(&a_merge.open(request))
                    .await?;
            }
            132 => {
                let request = branch.recv::<WriteMergeA>().await?;
                endpoint
                    .send::<WrittenMergeA>(&a_merge.write(request))
                    .await?;
            }
            134 => {
                let request = branch.recv::<CommitMergeA>().await?;
                merge_a.record(a_merge.arguments(request.0, "branch").and_then(|value| {
                    if value["branch"] == MERGE_BRANCH_A {
                        Ok(value)
                    } else {
                        Err(ShellError::message(
                            "operation target differs from this service",
                        ))
                    }
                }));
            }
            135 => {
                branch.recv::<MaterializeMergeA>().await?;
                let result = domain
                    .execute("DEVELOPER_A", "merge_code", merge_a.take()?)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                endpoint
                    .send::<CompletedMergeA<'_>>(&bytes.as_slice())
                    .await?;
                if result["success"] != true {
                    return Err(ShellError::message(format!(
                        "terminal REPO operation failure: {result}"
                    )));
                }
            }
            137 => {
                let bytes = serde_json::to_vec(
                    &json!({"status":"invalid_arguments", "error":merge_a.rejection()?}),
                )
                .map_err(|e| ShellError::message(e.to_string()))?;
                branch.send::<RejectedMergeA<'_>>(&bytes.as_slice()).await?;
            }
            150 => {
                let request = branch.recv::<OpenImplementB>().await?;
                endpoint
                    .send::<OpenedImplementB>(&b_implementation.open(request))
                    .await?;
            }
            152 => {
                let request = branch.recv::<WriteImplementB>().await?;
                endpoint
                    .send::<WrittenImplementB>(&b_implementation.write(request))
                    .await?;
            }
            154 => {
                let request = branch.recv::<CommitImplementB>().await?;
                implement_b.record(
                    b_implementation
                        .arguments(request.0, "feature_name")
                        .and_then(|value| {
                            if value["feature_name"] == FEATURE_B {
                                Ok(value)
                            } else {
                                Err(ShellError::message(
                                    "operation target differs from this service",
                                ))
                            }
                        }),
                );
            }
            155 => {
                branch.recv::<MaterializeImplementB>().await?;
                let result = domain
                    .execute("DEVELOPER_B", "implement_feature", implement_b.take()?)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                endpoint
                    .send::<CompletedImplementB<'_>>(&bytes.as_slice())
                    .await?;
                if result["success"] != true {
                    return Err(ShellError::message(format!(
                        "terminal REPO operation failure: {result}"
                    )));
                }
            }
            157 => {
                let bytes = serde_json::to_vec(
                    &json!({"status":"invalid_arguments", "error":implement_b.rejection()?}),
                )
                .map_err(|e| ShellError::message(e.to_string()))?;
                branch
                    .send::<RejectedImplementB<'_>>(&bytes.as_slice())
                    .await?;
            }
            170 => {
                let request = branch.recv::<OpenMergeB>().await?;
                endpoint
                    .send::<OpenedMergeB>(&b_merge.open(request))
                    .await?;
            }
            172 => {
                let request = branch.recv::<WriteMergeB>().await?;
                endpoint
                    .send::<WrittenMergeB>(&b_merge.write(request))
                    .await?;
            }
            174 => {
                let request = branch.recv::<CommitMergeB>().await?;
                merge_b.record(b_merge.arguments(request.0, "branch").and_then(|value| {
                    if value["branch"] == MERGE_BRANCH_B {
                        Ok(value)
                    } else {
                        Err(ShellError::message(
                            "operation target differs from this service",
                        ))
                    }
                }));
            }
            175 => {
                branch.recv::<MaterializeMergeB>().await?;
                let result = domain
                    .execute("DEVELOPER_B", "merge_code", merge_b.take()?)
                    .await
                    .map_err(ShellError::message)?;
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                endpoint
                    .send::<CompletedMergeB<'_>>(&bytes.as_slice())
                    .await?;
                if result["success"] != true {
                    return Err(ShellError::message(format!(
                        "terminal REPO operation failure: {result}"
                    )));
                }
            }
            177 => {
                let bytes = serde_json::to_vec(
                    &json!({"status":"invalid_arguments", "error":merge_b.rejection()?}),
                )
                .map_err(|e| ShellError::message(e.to_string()))?;
                branch.send::<RejectedMergeB<'_>>(&bytes.as_slice()).await?;
            }
            180 => {
                branch.recv::<ApproveRepoA>().await?;
                endpoint.send::<DecisionRecordedA>(&()).await?;
            }
            181 => {
                branch.recv::<ReworkRepoA>().await?;
                endpoint.send::<DecisionRecordedA>(&()).await?;
            }
            182 => {
                branch.recv::<ApproveRepoB>().await?;
                endpoint.send::<DecisionRecordedB>(&()).await?;
            }
            183 => {
                branch.recv::<ReworkRepoB>().await?;
                endpoint.send::<DecisionRecordedB>(&()).await?;
            }
            35 => {
                branch.recv::<RepoExit>().await?;
                return Ok(());
            }
            36 => {
                let status = branch.recv::<RepoFailed>().await?;
                return Err(ShellError::message(format!(
                    "reviewer guest failed: {status}"
                )));
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected REPO request: {label}"
                )));
            }
        }
    }
}
