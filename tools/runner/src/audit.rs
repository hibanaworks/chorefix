//! Independent saved-evidence checks. No experiment execution or task authority.
use crate::{
    evidence::{Result, digest},
    model, snapshot,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader},
    path::{Component, Path},
    process::Command,
};

fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn records(path: &Path) -> Result<Vec<Value>> {
    BufReader::new(fs::File::open(path)?)
        .lines()
        .map(|v| Ok(serde_json::from_str(&v?)?))
        .collect()
}
fn unique<'a>(rows: &'a [Value], direction: &str, method: &str) -> Result<&'a Value> {
    let values: Vec<_> = rows
        .iter()
        .filter(|v| v["direction"] == direction && v["message"]["method"] == method)
        .collect();
    if values.len() != 1 {
        return Err(format!("expected one {direction} {method}").into());
    }
    Ok(values[0])
}
fn tokens(value: &Value) -> Result<u64> {
    let input = value["inputTokens"].as_u64().ok_or("input tokens")?;
    let output = value["outputTokens"].as_u64().ok_or("output tokens")?;
    let total = value["totalTokens"].as_u64().ok_or("inclusive tokens")?;
    if input == 0 || output == 0 || total != input.checked_add(output).ok_or("token overflow")? {
        return Err("invalid inclusive token accounting".into());
    }
    Ok(total)
}

