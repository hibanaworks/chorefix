use super::*;
use crate::evidence::save;

#[tokio::test(flavor = "current_thread")]
#[ignore = "checks original simulator resource sensitivity with real concurrent calls"]
async fn codebase_simulator_allows_independent_work_and_reports_shared_module_overlap() -> Result<()>
{
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
        "codebase-simulator-sensitivity-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let mut outcomes = Vec::new();
    for (name, calls, conflict) in [
        ("disjoint_commits", vec![(0, 2), (2, 2), (4, 2)], false),
        ("shared_database", vec![(0, 2), (1, 2)], true),
        ("test_and_next_commit", vec![(0, 3), (1, 2)], false),
    ] {
        let case = directory.join(name);
        fs::create_dir(&case)?;
        let trace = Arc::new(Trace::create(&case.join("events.jsonl"))?);
        let (simulator, fixture) = Simulator::new("1H", 0, 1.0, trace, &[])?;
        let mut jobs = tokio::task::JoinSet::new();
        for (id, (agent, tool)) in calls.into_iter().enumerate() {
            let actor = &fixture["fixtures"][agent];
            let call = &actor["calls"][tool];
            let message = json!({"type":"execute","id":id,"agent":actor["agent"],
                "tool":call["tool"],"arguments":call["arguments"]});
            let simulator = simulator.clone();
            jobs.spawn(async move { simulator.call(message).await });
        }
        let mut replies = Vec::new();
        while let Some(reply) = jobs.join_next().await {
            replies.push(reply??);
        }
        let latest_start = replies
            .iter()
            .map(|r| r["started_ns"].as_u64().unwrap())
            .max()
            .unwrap();
        let earliest_end = replies
            .iter()
            .map(|r| r["finished_ns"].as_u64().unwrap())
            .min()
            .unwrap();
        assert!(latest_start < earliest_end, "{name} calls did not overlap");
        let score = simulator.score()?;
        let violations = score["violations"]
            .as_array()
            .ok_or("original violations")?;
        if conflict {
            assert!(
                violations
                    .iter()
                    .any(|v| v["violation_type"] == "concurrent_resource_use")
            );
        } else {
            assert!(violations.is_empty(), "independent work rejected: {score}");
        }
        let outcome = json!({"case":name,"replies":replies,"score":score,
            "overlap":true,"expected_resource_conflict":conflict,
            "model_calls":0,"scored_benchmark":false,
            "scope":"original simulator sensitivity; no Hibana protocol is under test"});
        save(&case.join("result.json"), &outcome)?;
        outcomes.push(outcome);
    }
    save(&directory.join("summary.json"), &json!({"cases":outcomes}))?;
    println!("{}", directory.display());
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "calls the original registry error path through the Rust callback"]
async fn original_tool_exception_reaches_rust_without_effects_or_a_hung_reply() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let directory = Path::new("output").join(format!(
        "upstream-rust-error-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let trace = Arc::new(Trace::create(&directory.join("events.jsonl"))?);
    let (simulator, _) = Simulator::new("6E", 0, 0.0, trace, &[])?;
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), simulator.call(json!({
        "type":"execute","id":1,"agent":"DEVELOPER_A","tool":"not_an_original_tool","arguments":{}
    }))).await?;
    assert!(result.is_err());
    assert!(simulator.observed_events()?.is_empty());
    save(
        &directory.join("result.json"),
        &json!({"kind":"upstream_api_error_test","model_calls":0,"scored_benchmark":false,
        "error":result.unwrap_err().to_string(),"events":simulator.observed_events()?}),
    )?;
    println!("{}", directory.display());
    Ok(())
}

#[test]
#[ignore = "initializes the unchanged upstream environments at all 144 task-depth combinations"]
fn all_original_environment_classes_load_from_verified_sources() -> Result<()> {
    std::env::set_current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
    )?;
    let manifest: Value = serde_json::from_slice(&fs::read("benchmark/upstream-manifest.json")?)?;
    let directory = Path::new("output").join(format!(
        "upstream-rust-api-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let trace = Arc::new(Trace::create(&directory.join("events.jsonl"))?);
    let mut results = Vec::new();
    for task in manifest["tasks"].as_array().ok_or("48 task records")? {
        let id = task["id"].as_str().ok_or("task id")?;
        for depth in 0..=2 {
            let (simulator, fixture) = Simulator::new(id, depth, 0.0, trace.clone(), &[])?;
            let original: Value = serde_json::from_slice(&fs::read(
                Path::new("vendor/tracefix/benchmark/descriptions")
                    .join(id)
                    .join("metadata.json"),
            )?)?;
            assert_eq!(
                fixture["fixtures"]
                    .as_array()
                    .ok_or("fixtures")?
                    .iter()
                    .map(|v| v["agent"].clone())
                    .collect::<Vec<_>>(),
                original["agents"]
                    .as_array()
                    .ok_or("original agents")?
                    .clone()
            );
            let score = simulator.score()?;
            assert_eq!(score["events"], json!([]));
            assert_eq!(score["violations"], json!([]));
            results.push(json!({"task":id,"depth":depth,"fixture":fixture,"initial_score":score}));
        }
    }
    assert_eq!(results.len(), 144);
    save(
        &directory.join("initializations.json"),
        &json!({"cases":results,"model_calls":0,"scored_benchmark":false,
        "scope":"upstream import, construction and initial scorer only; no task port acceptance"}),
    )?;
    println!("{}", directory.display());
    Ok(())
}
