//! Deterministic external stimuli and persisted-event audit, never task authority.
//! Python is used only to call the unchanged original Python simulator.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn send(output: &mut impl Write, message: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, message)?;
    writeln!(output)?;
    output.flush()?;
    Ok(())
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
// Fixed external stimuli, not task admission or progress authority.
fn stimuli(fixture: &Value) -> Result<VecDeque<(String, String)>> {
    let mut commands = VecDeque::new();
    let call = &fixture["calls"][0];
    let tool = call["tool"].as_str().ok_or("tool")?;
    commands.push_back((
        "echo '{}' > /service/an_unknown_operation".into(),
        "wash:".into(),
    ));
    commands.push_back((
        "x=3; expr $x + 4 > /work/note; cat /work/note".into(),
        "7\n".into(),
    ));
    commands.push_back((
        format!("echo '{{}}' > /service/{tool}; cat /service/result"),
        "invalid_arguments".into(),
    ));
    let key = call["arguments"]
        .as_object()
        .ok_or("arguments")?
        .keys()
        .next()
        .ok_or("key")?;
    commands.push_back((
        format!(
            "echo {} > /service/{tool}; cat /service/result",
            quote(&json!({key:"wrong_target"}).to_string())
        ),
        "invalid_arguments".into(),
    ));
    commands.push_back((
        format!(
            "printf '%s' {} > /service/{tool}; cat /service/result",
            quote(&format!("{}{}", " ".repeat(512), call["arguments"]))
        ),
        format!("\"tool_name\":\"{tool}\""),
    ));
    commands.push_back((
        format!(
            "echo {} > /service/{tool}",
            quote(&call["arguments"].to_string())
        ),
        "wash:".into(),
    ));
    if fixture["agent"] != "TESTER" {
        commands.push_back((
            "cat /service/outcome".into(),
            "\"tool_name\":\"run_integration_tests\"".into(),
        ));
        commands.push_back(("echo wrong > /service/outcome".into(), "wash:".into()));
        commands.push_back((
            "cat /service/outcome".into(),
            "\"tool_name\":\"run_integration_tests\"".into(),
        ));
    }
    commands.push_back(("cat /work/note".into(), "7\n".into()));
    commands.push_back(("exit 0".into(), String::new()));
    Ok(commands)
}
fn run(task: &str, output: &Path) -> Result<()> {
    let depth: u8 = task.parse()?;
    if depth > 2 {
        return Err("invalid depth".into());
    }
    let task = "8E";
    fs::create_dir(output)?;
    let mut evidence = File::create(output.join("events.jsonl"))?;
    let started = Instant::now();
    let (sender, receiver) = mpsc::channel();
    let mut simulator_command = Command::new(".venv/bin/python");
    simulator_command.env("PYTHONHASHSEED", "0").args([
        "tools/original_simulator.py",
        task,
        "boundary",
        &depth.to_string(),
    ]);
    let mut simulator = Process(
        simulator_command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(output.join("simulator-stderr.log"))?)
            .spawn()?,
    );
    let sim_stdout = simulator.0.stdout.take().ok_or("simulator stdout")?;
    let sim_sender = sender.clone();
    std::thread::spawn(move || {
        for line in BufReader::new(sim_stdout).lines() {
            if sim_sender.send(("simulator", line)).is_err() {
                break;
            }
        }
    });
    let (direction, line) = receiver.recv_timeout(Duration::from_secs(10))?;
    let fixtures: Value = serde_json::from_str(&line?)?;
    if fixtures["type"] != "fixtures" {
        return Err("missing simulator fixtures".into());
    }
    send(
        &mut evidence,
        &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "direction":direction, "message":fixtures}),
    )?;
    let mut scripts: BTreeMap<_, _> = fixtures["fixtures"]
        .as_array()
        .ok_or("fixtures array")?
        .iter()
        .map(|f| Ok((f["agent"].as_str().ok_or("agent")?.to_owned(), stimuli(f)?)))
        .collect::<Result<_>>()?;
    let mut runtime = Process(
        Command::new("target/debug/api-wasi")
            .args(["vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(output.join("runtime-stderr.log"))?)
            .spawn()?,
    );
    let runtime_stdout = runtime.0.stdout.take().ok_or("runtime stdout")?;
    std::thread::spawn(move || {
        for line in BufReader::new(runtime_stdout).lines() {
            if sender.send(("runtime", line)).is_err() {
                break;
            }
        }
    });
    let sim_input = simulator.0.stdin.as_mut().ok_or("simulator stdin")?;
    let runtime_input = runtime.0.stdin.as_mut().ok_or("runtime stdin")?;
    let mut expected = BTreeMap::<u64, String>::new();
    let mut command_id = 0u64;
    let mut early_reads = std::collections::BTreeSet::new();
    let mut test_request = None;
    loop {
        if started.elapsed() > Duration::from_secs(120) {
            return Err("boundary timeout".into());
        }
        let (direction, line) = receiver.recv_timeout(Duration::from_secs(30))?;
        let message: Value = serde_json::from_str(&line?)?;
        send(
            &mut evidence,
            &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "direction":direction, "message":message}),
        )?;
        match (direction, message["type"].as_str()) {
            ("runtime", Some("protocol_frame")) => {
                if message["action"] == "send"
                    && [13, 14].into_iter().any(|r| message["source"] == r)
                {
                    early_reads.insert(message["source"].as_u64().ok_or("outcome requester")?);
                }
                if early_reads.len() == 2
                    && let Some(call) = test_request.take()
                {
                    send(sim_input, &call)?;
                }
            }
            ("runtime", Some("execute")) => {
                if message["tool"] == "run_integration_tests" && early_reads.len() != 2 {
                    if test_request.replace(message.clone()).is_some() {
                        return Err("duplicate test stimulus".into());
                    }
                } else {
                    send(sim_input, &message)?;
                }
            }
            ("simulator", Some("simulator_result")) => send(runtime_input, &message)?,
            ("runtime", Some("terminal" | "exit")) => {
                if let Some(id) = message["id"].as_u64() {
                    let needle = expected.remove(&id).ok_or("unknown terminal reply")?;
                    let stdout = message["stdout"]
                        .as_str()
                        .ok_or("missing terminal output")?;
                    let normalized = stdout.replace(' ', "");
                    if !normalized.contains(&needle) {
                        return Err(format!(
                            "unexpected terminal output: expected {needle:?}, got {stdout:?}"
                        )
                        .into());
                    }
                }
                if message["type"] == "exit" {
                    if message["status"] != 0 {
                        return Err("guest exit failed".into());
                    }
                    continue;
                }
                let agent = message["agent"].as_str().ok_or("terminal agent")?;
                let (command, needle) = scripts
                    .get_mut(agent)
                    .ok_or("unknown agent")?
                    .pop_front()
                    .ok_or("extra terminal read")?;
                command_id += 1;
                expected.insert(command_id, needle);
                let input =
                    json!({"type":"command", "id":command_id, "agent":agent, "command":command});
                send(
                    &mut evidence,
                    &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "direction":"to_runtime", "message":input}),
                )?;
                send(runtime_input, &input)?;
            }
            ("runtime", Some("run_result")) => {
                if message["status"] != "complete" {
                    return Err(format!("runtime failed: {message}").into());
                }
                if !expected.is_empty() || scripts.values().any(|s| !s.is_empty()) {
                    return Err("unexecuted shell stimuli".into());
                }
                send(sim_input, &json!({"type":"score"}))?;
            }
            ("simulator", Some("score")) => break,
            _ => return Err(format!("unknown boundary event: {direction} {message}").into()),
        }
    }
    if !runtime.0.wait()?.success() || !simulator.0.wait()?.success() {
        return Err("boundary subprocess failed".into());
    }
    let audit = audit(&output.join("events.jsonl"))?;
    fs::write(
        output.join("audit.json"),
        format!("{}\n", serde_json::to_string_pretty(&audit)?),
    )?;
    println!("{}", serde_json::to_string(&audit)?);
    Ok(())
}