fn participant(root: &Path, agent: &str, runtime: &[Value]) -> Result<Value> {
    let directory = root.join(agent);
    let report = load(&directory.join("participant.json"))?;
    if report["status"] != "completed"
        || !report["error"].is_null()
        || report["requested_model"] != model::MODEL
        || report["returned_model"] != model::MODEL
        || report["requested_effort"] != model::EFFORT
        || report["returned_effort"] != model::EFFORT
        || report["raw_events_sha256"] != digest(&directory.join("events.jsonl"))?
    {
        return Err("failed or altered participant report".into());
    }
    let rows = records(&directory.join("events.jsonl"))?;
    let thread = &unique(&rows, "send", "thread/start")?["message"]["params"];
    let instructions = fs::read_to_string(root.join(format!("{agent}-prompt.txt")))?;
    if thread["model"] != model::MODEL
        || thread["allowProviderModelFallback"] != false
        || thread["config"]["model_reasoning_effort"] != model::EFFORT
        || thread["config"]["features.shell_tool"] != false
        || thread["config"]["features.multi_agent"] != false
        || thread["baseInstructions"]
            != format!(
                "{instructions}\nUse only the supplied experiment tools. Do not use native tools or start other agents. Finish with a brief final response."
            )
        || thread["dynamicTools"]
            .as_array()
            .is_none_or(|a| a.len() != 1 || a[0]["name"] != "shell")
    {
        return Err("model request settings or prompt differ".into());
    }
    let metadata: Vec<_> = rows
        .iter()
        .filter(|v| {
            v["direction"] == "receive"
                && v["message"]["id"] == 2
                && v["message"].get("method").is_none()
        })
        .collect();
    if metadata.len() != 1
        || metadata[0]["message"]["result"]["model"] != model::MODEL
        || metadata[0]["message"]["result"]["reasoningEffort"] != model::EFFORT
    {
        return Err("actual model/effort not proven".into());
    }
    let turn = unique(&rows, "send", "turn/start")?;
    if turn["message"]["params"]["model"] != model::MODEL
        || turn["message"]["params"]["effort"] != model::EFFORT
    {
        return Err("turn settings differ".into());
    }
    let completed = unique(&rows, "receive", "turn/completed")?;
    if completed["message"]["params"]["turn"]["status"] != "completed"
        || !completed["message"]["params"]["turn"]["error"].is_null()
    {
        return Err("turn did not complete".into());
    }
    let usage = rows
        .iter()
        .rev()
        .find(|v| {
            v["direction"] == "receive" && v["message"]["method"] == "thread/tokenUsage/updated"
        })
        .ok_or("missing raw usage")?;
    let usage = &usage["message"]["params"]["tokenUsage"]["total"];
    if *usage != report["usage"] {
        return Err("usage differs from raw event".into());
    }
    let total = tokens(usage)?;
    let mut pending = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut exchanges = Vec::new();
    for row in &rows {
        let message = &row["message"];
        if row["direction"] == "receive" {
            if let Some(kind) = message["params"]["item"]["type"].as_str()
                && ![
                    "dynamicToolCall",
                    "reasoning",
                    "userMessage",
                    "agentMessage",
                ]
                .contains(&kind)
            {
                return Err("native model tool item present".into());
            }
            if message["method"] == "item/tool/call" {
                let id = message["id"].to_string();
                if !ids.insert(id.clone()) || message["params"]["tool"] != "shell" {
                    return Err("duplicate or unexpected model tool".into());
                }
                let arguments = &message["params"]["arguments"];
                if arguments.as_object().is_none_or(|v| v.len() != 1) {
                    return Err("unexpected shell arguments".into());
                }
                pending.insert(
                    id,
                    arguments["command"]
                        .as_str()
                        .ok_or("shell command")?
                        .to_owned(),
                );
            }
        } else if row["direction"] == "send"
            && let Some(command) = pending.remove(&message["id"].to_string())
        {
            if message["result"]["success"] != true {
                return Err("failed shell dispatch".into());
            }
            let items = message["result"]["contentItems"]
                .as_array()
                .ok_or("tool reply")?;
            if items.len() != 1 || items[0]["type"] != "inputText" {
                return Err("tool reply shape".into());
            }
            let response: Value =
                serde_json::from_str(items[0]["text"].as_str().ok_or("tool text")?)?;
            if command.len() > 4096 {
                if response != json!({"status":"script_too_long","limit_utf8_bytes":4096}) {
                    return Err("script limit response".into());
                }
            } else {
                exchanges.push((command, response));
            }
        }
    }
    if !pending.is_empty() || report["tool_calls"] != ids.len() || ids.is_empty() {
        return Err("incomplete tool exchanges".into());
    }
    let mut commands = BTreeMap::new();
    let mut delivered = Vec::new();
    let mut exited = false;
    for row in runtime {
        let message = &row["message"];
        if message["agent"] != agent {
            continue;
        }
        if row["direction"] == "to_runtime" && message["type"] == "command" {
            if exited
                || commands
                    .insert(message["id"].to_string(), message["command"].clone())
                    .is_some()
            {
                return Err("duplicate or after-exit command".into());
            }
        } else if row["direction"] == "runtime"
            && ["terminal", "exit", "terminal_closed"]
                .contains(&message["type"].as_str().unwrap_or(""))
            && !message["id"].is_null()
        {
            let command = commands
                .remove(&message["id"].to_string())
                .ok_or("terminal reply without command")?;
            let reply = match message["type"].as_str() {
                Some("terminal") => json!({"stdout":message["stdout"]}),
                Some("exit") => {
                    exited = true;
                    json!({"stdout":message["stdout"],"exit_status":message["status"]})
                }
                _ => json!({"status":"guest_closed"}),
            };
            delivered.push((command.as_str().ok_or("runtime command")?.to_owned(), reply));
        }
    }
    if !commands.is_empty() || !exited || delivered != exchanges {
        return Err("model commands/replies differ from actual guest transport".into());
    }
    Ok(
        json!({"participant":agent,"inclusive_tokens":total,"usage":usage,"tool_calls":ids.len(),
        "turn_start_unix_ns":turn["unix_ns"],"turn_end_unix_ns":completed["unix_ns"]}),
    )
}

