//! Actual wash/WASI requests against the unchanged codebase simulator.
//! Scripted stimuli test the boundary and do not count as model experiments.
use crate::{
    evidence::{Result, Trace, archive_build_receipt, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::task::JoinSet;

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("terminal stdout")?;
    let start = text.find('{').ok_or("service result missing")?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service result")?
        .map_err(Into::into)
}

fn count(events: &[Value], agent: &str, tool: &str) -> usize {
    events
        .iter()
        .filter(|event| event["agent"] == agent && event["tool"] == tool)
        .count()
}

async fn developer(
    guest: process::Guest,
    simulator: oracle::Simulator,
    actor: Value,
    task: String,
) -> Result<Vec<Value>> {
    let agent = actor["agent"].as_str().ok_or("original participant")?;
    let calls = actor["calls"].as_array().ok_or("original operations")?;
    assert_eq!(calls.len(), 4);

    // Ordinary private work and failures must not consume a shared operation.
    let private = guest.command(agent,
        "total=0; for x in 1 2 3; do expr $total + $x | read total; done; printf '%s' \"$total\" > /work/count; cat /missing || cat /work/count".into()).await?;
    assert!(
        private["stdout"]
            .as_str()
            .ok_or("private stdout")?
            .contains('6')
    );
    assert_eq!(
        count(&simulator.observed_events().await?, agent, "design_feature"),
        0
    );
    let early = guest
        .command(
            agent,
            format!("echo '{}' > /service/commit_changes", calls[2]["arguments"]),
        )
        .await?;
    assert!(
        early["stdout"]
            .as_str()
            .ok_or("early stdout")?
            .contains("wash:")
    );
    assert_eq!(
        count(&simulator.observed_events().await?, agent, "commit_changes"),
        0
    );

    let mut feedback = Vec::new();
    for call in calls {
        let tool = call["tool"].as_str().ok_or("original tool")?;
        for arguments in [
            "not-json".to_string(),
            "{}".to_string(),
            if tool == "commit_changes" {
                "{\"modules\":\"AUTH_MODULE,AUTH_MODULE\"}".into()
            } else {
                "{\"feature_name\":\"another_participants_feature\"}".into()
            },
        ] {
            let before = count(&simulator.observed_events().await?, agent, tool);
            let invalid = guest
                .command(
                    agent,
                    format!("echo '{arguments}' > /service/{tool}; cat /service/result"),
                )
                .await?;
            assert_eq!(response(&invalid)?["status"], "invalid_arguments");
            assert_eq!(
                count(&simulator.observed_events().await?, agent, tool),
                before
            );
        }
        // This exceeds one guest write chunk and exercises ABI normalization.
        let padding = " ".repeat(512);
        let valid = guest
            .command(
                agent,
                format!(
                    "printf '%s' '{padding}{}' > /service/{tool}; cat /service/result",
                    call["arguments"]
                ),
            )
            .await?;
        let result = response(&valid)?;
        if tool != "run_local_tests" {
            assert_eq!(result["success"], true, "{agent}/{tool}: {result}");
        }
        feedback.push(result);
        let before = count(&simulator.observed_events().await?, agent, tool);
        assert_eq!(before, 1, "{agent}/{tool} effect count");
        let duplicate = guest
            .command(
                agent,
                format!("echo '{}' > /service/{tool}", call["arguments"]),
            )
            .await?;
        assert!(
            duplicate["stdout"]
                .as_str()
                .ok_or("duplicate stdout")?
                .contains("wash:")
        );
        assert_eq!(
            count(&simulator.observed_events().await?, agent, tool),
            before
        );
    }
    if task == "1H" {
        while feedback.last().ok_or("test result")?["success"] == false {
            let private=guest.command(agent,
                "note=reconsider; printf '%s' \"$note\" > /work/revision; cat /work/revision; echo '{}' > /service/implement_code; cat /service/result".into()).await?;
            assert_eq!(response(&private)?["status"], "invalid_arguments");
            for call in &calls[1..] {
                let tool = call["tool"].as_str().ok_or("revision operation")?;
                let before = count(&simulator.observed_events().await?, agent, tool);
                let reply = guest
                    .command(
                        agent,
                        format!(
                            "echo '{}' > /service/{tool}; cat /service/result",
                            call["arguments"]
                        ),
                    )
                    .await?;
                let result = response(&reply)?;
                if tool != "run_local_tests" {
                    assert_eq!(result["success"], true);
                }
                assert_eq!(
                    count(&simulator.observed_events().await?, agent, tool),
                    before + 1
                );
                feedback.push(result);
            }
        }
        assert_eq!(feedback.last().ok_or("test result")?["success"], true);
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(feedback)
}

// Fixture setup and evidence checks contain no application Endpoint operations.
fn output(label: &str) -> Result<std::path::PathBuf> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let root = std::env::var_os("CHOREOFIX_EVIDENCE_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new("output").to_owned());
    fs::create_dir_all(&root)?;
    let directory = root.join(format!(
        "{label}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    Ok(directory)
}

async fn case(
    directory: &Path,
    task: &str,
    depth: u8,
    delay: f64,
    failures: &[(&str, u32)],
) -> Result<Value> {
    let runtime = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_CODEBASE_WASI_BINARY").ok_or("codebase binary missing")?,
    );
    let receipt_path = std::path::PathBuf::from(
        std::env::var_os("CHOREOFIX_CODEBASE_BUILD_RECEIPT")
            .ok_or("codebase build receipt missing")?,
    );
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let case = directory.join(format!("{task}-depth{depth}"));
    fs::create_dir(&case)?;
    let receipt = archive_build_receipt(&runtime, &receipt_path, &case)?;
    save(
        &case.join("case.json"),
        &json!({"task":task,"depth":depth,
        "kind":"deterministic_rust_wasi_boundary","model_calls":0,"scored_benchmark":false,"build":receipt,
        "test_barriers":"none", "domain_delay_multiplier":delay,"external_failure_specs":failures,
        "runtime_binary_sha256":digest(&runtime)?,"guest_wasm_sha256":digest(module)?,
        "harness_binary_sha256":digest(&std::env::current_exe()?)?,
        "harness_source_sha256":digest(Path::new("tools/runner/src/codebase_boundary.rs"))?,
        "oracle_source_sha256":digest(Path::new("tools/runner/src/oracle.rs"))?,
        "audit_source_sha256":digest(Path::new("tools/runner/src/codebase_audit.rs"))?,
        "root_lock_sha256":digest(Path::new("Cargo.lock"))?,"runner_lock_sha256":digest(Path::new("tools/runner/Cargo.lock"))?,
        "simulator_source_sha256":digest(&Path::new("vendor/tracefix/benchmark/environments").join(task).join("sim.py"))?}),
    )?;
    let trace = Trace::create(&case.join("events.jsonl"))?;
    let (simulator, fixture) = oracle::Simulator::new(task, depth, delay, trace.clone(), failures)?;
    save(&case.join("simulator-fixture.json"), &fixture)?;
    let actors = fixture["fixtures"].as_array().ok_or("original actors")?;
    let (guest, runtime_loop) =
        process::guest(&runtime, module, task, &case, trace, simulator.clone())?;
    let mut participants = JoinSet::new();
    for actor in actors {
        participants.spawn(developer(
            guest.clone(),
            simulator.clone(),
            actor.clone(),
            task.to_owned(),
        ));
    }
    let (participants, result) = tokio::time::timeout(Duration::from_secs(90), async {
        tokio::join!(
            async {
                let mut feedback = Vec::new();
                while let Some(result) = participants.join_next().await {
                    feedback.push(result??);
                }
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(feedback)
            },
            runtime_loop
        )
    })
    .await?;
    let result = result?;
    save(&case.join("result.json"), &result)?;
    let feedback = participants?;
    let audit = crate::codebase_audit::audit(&case.join("events.jsonl"))?;
    save(&case.join("audit.json"), &audit)?;
    crate::codebase_audit::check_mutants(&case.join("events.jsonl"), &case.join("mutants"))?;
    assert_eq!(
        result["protocol"],
        json!({"type":"run_result","status":"complete"})
    );
    assert_eq!(result["runtime_exit_code"], 0);
    assert_eq!(result["backend_errors"], json!([]));
    assert_eq!(
        result["score"]["complete"],
        task == "1H" || failures.is_empty()
    );
    assert_eq!(result["score"]["violations"], json!([]));
    assert_eq!(
        result["score"]["events"]
            .as_array()
            .ok_or("original events")?
            .len(),
        audit["domain_calls"]
            .as_u64()
            .ok_or("audited effect count")? as usize
    );
    if !failures.is_empty() && task != "1H" {
        for participant in &feedback {
            assert_eq!(participant.last().ok_or("test verdict")?["success"], false);
        }
    }
    Ok(
        json!({"task":task,"depth":depth,"result":result,"feedback":feedback,"audit":audit,
        "kind":"deterministic_rust_wasi_boundary","model_calls":0,"scored_benchmark":false}),
    )
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires seven actual WASI guests and the unchanged original simulator"]
async fn seven_guests_reconsider_failures_through_delegated_module_endpoints() -> Result<()> {
    let directory = output("codebase-rust-reconsider")?;
    let mut outcomes = Vec::new();
    for depth in [0, 1, 2] {
        outcomes.push(case(&directory, "1H", depth, 0.01, &[]).await?);
        save(&directory.join("summary.json"), &json!(outcomes))?;
    }
    let repeated = directory.join("two-failed-tests");
    fs::create_dir(&repeated)?;
    let outcome = case(
        &repeated,
        "1H",
        0,
        0.01,
        &[("run_local_tests", 0), ("run_local_tests", 1)],
    )
    .await?;
    assert!(
        outcome["audit"]["attempts"]
            .as_object()
            .ok_or("attempts")?
            .values()
            .all(|count| count.as_u64() == Some(3))
    );
    outcomes.push(outcome);
    let normal = directory.join("normal-delays");
    fs::create_dir(&normal)?;
    let outcome = case(&normal, "1H", 0, 1.0, &[]).await?;
    assert_eq!(outcome["audit"]["max_disjoint_commits"], 3);
    outcomes.push(outcome);
    save(&directory.join("summary.json"), &json!(outcomes))?;
    println!("{}", directory.display());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the built codebase WASI runtime, wash and original simulator environment"]
async fn typed_requests_gate_effects_and_keep_shell_errors_recoverable() -> Result<()> {
    let directory = output("codebase-rust-boundary")?;
    let mut outcomes = Vec::new();
    for task in ["1E", "1M"] {
        for depth in [0, 1, 2] {
            outcomes.push(case(&directory, task, depth, 0.01, &[]).await?);
            save(&directory.join("summary.json"), &json!(outcomes))?;
        }
    }
    println!("{}", directory.display());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires real WASI and uses only the upstream FailureSpec environment API"]
async fn terminal_test_errors_close_without_retries() -> Result<()> {
    let directory = output("codebase-rust-terminal-tests")?;
    let mut outcomes = Vec::new();
    for task in ["1E", "1M"] {
        outcomes.push(case(&directory, task, 0, 0.01, &[("run_local_tests", 0)]).await?);
        save(&directory.join("summary.json"), &json!(outcomes))?;
    }
    println!("{}", directory.display());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires real WASI and original normal domain delays, without barriers"]
async fn normal_delays_allow_two_disjoint_actual_wasi_commits() -> Result<()> {
    let directory = output("codebase-rust-normal-delays")?;
    let outcome = case(&directory, "1M", 0, 1.0, &[]).await?;
    save(&directory.join("summary.json"), &outcome)?;
    assert_eq!(outcome["audit"]["max_disjoint_commits"], 2);
    println!("{}", directory.display());
    Ok(())
}