fn payload(message: &Value) -> Result<Vec<u8>> {
    message["payload"]
        .as_array()
        .ok_or("frame bytes")?
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "invalid frame byte".into())
        })
        .collect()
}
fn audit(path: &Path) -> Result<Value> {
    use choreofix::tasks::api::choreofs::global::*;
    use hibana::runtime::wire::{Payload, WirePayload};
    use hibana_wasip1_runtime::protocol;
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|l| Ok(serde_json::from_str::<Value>(&l?)?))
        .collect::<Result<Vec<_>>>()?;
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    if fixture["task"] != "8E" || fixture["fixtures"].as_array().ok_or("actors")?.len() != 3 {
        return Err("unexpected API task".into());
    }
    let depth = fixture["depth"].as_u64().ok_or("depth")?;
    if depth > 2 {
        return Err("depth".into());
    }
    let passes = depth == 0;
    if fixture["model_run"] == true && fixture["test_delays"] != json!({}) {
        return Err("model trial has artificial delays".into());
    }
    let binding = |a: &str| -> Result<(u64, u64)> {
        match a {
            "DB_ADMIN" => Ok((u64::from(ADMIN), u64::from(ADMIN_VM))),
            "BACKEND_DEV" => Ok((u64::from(BACKEND), u64::from(BACKEND_VM))),
            "TESTER" => Ok((u64::from(TESTER), u64::from(TESTER_VM))),
            _ => Err("unknown actor".into()),
        }
    };
    let actors = fixture["fixtures"].as_array().unwrap();
    let mut frames = BTreeMap::<u64, (Value, i32, usize, usize)>::new();
    let mut outstanding = BTreeMap::new();
    let mut calls = BTreeSet::new();
    let mut effects = BTreeMap::new();
    let mut completed = BTreeMap::new();
    let mut closes = BTreeMap::new();
    let mut delivered = BTreeMap::new();
    let mut responses = BTreeMap::new();
    let mut sent_resources = BTreeMap::<(u64, u64), Vec<usize>>::new();
    let mut replies_resources = BTreeMap::<(u64, u64), Vec<usize>>::new();
    let mut notification = BTreeMap::<u64, (usize, Value)>::new();
    let mut open_sent = BTreeMap::new();
    let mut first_reply = BTreeMap::new();
    let mut pending_read = BTreeSet::new();
    let mut read_bytes = BTreeMap::<u64, Vec<u8>>::new();
    let mut complete_reads = BTreeMap::<u64, usize>::new();
    let mut exits = BTreeSet::new();
    let mut verdict = false;
    let mut score = None;
    for (index, r) in records.iter().enumerate() {
        let m = &r["message"];
        match (r["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                if m["session"] != 28 {
                    return Err("wrong session".into());
                }
                let id = m["frame_id"].as_u64().ok_or("frame ID")?;
                let mut identity = m.clone();
                identity.as_object_mut().unwrap().remove("action");
                let row = frames.entry(id).or_insert((identity.clone(), 0, 0, 0));
                if row.0 != identity {
                    return Err("altered frame identity".into());
                }
                let source = m["source"].as_u64().ok_or("source")?;
                let target = m["target"].as_u64().ok_or("target")?;
                match m["action"].as_str() {
                    Some("send") if row.1 == 0 && row.2 == 0 => {
                        row.1 = 1;
                        row.2 = 1;
                        if (source == 13 && target == 11) || (source == 14 && target == 12) {
                            open_sent.entry(target).or_insert(index);
                        }
                        if (target == 3 || target == 4) && source <= 2 {
                            if m["payload"] != json!([]) {
                                return Err("resource request data".into());
                            }
                            sent_resources
                                .entry((source, target))
                                .or_default()
                                .push(index);
                        }
                        if (source == 3 || source == 4) && target <= 2 {
                            if m["payload"] != json!([]) {
                                return Err("resource grant data".into());
                            }
                            replies_resources
                                .entry((target, source))
                                .or_default()
                                .push(index);
                        }
                        if (source == 11 && target == 13) || (source == 12 && target == 14) {
                            first_reply.entry(source).or_insert(index);
                            if pending_read.remove(&source) {
                                let bytes = payload(m)?;
                                protocol::FdReadDoneRet::validate_payload(Payload::new(&bytes))
                                    .map_err(|e| format!("invalid read reply: {e:?}"))?;
                                let decoded = protocol::FdReadDoneRet::decode_validated_payload(
                                    Payload::new(&bytes),
                                );
                                if decoded.0.fd() != OUTCOME_FD || decoded.0.errno() != 0 {
                                    return Err("result read failed".into());
                                }
                                let expected = notification
                                    .get(&source)
                                    .ok_or("read before notification")?
                                    .1
                                    .clone();
                                let canonical = serde_json::to_vec(&expected)?;
                                let current = read_bytes.entry(source).or_default();
                                current.extend_from_slice(decoded.0.as_bytes());
                                if !canonical.starts_with(current.as_slice()) {
                                    return Err("result file bytes differ from notification".into());
                                }
                                if *current == canonical {
                                    *complete_reads.entry(source).or_default() += 1;
                                    current.clear();
                                }
                            }
                        }
                    }
                    Some("receive") if row.1 == 1 => {
                        row.1 = 0;
                        row.3 += 1;
                        if ((source == 5 && target == 0)
                            || (source == 6 && target == 1)
                            || (source == 7 && target == 2))
                            && m["payload"] == json!([4])
                        {
                            closes.insert(target, index);
                        }
                        if ((source == 0 && target == 5)
                            || (source == 1 && target == 6)
                            || (source == 2 && target == 7))
                            && let Ok(value) = serde_json::from_slice::<Value>(&payload(m)?)
                            && value["tool_name"].is_string()
                            && delivered.insert(source, value).is_some()
                        {
                            return Err("duplicate delivered result".into());
                        }
                        if source == 2 && (target == 11 || target == 12) {
                            let value: Value = serde_json::from_slice(&payload(m)?)?;
                            if notification.insert(target, (index, value)).is_some() {
                                return Err("duplicate result notification".into());
                            }
                        }
                        if ((source == 13 && target == 11) || (source == 14 && target == 12))
                            && payload(m)?.len() == 2
                        {
                            let bytes = payload(m)?;
                            protocol::FdReadReq::validate_payload(Payload::new(&bytes))
                                .map_err(|e| format!("invalid result read: {e:?}"))?;
                            let decoded =
                                protocol::FdReadReq::decode_validated_payload(Payload::new(&bytes));
                            if decoded.0.fd() != OUTCOME_FD || !pending_read.insert(target) {
                                return Err("overlapping result read".into());
                            }
                        }
                    }
                    Some("requeue") if row.1 == 0 && row.3 > 0 => row.1 = 1,
                    _ => return Err("invalid carrier transition".into()),
                }
            }
            (Some("runtime"), Some("execute")) => {
                let agent = m["agent"].as_str().ok_or("agent")?;
                let (role, _) = binding(agent)?;
                let original = actors
                    .iter()
                    .find(|a| a["agent"] == agent)
                    .ok_or("fixture agent")?;
                if !calls.insert(agent.to_owned())
                    || original["calls"][0]["tool"] != m["tool"]
                    || original["calls"][0]["arguments"] != m["arguments"]
                {
                    return Err("task fidelity mismatch".into());
                }
                if closes.remove(&role).is_none() {
                    return Err("effect without fresh projected close".into());
                }
                let id = m["id"].as_u64().ok_or("execute ID")?;
                if outstanding.insert(id, m.clone()).is_some() {
                    return Err("duplicate execution".into());
                }
                effects.insert(m["tool"].as_str().unwrap().to_owned(), index);
            }
            (Some("simulator"), Some("simulator_result")) => {
                let id = m["id"].as_u64().ok_or("reply ID")?;
                let call = outstanding.remove(&id).ok_or("unmatched result")?;
                let tool = call["tool"].as_str().unwrap();
                let success = tool != "run_integration_tests" || passes;
                if m["result"]["tool_name"] != tool || m["result"]["success"] != success {
                    return Err("incorrect original outcome".into());
                }
                responses.insert(
                    call["agent"].as_str().unwrap().to_owned(),
                    (call.clone(), m["result"].clone()),
                );
                completed.insert(tool.to_owned(), index);
            }
            (Some("runtime"), Some("exit")) => {
                let a = m["agent"].as_str().ok_or("exit")?;
                binding(a)?;
                if m["status"] != 0 || !exits.insert(a.to_owned()) {
                    return Err("bad guest exit".into());
                }
            }
            (Some("runtime"), Some("run_result")) => {
                if verdict || m["status"] != "complete" {
                    return Err("bad runtime verdict".into());
                }
                verdict = true;
            }
            (Some("simulator"), Some("score")) if score.replace(m.clone()).is_some() => {
                return Err("duplicate score".into());
            }
            _ => {}
        }
    }
    let score = score.ok_or("missing score")?;
    let effect = |tool: &str| effects.get(tool).copied().ok_or("missing effect");
    let done = |tool: &str| completed.get(tool).copied().ok_or("missing completion");
    if !(done("migrate_schema")? < effect("implement_backend")?
        && done("implement_backend")? < effect("run_integration_tests")?)
    {
        return Err("broken task dependency".into());
    }
    for (owner, resource, tool) in [(0, 3, "migrate_schema"), (2, 4, "run_integration_tests")] {
        let s = sent_resources
            .remove(&(owner, resource))
            .ok_or("missing resource requests")?;
        let g = replies_resources
            .remove(&(owner, resource))
            .ok_or("missing resource replies")?;
        if s.len() != 2
            || g.len() != 2
            || !(s[0] < g[0] && g[0] < effect(tool)? && done(tool)? < s[1] && s[1] < g[1])
        {
            return Err("invalid resource lifetime".into());
        }
    }
    for (agent, (call, response)) in &responses {
        let (role, _) = binding(agent)?;
        if delivered.remove(&role).as_ref() != Some(response) {
            return Err("task reply differs from effect".into());
        }
        let events = score["events"]
            .as_array()
            .ok_or("events")?
            .iter()
            .filter(|e| e["agent"] == *agent && e["tool"] == call["tool"])
            .collect::<Vec<_>>();
        if events.len() != 1
            || events[0]["args"] != call["arguments"]
            || events[0]["success"] != response["success"]
            || events[0]["result"] != response["data"]
        {
            return Err("original journal mismatch".into());
        }
    }
    let test_result = &responses.get("TESTER").ok_or("tester result")?.1;
    for owner in [11, 12] {
        let (notified, value) = notification
            .get(&owner)
            .ok_or("missing outcome notification")?;
        if value != test_result
            || *notified <= done("run_integration_tests")?
            || first_reply.get(&owner).is_none_or(|i| i <= notified)
            || complete_reads.get(&owner).is_none_or(|n| *n < 1)
        {
            return Err("result read escaped its test dependency".into());
        }
        if fixture["model_run"] != true
            && open_sent
                .get(&owner)
                .is_none_or(|i| *i >= done("run_integration_tests").unwrap())
        {
            return Err("early-read stimulus was not exercised".into());
        }
    }
    if !verdict
        || calls.len() != 3
        || exits.len() != 3
        || responses.len() != 3
        || !outstanding.is_empty()
        || !delivered.is_empty()
        || !pending_read.is_empty()
        || read_bytes.values().any(|v| !v.is_empty())
        || frames.values().any(|r| r.1 != 0 || r.2 != 1 || r.3 == 0)
        || !sent_resources.is_empty()
        || !replies_resources.is_empty()
        || score["complete"] != passes
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("events")?.len() != 3
    {
        return Err("incomplete API boundary evidence".into());
    }
    Ok(
        json!({"task":"8E","depth":depth,"kind":if fixture["model_run"]==true{"model_direct_wasi_protocol"}else{"deterministic_direct_wasi_boundary"},"model_calls":if fixture["model_run"]==true{Value::Null}else{json!(0)},"scored_benchmark":false,"status":"passed","participants":3,"domain_calls":3,"simulator_complete":score["complete"],"expected_terminal_test_failure":!passes,"carrier_frames":frames.len(),"result_receipts":2,"read_dependency_enforced":true,"early_result_opens":open_sent.len(),"complete_result_reads":complete_reads,"violations":0,"guest_exits":3,"original_test_delays":fixture["test_delays"]}),
    )
}
fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    fs::create_dir(output)?;
    let rows = BufReader::new(File::open(path)?)
        .lines()
        .map(|l| Ok(serde_json::from_str::<Value>(&l?)?))
        .collect::<Result<Vec<_>>>()?;
    let mut reports = Vec::new();
    for kind in [
        "missing_close",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_notification",
        "missing_read",
        "missing_exit",
        "false_score",
    ] {
        let mut copy = rows.clone();
        let index = copy
            .iter()
            .position(|r| {
                let m = &r["message"];
                match kind {
                    "missing_close" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["source"] == 5
                            && m["target"] == 0
                            && m["payload"] == json!([4])
                    }
                    "altered_frame" => m["type"] == "protocol_frame" && m["action"] == "receive",
                    "forged_arguments" => m["type"] == "execute",
                    "forged_result" => m["type"] == "simulator_result",
                    "missing_notification" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["source"] == 2
                            && m["target"] == 11
                    }
                    "missing_read" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 11
                            && m["target"] == 13
                            && m["payload"].as_array().is_some_and(|b| b.len() > 5)
                    }
                    "missing_exit" => m["type"] == "exit",
                    "false_score" => m["type"] == "score",
                    _ => false,
                }
            })
            .ok_or("mutation target")?;
        match kind {
            "missing_close" | "missing_notification" | "missing_read" | "missing_exit" => {
                copy.remove(index);
            }
            "altered_frame" => copy[index]["message"]["source"] = json!(254),
            "forged_arguments" => {
                copy[index]["message"]["arguments"] = json!({"component":"wrong"})
            }
            "forged_result" => copy[index]["message"]["result"]["data"] = json!({"forged":true}),
            "false_score" => {
                copy[index]["message"]["complete"] =
                    json!(!copy[index]["message"]["complete"].as_bool().unwrap())
            }
            _ => unreachable!(),
        }
        let file = output.join(format!("{kind}.jsonl"));
        let mut out = File::create(&file)?;
        for r in &copy {
            send(&mut out, r)?
        }
        let error = audit(&file).err().ok_or("accepted a mutant")?.to_string();
        reports.push(json!({"mutation":kind,"rejected":true,"error":error}));
    }
    let report = json!({"mutants":reports,"original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.as_slice() {
        [mode, input] if mode == "--audit" => println!("{}", audit(Path::new(input))?),
        [mode, input, output] if mode == "--check-mutants" => {
            println!("{}", check_mutants(Path::new(input), Path::new(output))?)
        }
        [depth, output] => run(depth, Path::new(output))?,
        _ => {
            return Err(
                "usage: api-boundary DEPTH OUTPUT | --audit EVENTS | --check-mutants EVENTS OUTPUT"
                    .into(),
            );
        }
    }
    Ok(())
}