pub fn run(root: &Path, verifier: &Path) -> Result<Value> {
    let result = load(&root.join("result.json"))?;
    let versions = load(&root.join("versions.json"))?;
    if result["versions"] != versions
        || !result["error"].is_null()
        || result["scored_benchmark"] != false
        || result["events_sha256"] != digest(&root.join("events.jsonl"))?
    {
        return Err("failed or altered case result".into());
    }
    for (name, hash) in versions["binaries"].as_object().ok_or("binary hashes")? {
        if name.contains('/') || json!(digest(&root.join(name))?) != *hash {
            return Err("binary differs".into());
        }
    }
    if versions["binaries"]["runtime"]
        != "bbeed751bbf811e6aadf97f5cc787fe8a00b9208889a02af5dca75ca210b35d5"
        || versions["binaries"]["wash.wasm"]
            != "407e1fe263231022eb542a7d2a35e4d4705a58590f92b77bf21fe5f7c77258bc"
    {
        return Err("unqualified runtime or guest".into());
    }
    let manifest = load(&root.join("source-manifest.json"))?;
    if versions["source_manifest_sha256"] != digest(&root.join("source-manifest.json"))? {
        return Err("source manifest hash differs".into());
    }
    for (name, hash) in manifest.as_object().ok_or("source manifest")? {
        if Path::new(name)
            .components()
            .any(|v| !matches!(v, Component::Normal(_)))
            || json!(digest(&root.join("source").join(name))?) != *hash
        {
            return Err(format!("source differs: {name}").into());
        }
    }
    snapshot::verify_upstream(&root.join("source"))?;
    let rows = records(&root.join("events.jsonl"))?;
    let fixture = &rows.first().ok_or("no runtime evidence")?["message"];
    if fixture["model_run"] != true
        || fixture["test_delays"] != json!({})
        || fixture["delay_multiplier"] != 1.0
    {
        return Err("model fixture settings differ".into());
    }
    let mut participants = Vec::new();
    for actor in fixture["fixtures"].as_array().ok_or("fixture actors")? {
        let expected = crate::prompt(
            &root.join("source"),
            fixture["task"].as_str().ok_or("task")?,
            actor,
        )?;
        if fs::read_to_string(root.join(format!(
            "{}-prompt.txt",
            actor["agent"].as_str().ok_or("actor")?
        )))? != expected
        {
            return Err("saved prompt differs from frozen original inputs and interface".into());
        }
        participants.push(participant(
            root,
            actor["agent"].as_str().ok_or("actor")?,
            &rows,
        )?);
    }
    let mut times = Vec::new();
    for p in &participants {
        let start: u128 = p["turn_start_unix_ns"]
            .as_str()
            .ok_or("turn start time")?
            .parse()?;
        let end: u128 = p["turn_end_unix_ns"]
            .as_str()
            .ok_or("turn end time")?
            .parse()?;
        if start >= end {
            return Err("invalid model interval".into());
        }
        times.push((start, 1i32));
        times.push((end, -1i32));
    }
    times.sort();
    let (mut active, mut maximum) = (0, 0);
    for (_, change) in times {
        active += change;
        maximum = maximum.max(active);
    }
    if maximum != participants.len() as i32 {
        return Err("model turns did not overlap".into());
    }
    let protocol = Command::new(verifier.canonicalize()?)
        .arg("--audit")
        .arg(root.join("events.jsonl"))
        .output()?;
    if !protocol.status.success() {
        return Err(format!(
            "protocol audit: {}",
            String::from_utf8_lossy(&protocol.stderr)
        )
        .into());
    }
    let protocol: Value = serde_json::from_slice(&protocol.stdout)?;
    let total: u64 = participants
        .iter()
        .map(|v| v["inclusive_tokens"].as_u64().unwrap())
        .sum();
    Ok(
        json!({"status":"passed","task":fixture["task"],"kind":"unscored_direct_wasi_model_preflight",
        "scored_benchmark":false,"participants":participants,"model_calls":maximum,"max_simultaneous_model_turns":maximum,
        "inclusive_tokens":total,"protocol":protocol,"events_sha256":result["events_sha256"],
        "protocol_verifier_sha256":digest(verifier)?,"model_verifier_sha256":digest(&std::env::current_exe()?)?}),
    )
}

