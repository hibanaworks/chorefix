//! Deterministic shell stimuli for the actual 6E WASI runtime.
//! These tests neither authorize an operation nor replace a localside.
use crate::{
    evidence::{Result, Trace, digest, save},
    oracle, process,
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug)]
enum ReviewSchedule {
    FinishFirst,
    Alternate,
}

fn response(reply: &Value) -> Result<Value> {
    let text = reply["stdout"].as_str().ok_or("terminal stdout")?;
    let start = text
        .find('{')
        .ok_or_else(|| format!("service reply missing: {reply}"))?;
    serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<Value>()
        .next()
        .ok_or("empty service reply")?
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
    simulator: Arc<oracle::Simulator>,
    agent: &str,
    feature: &str,
    branch: &str,
) -> Result<Vec<Value>> {
    let early = guest
        .command(
            agent,
            format!("echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code"),
        )
        .await?;
    assert!(
        early["stdout"]
            .as_str()
            .ok_or("early stdout")?
            .contains("wash:")
    );
    assert_eq!(count(&simulator.observed_events()?, agent, "merge_code"), 0);
    guest
        .command(agent, "echo '{}' > /service/implement_feature".into())
        .await?;
    let invalid = guest.command(agent, "cat /service/result".into()).await?;
    assert_eq!(response(&invalid)?["status"], "invalid_arguments");
    let (other_feature, other_branch) = if agent == "DEVELOPER_A" {
        ("notification_feature", "dev_b_feature")
    } else {
        ("user_authentication", "dev_a_feature")
    };
    let wrong = guest.command(agent, format!("echo '{{\"feature_name\":\"{other_feature}\"}}' > /service/implement_feature; cat /service/result")).await?;
    assert_eq!(response(&wrong)?["status"], "invalid_arguments");
    assert_eq!(
        count(&simulator.observed_events()?, agent, "implement_feature"),
        0
    );
    let mut feedback = Vec::new();
    loop {
        let padding = " ".repeat(512);
        let reply = guest.command(agent, format!("printf '%s' '{padding}{{\"feature_name\":\"{feature}\"}}' > /service/implement_feature; cat /service/result")).await?;
        let result = response(&reply)?;
        assert_eq!(result["success"], true);
        let review = result["coordination"]["code_review"].clone();
        let accepted = review["success"].as_bool().ok_or("actual review result")?;
        feedback.push(review);
        if accepted {
            break;
        }
        let denied = guest
            .command(
                agent,
                format!("echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code"),
            )
            .await?;
        assert!(
            denied["stdout"]
                .as_str()
                .ok_or("rejected merge stdout")?
                .contains("wash:")
        );
        assert_eq!(count(&simulator.observed_events()?, agent, "merge_code"), 0);
    }
    let wrong = guest.command(agent, format!("echo '{{\"branch\":\"{other_branch}\"}}' > /service/merge_code; cat /service/result")).await?;
    assert_eq!(response(&wrong)?["status"], "invalid_arguments");
    assert_eq!(count(&simulator.observed_events()?, agent, "merge_code"), 0);
    let reply = guest
        .command(
            agent,
            format!(
                "echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code; cat /service/result"
            ),
        )
        .await?;
    assert_eq!(response(&reply)?["success"], true);
    let before = simulator.observed_events()?;
    for (tool, arguments) in [
        ("implement_feature", json!({"feature_name":feature})),
        ("merge_code", json!({"branch":branch})),
    ] {
        let reply = guest
            .command(agent, format!("echo '{arguments}' > /service/{tool}"))
            .await?;
        assert!(
            reply["stdout"]
                .as_str()
                .ok_or("duplicate stdout")?
                .contains("os error 8")
        );
        assert_eq!(
            count(&simulator.observed_events()?, agent, tool),
            count(&before, agent, tool)
        );
    }
    assert_eq!(
        guest.command(agent, "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(feedback)
}

async fn reviewer(
    guest: process::Guest,
    simulator: Arc<oracle::Simulator>,
    first: &str,
    schedule: ReviewSchedule,
) -> Result<()> {
    let other = if first == "user_authentication" {
        "notification_feature"
    } else {
        "user_authentication"
    };
    let mut requests = VecDeque::from([first, other]);
    while let Some(feature) = requests.pop_front() {
        let other_feature = if feature == "user_authentication" {
            "notification_feature"
        } else {
            "user_authentication"
        };
        let before = simulator.observed_events()?;
        let wrong = guest.command("REVIEWER", format!("echo '{{\"pr_id\":\"{other_feature}\"}}' > /service/code_review/{feature}; cat /service/result")).await?;
        assert_eq!(response(&wrong)?["status"], "invalid_arguments");
        let after = simulator.observed_events()?;
        assert!(!after[before.len()..].iter().any(
            |event| event["tool"] == "code_review" && event["args"]["target"] == other_feature
        ));
        let reply = guest.command("REVIEWER", format!("echo '{{\"pr_id\":\"{feature}\"}}' > /service/code_review/{feature}; cat /service/result")).await?;
        if !response(&reply)?["success"]
            .as_bool()
            .ok_or("review outcome")?
        {
            match schedule {
                ReviewSchedule::FinishFirst => requests.push_front(feature),
                ReviewSchedule::Alternate => requests.push_back(feature),
            }
            continue;
        }
        let before = simulator.observed_events()?;
        let duplicate = guest
            .command(
                "REVIEWER",
                format!("echo '{{\"pr_id\":\"{feature}\"}}' > /service/code_review/{feature}"),
            )
            .await?;
        assert!(
            duplicate["stdout"]
                .as_str()
                .ok_or("duplicate review stdout")?
                .contains("os error 8")
        );
        assert_eq!(
            count(&simulator.observed_events()?, "REVIEWER", "code_review"),
            count(&before, "REVIEWER", "code_review")
        );
    }
    assert_eq!(
        guest.command("REVIEWER", "exit 0".into()).await?["exit_status"],
        0
    );
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "uses the built WASI runtime, real wash and unchanged upstream simulator"]
async fn real_wasi_reviews_reconsider_and_interleave_with_unchanged_upstream() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = Path::new("output").join(format!(
        "review-rust-boundary-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&directory)?;
    let runtime = std::env::var_os("CHOREOFIX_REVIEW_WASI_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "target/debug/review-wasi".into());
    let module = Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm");
    let mut outcomes = Vec::new();
    for depth in [0, 1, 2] {
        for review_first in ["user_authentication", "notification_feature"] {
            for schedule in [ReviewSchedule::FinishFirst, ReviewSchedule::Alternate] {
                let case = directory.join(format!("depth{depth}-{review_first}-{schedule:?}"));
                fs::create_dir(&case)?;
                save(
                    &case.join("case.json"),
                    &json!({"task":"6E","depth":depth,"review_first":review_first,"schedule":format!("{schedule:?}"),
                "kind":"deterministic_rust_wasi_regression","model_calls":0,"scored_benchmark":false,
                "test_barriers":"none",
                "source_qualification":"This fixture does not qualify the recorded root Git dependencies",
                "runtime_binary_sha256":digest(&runtime)?,"guest_wasm_sha256":digest(module)?,
                "root_lock_sha256":digest(Path::new("Cargo.lock"))?,"runner_lock_sha256":digest(Path::new("tools/runner/Cargo.lock"))?,
                "simulator_source_sha256":digest(Path::new("vendor/tracefix/benchmark/environments/6E/sim.py"))?}),
                )?;
                let trace = Arc::new(Trace::create(&case.join("events.jsonl"))?);
                let (simulator, _) = oracle::Simulator::new("6E", depth, 0.01, trace.clone(), &[])?;
                let (guest, runtime_loop) =
                    process::guest(&runtime, module, "6E", &case, trace, simulator.clone())?;
                let (a, b, review, result) = tokio::time::timeout(Duration::from_secs(60), async {
                    tokio::join!(
                        developer(
                            guest.clone(),
                            simulator.clone(),
                            "DEVELOPER_A",
                            "user_authentication",
                            "dev_a_feature"
                        ),
                        developer(
                            guest.clone(),
                            simulator.clone(),
                            "DEVELOPER_B",
                            "notification_feature",
                            "dev_b_feature"
                        ),
                        reviewer(guest.clone(), simulator.clone(), review_first, schedule),
                        runtime_loop,
                    )
                })
                .await?;
                let a = a?;
                let b = b?;
                review?;
                let result = result?;
                assert_eq!(
                    result["protocol"],
                    json!({"type":"run_result","status":"complete"})
                );
                assert_eq!(result["runtime_exit_code"], 0);
                assert_eq!(result["backend_errors"], json!([]));
                assert_eq!(result["score"]["complete"], true);
                assert_eq!(result["score"]["violations"], json!([]));
                assert_eq!(
                    result["score"]["events"]
                        .as_array()
                        .ok_or("original events")?
                        .len(),
                    6 + 2 * usize::from(depth)
                );
                assert_eq!(
                    a.iter()
                        .chain(&b)
                        .filter(|reply| reply["success"] == false)
                        .count(),
                    usize::from(depth)
                );
                for feedback in [&a, &b] {
                    assert_eq!(
                        feedback.last().ok_or("missing review feedback")?["success"],
                        true
                    );
                }
                if matches!(schedule, ReviewSchedule::FinishFirst) {
                    for (feedback, expected) in [
                        (
                            &a,
                            if review_first == "user_authentication" {
                                depth
                            } else {
                                0
                            },
                        ),
                        (
                            &b,
                            if review_first == "notification_feature" {
                                depth
                            } else {
                                0
                            },
                        ),
                    ] {
                        let expected: Vec<_> =
                            std::iter::repeat_n(json!(false), usize::from(expected))
                                .chain([json!(true)])
                                .collect();
                        assert_eq!(
                            feedback
                                .iter()
                                .map(|r| r["success"].clone())
                                .collect::<Vec<_>>(),
                            expected
                        );
                    }
                }
                let result = json!({"depth":depth,"review_first":review_first,"schedule":format!("{schedule:?}"),"result":result,"a_feedback":a,"b_feedback":b,
                "kind":"deterministic_rust_wasi_regression","model_calls":0,"scored_benchmark":false});
                save(&case.join("result.json"), &result)?;
                outcomes.push(result);
                save(&directory.join("summary.json"), &json!(outcomes))?;
            }
        }
    }
    println!("{}", directory.display());
    Ok(())
}
