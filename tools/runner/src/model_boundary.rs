//! Live connection qualification with original inputs and actual WASI guests.
use crate::{
    evidence::{Result, Trace, digest, save},
    model, oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{process::Command, task::JoinSet};

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses three Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_participants_use_independent_wash_guests() -> Result<()> {
    subscription_case("9E", 0).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses five Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_five_philosophers_use_independent_wash_guests() -> Result<()> {
    subscription_case("9M", 0).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses seven Luna xhigh participants through the existing ChatGPT subscription and actual WASI guests"]
async fn subscription_seven_philosophers_use_independent_wash_guests() -> Result<()> {
    subscription_case("9H", 0).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses seven Luna xhigh participants, actual WASI guests and upstream test failures"]
async fn subscription_developers_reconsider_through_seven_wash_guests() -> Result<()> {
    subscription_case("1H", 2).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses three Luna xhigh participants through actual API-task resource Endpoints"]
async fn subscription_api_participants_receive_the_actual_successful_result() -> Result<()> {
    subscription_case("8E", 0).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses three Luna xhigh participants and preserves the original terminal test failure"]
async fn subscription_api_participants_receive_the_actual_failed_result() -> Result<()> {
    subscription_case("8E", 2).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses three Luna xhigh participants and actual CI/CD resource Endpoints"]
async fn subscription_cicd_participants_use_resource_endpoints() -> Result<()> {
    subscription_case("16E", 0).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses three Luna xhigh participants, actual repository Endpoints and upstream review failures"]
async fn subscription_review_participants_reconsider_with_resource_endpoints() -> Result<()> {
    subscription_case("6E", 2).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses three Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_three_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2E", 0).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses five Luna xhigh participants and actual section Endpoints through WASI"]
async fn subscription_writing_five_participants_use_section_endpoints() -> Result<()> {
    subscription_case("2M", 0).await
}

#[tokio::test(flavor = "current_thread")]
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
    fs::create_dir_all(&directory).map_err(|error| {
        format!(
            "create experiment evidence {}: {error}",
            directory.display()
        )
    })?;
    let (runtime_variable, participant_count) = match task {
        "9E" => ("CHOREOFIX_PHILOSOPHERS_WASI_BINARY", 3),
        "9M" => ("CHOREOFIX_PHILOSOPHERS_WASI_BINARY", 5),
        "9H" => ("CHOREOFIX_PHILOSOPHERS_WASI_BINARY", 7),
        "1H" => ("CHOREOFIX_CODEBASE_WASI_BINARY", 7),
        "8E" => ("CHOREOFIX_API_WASI_BINARY", 3),
        "16E" => ("CHOREOFIX_CICD_WASI_BINARY", 3),
        "6E" => ("CHOREOFIX_REVIEW_WASI_BINARY", 3),
        "2E" => ("CHOREOFIX_WRITING_WASI_BINARY", 3),
        "2M" => ("CHOREOFIX_WRITING_WASI_BINARY", 5),
        "2H" => ("CHOREOFIX_WRITING_WASI_BINARY", 7),
        _ => return Err("unqualified model preflight task".into()),
    };
    let runtime = PathBuf::from(std::env::var_os(runtime_variable).ok_or(runtime_variable)?);
    let verifier = if matches!(task, "9E" | "9M" | "9H") {
        Some(PathBuf::from(
            std::env::var_os("CHOREOFIX_PHILOSOPHERS_VERIFIER_BINARY")
                .ok_or("CHOREOFIX_PHILOSOPHERS_VERIFIER_BINARY is required")?,
        ))
    } else {
        None
    };
    let verifier_sha256 = verifier.as_deref().map(digest).transpose()?;
    let receipt_variable = match task {
        "1H" => Some("CHOREOFIX_CODEBASE_BUILD_RECEIPT"),
        "8E" => Some("CHOREOFIX_API_BUILD_RECEIPT"),
        "16E" => Some("CHOREOFIX_CICD_BUILD_RECEIPT"),
        "6E" => Some("CHOREOFIX_REVIEW_BUILD_RECEIPT"),
        "2E" | "2M" | "2H" => Some("CHOREOFIX_WRITING_BUILD_RECEIPT"),
        _ => None,
    };
    let build_receipt = if let Some(variable) = receipt_variable {
        let receipt = std::env::var_os(variable).ok_or(variable)?;
        let receipt: Value = serde_json::from_slice(&fs::read(receipt)?)?;
        if receipt["binary_sha256"] != digest(&runtime)? || receipt["source_unchanged"] != true {
            return Err("runtime binary does not match the source build receipt".into());
        }
        receipt
    } else {
        Value::Null
    };
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let config: Value = serde_json::from_slice(&fs::read("experiment.json")?)?;
    let seconds = config["run_timeout_seconds"]
        .as_u64()
        .ok_or("run timeout")?;
    let limit = config["participant_tool_call_limit"]
        .as_u64()
        .ok_or("tool limit")?;
    let trace = Arc::new(Trace::create(&directory.join("events.jsonl"))?);
    let (simulator, fixture) = oracle::Simulator::new(task, depth, 1.0, trace.clone(), &[])?;
    let audit_source = match task {
        "16E" => "tools/runner/src/cicd_audit.rs",
        "6E" => "tools/runner/src/review_audit.rs",
        "2E" | "2M" | "2H" => "tools/runner/src/writing_audit.rs",
        "8E" => "tools/runner/src/api_audit.rs",
        "1H" => "tools/runner/src/codebase_audit.rs",
        "9E" | "9M" | "9H" => "src/bin/philosophers-boundary/audit.rs",
        _ => return Err("missing protocol audit source".into()),
    };
    save(
        &directory.join("case.json"),
        &json!({"task":task,"depth":depth,"seed":0,"kind":"subscription_connection_actual_wasi_preflight","scored_benchmark":false,
        "authentication":"chatgpt","model":model::MODEL,"effort":model::EFFORT,"external_failure_specs":[],"test_barriers":"none","domain_delay_multiplier":1.0,
        "runtime_sha256":digest(&runtime)?,"wash_sha256":digest(module)?,"verifier_sha256":verifier_sha256,"runtime_build_receipt":build_receipt,"root_lock_sha256":digest(Path::new("Cargo.lock"))?,
        "model_source_sha256":digest(Path::new("tools/runner/src/model.rs"))?,"prompt_source_sha256":digest(Path::new("tools/runner/src/main.rs"))?,
        "audit_source_sha256":digest(Path::new(audit_source))?,
        "runner_sha256":digest(&std::env::current_exe()?)?,"runner_lock_sha256":digest(Path::new("tools/runner/Cargo.lock"))?,
        "interpreter":oracle::interpreter_identity()?}),
    )?;
    save(&directory.join("simulator-fixture.json"), &fixture)?;
    let (guest, runtime_loop) =
        process::guest(&runtime, module, task, &directory, trace, simulator)?;
    let started = Instant::now();
    let mut jobs = JoinSet::new();
    jobs.spawn(async move {
        runtime_loop
            .await
            .map(|result| json!({"kind":"runtime","result":result}))
    });
    let mut people = Vec::new();
    for actor in fixture["fixtures"]
        .as_array()
        .ok_or("original participants")?
    {
        let agent = actor["agent"].as_str().ok_or("agent")?.to_owned();
        let instructions = crate::prompt(Path::new("."), task, actor)?;
        fs::write(directory.join(format!("{agent}-prompt.txt")), &instructions)?;
        people.push(model::Participant {
            output: directory.join(&agent),
            agent,
            instructions,
        });
    }
    jobs.spawn(async move {
        model::run(
            people,
            model::Limits {
                timeout: Duration::from_secs(seconds),
                tool_calls: limit,
            },
            move |agent, command| {
                let guest = guest.clone();
                async move { guest.command(&agent, command).await }
            },
        )
        .await
        .map(|results| json!({"kind":"participants","results":results}))
    });
    let mut outcomes = Vec::new();
    let completed = tokio::time::timeout(Duration::from_secs(seconds + 20), async {
        while let Some(value) = jobs.join_next().await {
            match value {
                Ok(Ok(value)) if value["kind"] == "participants" => {
                    for participant in value["results"].as_array().ok_or("model results")? {
                        outcomes.push(json!({"kind":"participant","result":participant}));
                    }
                }
                Ok(Ok(value)) => outcomes.push(value),
                Ok(Err(error)) => {
                    outcomes.push(json!({"kind":"harness_error","error":error.to_string()}))
                }
                Err(error) => {
                    outcomes.push(json!({"kind":"harness_error","error":error.to_string()}))
                }
            }
            save(&directory.join("outcomes.json"), &json!(outcomes))?;
        }
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await;
    jobs.abort_all();
    while jobs.join_next().await.is_some() {}
    let result = json!({"kind":"subscription_connection_actual_wasi_preflight","task":task,"depth":depth,"scored_benchmark":false,
        "elapsed_seconds":started.elapsed().as_secs_f64(),"completed_within_timeout":completed.is_ok(),"outcomes":outcomes,"events_sha256":digest(&directory.join("events.jsonl"))?});
    save(&directory.join("result.json"), &result)?;
    println!("{}", directory.display());
    completed??;
    let participants: Vec<_> = outcomes
        .iter()
        .filter(|o| o["kind"] == "participant")
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
        .find(|o| o["kind"] == "runtime")
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
    let audit = if task == "16E" {
        crate::cicd_audit::audit(&directory.join("events.jsonl"))?
    } else if task == "6E" {
        crate::review_audit::audit(&directory.join("events.jsonl"))?
    } else if matches!(task, "2E" | "2M" | "2H") {
        crate::writing_audit::audit(&directory.join("events.jsonl"))?
    } else if task == "8E" {
        crate::api_audit::audit(&directory.join("events.jsonl"))?
    } else if task == "1H" {
        crate::codebase_audit::audit(&directory.join("events.jsonl"))?
    } else {
        let verifier = verifier.ok_or("missing dining verifier")?;
        let audit = Command::new(verifier)
            .arg("--audit")
            .arg(directory.join("events.jsonl"))
            .stdout(Stdio::piped())
            .stderr(fs::File::create_new(directory.join("audit.stderr"))?)
            .output()
            .await?;
        fs::write(directory.join("audit.json"), &audit.stdout)?;
        if !audit.status.success() {
            return Err("model trace failed delegation audit".into());
        }
        serde_json::from_slice(&audit.stdout)?
    };
    save(&directory.join("audit.json"), &audit)?;
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
