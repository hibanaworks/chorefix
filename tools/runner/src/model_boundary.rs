//! Live connection qualification with original inputs and actual WASI guests.
use crate::{
    evidence::{Result, save},
    model,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_participants_use_independent_wash_guests() -> Result<()> {
    subscription_case("9E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_five_philosophers_use_independent_wash_guests() -> Result<()> {
    subscription_case("9M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_seven_philosophers_use_independent_wash_guests() -> Result<()> {
    subscription_case("9H", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants, actual WASI guests and upstream test failures"]
async fn subscription_developers_reconsider_through_seven_wash_guests() -> Result<()> {
    subscription_case("1H", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants through actual API-task resource Endpoints"]
async fn subscription_api_participants_receive_the_actual_successful_result() -> Result<()> {
    subscription_case("8E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and preserves the original terminal test failure"]
async fn subscription_api_participants_receive_the_actual_failed_result() -> Result<()> {
    subscription_case("8E", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and actual CI/CD resource Endpoints"]
async fn subscription_cicd_participants_use_resource_endpoints() -> Result<()> {
    subscription_case("16E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants, actual repository Endpoints and upstream review failures"]
async fn subscription_review_participants_reconsider_with_resource_endpoints() -> Result<()> {
    subscription_case("6E", 2).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses three Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_three_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2E", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses five Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_five_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2M", 0).await
}

#[hibana_runtime::main]
#[test]
#[ignore = "uses seven Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_seven_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2H", 0).await
}

// This harness starts the model connections and records their observations.
// Task permissions and Endpoint operations stay in the application's localsides.
async fn subscription_case(task: &str, depth: u8) -> Result<()> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_root = source_root.canonicalize().map_err(|error| {
        format!(
            "resolve experiment source {}: {error}",
            source_root.display()
        )
    })?;
    std::env::set_current_dir(&source_root)
        .map_err(|error| format!("open experiment source {}: {error}", source_root.display()))?;
    let directory = PathBuf::from(
        std::env::var_os("CHOREOFIX_EVIDENCE_ROOT").ok_or("CHOREOFIX_EVIDENCE_ROOT is required")?,
    )
    .join(format!(
        "subscription-{task}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let result = crate::run(task, depth, &directory).await?;
    println!("{}", directory.display());
    if !result["error"].is_null() {
        return Err("model run failed; see the preserved outcomes".into());
    }
    let outcomes = result["outcomes"].as_array().ok_or("run outcomes")?;
    let metadata: Value = serde_json::from_slice(&fs::read(
        directory
            .join("source/vendor/tracefix/benchmark/descriptions")
            .join(task)
            .join("metadata.json"),
    )?)?;
    let participant_count = metadata["agents"]
        .as_array()
        .ok_or("original participant set")?
        .len();
    let participants: Vec<_> = outcomes
        .iter()
        .filter(|o| o["kind"] == "model_participant")
        .collect();
    if participants.len() != participant_count
        || outcomes.len() != participant_count + 1
        || participants.iter().any(|p| {
            p["result"]["status"] != "completed"
                || p["result"]["returned_model"] != model::MODEL
                || p["result"]["returned_effort"] != model::EFFORT
                || p["result"]["usage"]["totalTokens"].as_u64().is_none()
        })
    {
        return Err(
            "subscription participants did not complete; see the preserved outcomes".into(),
        );
    }
    let runtime = outcomes
        .iter()
        .find(|o| o["kind"] == "guest_runtime")
        .ok_or("runtime outcome")?;
    let expected_complete = task != "8E" || depth == 0;
    if runtime["result"]["score"]["complete"] != expected_complete
        || runtime["result"]["score"]["violations"] != json!([])
        || runtime["result"]["runtime_exit_code"] != 0
        || runtime["result"]["protocol"]["status"] != "complete"
        || runtime["result"]["backend_errors"] != json!([])
    {
        return Err("actual WASI model task did not complete; see the preserved outcomes".into());
    }
    let audited = crate::audit::run(&directory)?;
    save(&directory.join("audit.json"), &audited)?;
    let audit = &audited["protocol"];
    if audit["status"] != "passed"
        || (task == "6E"
            && (audit["actual_resource_endpoint_requests"] != 4 + u64::from(depth)
                || audit["actual_resource_endpoint_returns"] != 4 + u64::from(depth)
                || audit["actual_review_requests"] != 2 + u64::from(depth)))
        || (task == "16E"
            && (audit["actual_resource_endpoint_requests"] != 3
                || audit["actual_resource_endpoint_returns"] != 3))
        || (matches!(task, "9E" | "9M" | "9H")
            && audit["actual_resource_endpoint_returns"] != 2 * participant_count)
        || (task == "1H"
            && !audit["attempts"]
                .as_object()
                .ok_or("revision attempts")?
                .values()
                .all(|visits| visits.as_u64() == Some(3)))
        || (task == "8E"
            && (audit["actual_resource_endpoint_requests"] != 2
                || audit["actual_resource_endpoint_returns"] != 2
                || audit["simulator_complete"] != expected_complete))
        || (matches!(task, "2E" | "2M" | "2H")
            && (audit["actual_resource_endpoint_requests"] != 2 * participant_count
                || audit["actual_resource_endpoint_returns"] != 2 * participant_count
                || audit["simulator_complete"] != expected_complete))
    {
        return Err("delegation audit did not confirm all actual returns".into());
    }
    Ok(())
}