pub fn check_mutants(root: &Path, output: &Path, verifier: &Path) -> Result<Value> {
    run(root, verifier)?;
    fs::create_dir(output)?;
    if output.canonicalize()?.starts_with(root.canonicalize()?) {
        return Err("mutant output must be outside original evidence".into());
    }
    fn copy(from: &Path, to: &Path) -> Result<()> {
        fs::create_dir(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let target = to.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy(&entry.path(), &target)?;
            } else {
                fs::copy(entry.path(), target)?;
            }
        }
        Ok(())
    }
    let actors = records(&root.join("events.jsonl"))?;
    let agent = actors[0]["message"]["fixtures"][0]["agent"]
        .as_str()
        .ok_or("first actor")?;
    let mut checked = Vec::new();
    for kind in [
        "forged_model",
        "incorrect_inclusive_usage",
        "forged_command",
        "missing_tool_reply",
        "false_simulator_score",
        "changed_upstream_source",
    ] {
        let case = output.join(kind);
        copy(root, &case)?;
        let events = case.join(agent).join("events.jsonl");
        let mut rows = records(&events)?;
        let mut participant = load(&case.join(agent).join("participant.json"))?;
        let mut result = load(&case.join("result.json"))?;
        match kind {
            "forged_model" => {
                participant["requested_model"] = json!("other");
                participant["returned_model"] = json!("other");
                for row in &mut rows {
                    let message = &mut row["message"];
                    if ["thread/start", "turn/start"]
                        .contains(&message["method"].as_str().unwrap_or(""))
                    {
                        message["params"]["model"] = json!("other");
                    }
                    if message["id"] == 2 && message.get("method").is_none() {
                        message["result"]["model"] = json!("other");
                    }
                }
            }
            "incorrect_inclusive_usage" => {
                let total = participant["usage"]["totalTokens"]
                    .as_u64()
                    .ok_or("usage")?
                    + 1;
                participant["usage"]["totalTokens"] = json!(total);
                for row in &mut rows {
                    if row["message"]["method"] == "thread/tokenUsage/updated" {
                        row["message"]["params"]["tokenUsage"]["total"]["totalTokens"] =
                            json!(total);
                    }
                }
            }
            "forged_command" => {
                rows.iter_mut()
                    .find(|r| r["message"]["method"] == "item/tool/call")
                    .ok_or("tool call")?["message"]["params"]["arguments"]["command"] =
                    json!("echo forged");
            }
            "missing_tool_reply" => {
                let id = rows
                    .iter()
                    .find(|r| r["message"]["method"] == "item/tool/call")
                    .ok_or("tool call")?["message"]["id"]
                    .clone();
                rows.retain(|r| !(r["direction"] == "send" && r["message"]["id"] == id));
            }
            "false_simulator_score" => {
                let mut runtime = records(&case.join("events.jsonl"))?;
                runtime
                    .iter_mut()
                    .find(|r| r["direction"] == "simulator" && r["message"]["type"] == "score")
                    .ok_or("score")?["message"]["complete"] = json!(false);
                fs::write(
                    case.join("events.jsonl"),
                    runtime.iter().map(|r| format!("{r}\n")).collect::<String>(),
                )?;
                result["events_sha256"] = json!(digest(&case.join("events.jsonl"))?);
            }
            "changed_upstream_source" => {
                let mut manifest = load(&case.join("source-manifest.json"))?;
                let relative = format!(
                    "vendor/tracefix/benchmark/descriptions/{}/description.md",
                    result["task"].as_str().ok_or("task")?
                );
                fs::write(case.join("source").join(&relative), "changed input\n")?;
                manifest[&relative] = json!(digest(&case.join("source").join(&relative))?);
                crate::evidence::save(&case.join("source-manifest.json"), &manifest)?;
                let mut versions = load(&case.join("versions.json"))?;
                versions["source_manifest_sha256"] =
                    json!(digest(&case.join("source-manifest.json"))?);
                crate::evidence::save(&case.join("versions.json"), &versions)?;
                result["versions"] = versions;
            }
            _ => unreachable!(),
        }
        fs::write(
            &events,
            rows.iter().map(|r| format!("{r}\n")).collect::<String>(),
        )?;
        participant["raw_events_sha256"] = json!(digest(&events)?);
        crate::evidence::save(&case.join(agent).join("participant.json"), &participant)?;
        crate::evidence::save(&case.join("result.json"), &result)?;
        let error = run(&case, verifier)
            .expect_err("altered evidence passed independent audit")
            .to_string();
        checked.push(json!({"mutation":kind,"status":"rejected","reason":error,"outer_hashes_recomputed":true}));
    }
    let report = json!({"status":"passed","checked":checked});
    crate::evidence::save(&output.join("report.json"), &report)?;
    Ok(report)
}
