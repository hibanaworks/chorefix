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
// A planned list of test inputs is not a second runtime permission mechanism.
fn stimuli(fixture: &Value) -> Result<VecDeque<(String, String)>> {
    let mut commands = VecDeque::new();
    commands.push_back((
        "echo '{}' > /service/write_to_document".into(),
        "wash:".into(),
    ));
    commands.push_back((
        "x=3; expr $x + 4 > /work/note; cat /work/note".into(),
        "7\n".into(),
    ));
    for call in fixture["calls"].as_array().ok_or("missing calls")? {
        let tool = call["tool"].as_str().ok_or("missing tool")?;
        commands.push_back((
            format!("echo '{{}}' > /service/{tool}; cat /service/result"),
            "invalid_arguments".into(),
        ));
        let key = call["arguments"]
            .as_object()
            .ok_or("argument object")?
            .keys()
            .next()
            .ok_or("argument key")?;
        let wrong = serde_json::to_string(&json!({key: "a_different_object"}))?;
        commands.push_back((
            format!(
                "echo {} > /service/{tool}; cat /service/result",
                quote(&wrong)
            ),
            "invalid_arguments".into(),
        ));
        let arguments = serde_json::to_string(&call["arguments"])?;
        let padded = format!("{}{}", " ".repeat(512), arguments);
        commands.push_back((
            format!(
                "printf '%s' {} > /service/{tool}; cat /service/result",
                quote(&padded)
            ),
            format!("\"tool_name\":\"{tool}\""),
        ));
        commands.push_back((
            format!("echo {} > /service/{tool}", quote(&arguments)),
            "wash:".into(),
        ));
    }
    commands.push_back(("cat /work/note".into(), "7\n".into()));
    commands.push_back(("exit 0".into(), String::new()));
    Ok(commands)
}

fn run(task: &str, output: &Path) -> Result<()> {
    let (task, scenario) = task.split_once(':').unwrap_or((task, "normal"));
    if scenario != "normal" && scenario != "terminal-review-failure" {
        return Err("unknown boundary scenario".into());
    }
    fs::create_dir(output)?;
    let mut evidence = File::create(output.join("events.jsonl"))?;
    let started = Instant::now();
    let (sender, receiver) = mpsc::channel();
    let mut simulator_command = Command::new(".venv/bin/python");
    simulator_command.args(["tools/original_writing_simulator.py", task]);
    if scenario == "terminal-review-failure" {
        simulator_command.arg(scenario);
    }
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
        Command::new("target/debug/writing-wasi")
            .args([
                "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
                task,
            ])
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
            ("runtime", Some("protocol_frame")) => {}
            ("runtime", Some("execute")) => send(sim_input, &message)?,
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

fn audit(path: &Path) -> Result<Value> {
    use choreofix::tasks::writing::choreofs::global::{roles2e, roles2h, roles2m};
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(serde_json::from_str::<Value>(&line?)?))
        .collect::<Result<Vec<_>>>()?;
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    let actors = fixture["fixtures"].as_array().ok_or("fixtures")?;
    let terminal_review_failure = fixture["terminal_review_failure"]
        .as_bool()
        .ok_or("terminal failure scenario")?;
    let sections: BTreeMap<&str, u64> = match fixture["task"].as_str() {
        Some("2E") => [
            ("INTRO_SECTION", roles2e::INTRO),
            ("METHODS_SECTION", roles2e::METHODS),
            ("RESULTS_SECTION", roles2e::RESULTS),
        ]
        .into_iter()
        .map(|(k, v)| (k, u64::from(v)))
        .collect(),
        Some("2M") => [
            ("BACKGROUND_SECTION", roles2m::BACKGROUND),
            ("INTRO_SECTION", roles2m::INTRO),
            ("METHODS_SECTION", roles2m::METHODS),
            ("EXPERIMENTS_SECTION", roles2m::EXPERIMENTS),
            ("RESULTS_SECTION", roles2m::RESULTS),
        ]
        .into_iter()
        .map(|(k, v)| (k, u64::from(v)))
        .collect(),
        Some("2H") => [
            ("BACKGROUND_SECTION", roles2h::BACKGROUND),
            ("INTRO_SECTION", roles2h::INTRO),
            ("RELATED_SECTION", roles2h::RELATED),
            ("METHODS_SECTION", roles2h::METHODS),
            ("EXPERIMENTS_SECTION", roles2h::EXPERIMENTS),
            ("RESULTS_SECTION", roles2h::RESULTS),
            ("DISCUSSION_SECTION", roles2h::DISCUSSION),
        ]
        .into_iter()
        .map(|(k, v)| (k, u64::from(v)))
        .collect(),
        _ => return Err("unknown writing task".into()),
    };
    let assignments: BTreeMap<_, _> = actors
        .iter()
        .enumerate()
        .map(|(i, a)| (a["agent"].as_str().unwrap(), (i as u64, a)))
        .collect();
    let mut calls = BTreeMap::<&str, usize>::new();
    let mut outstanding = BTreeMap::<u64, &Value>::new();
    let mut closes = BTreeMap::<u64, &Value>::new();
    let mut frames = BTreeMap::<u64, (Value, i64, usize, usize)>::new();
    let mut active = BTreeMap::<u64, BTreeSet<&str>>::new();
    let mut maximum_writes = 0;
    let mut responses = BTreeMap::new();
    let mut delivered = BTreeMap::new();
    let mut resource_sends = BTreeMap::<(u64, u64), Vec<usize>>::new();
    let mut resource_reply_sends = BTreeMap::<(u64, u64), Vec<usize>>::new();
    let mut resource_receives = BTreeMap::<(u64, u64), Vec<usize>>::new();
    let mut effects = BTreeMap::new();
    let mut completions = BTreeMap::new();
    let mut exits = BTreeSet::new();
    let mut score = None;
    let mut verdict = false;
    for (index, record) in records.iter().enumerate() {
        let message = &record["message"];
        match (record["direction"].as_str(), message["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                let id = message["frame_id"].as_u64().ok_or("frame id")?;
                let mut identity = message.clone();
                identity.as_object_mut().unwrap().remove("action");
                let row = frames.entry(id).or_insert((identity.clone(), 0, 0, 0));
                if row.0 != identity {
                    return Err("altered frame identity".into());
                }
                match message["action"].as_str() {
                    Some("send") if row.2 == 0 && row.1 == 0 => {
                        row.1 = 1;
                        row.2 += 1;
                        let source = message["source"].as_u64().ok_or("frame source")?;
                        let target = message["target"].as_u64().ok_or("frame target")?;
                        if source < actors.len() as u64 && sections.values().any(|&r| r == target) {
                            if message["payload"] != json!([]) {
                                return Err("resource request payload".into());
                            }
                            resource_sends
                                .entry((source, target))
                                .or_default()
                                .push(index);
                        }
                        if target < actors.len() as u64 && sections.values().any(|&r| r == source) {
                            resource_reply_sends
                                .entry((target, source))
                                .or_default()
                                .push(index);
                        }
                    }
                    Some("receive") if row.1 == 1 => {
                        row.1 = 0;
                        row.3 += 1;
                        let source = message["source"].as_u64().ok_or("frame source")?;
                        let target = message["target"].as_u64().ok_or("frame target")?;
                        if target < actors.len() as u64
                            && sections.values().any(|&r| r == source)
                            && row.3 == 1
                        {
                            if message["payload"] != json!([]) {
                                return Err("resource reply payload".into());
                            }
                            resource_receives
                                .entry((target, source))
                                .or_default()
                                .push(index);
                        }
                        if message["target"]
                            .as_u64()
                            .is_some_and(|t| (14..=20).contains(&t))
                            && message["source"].as_u64().unwrap() + 14
                                == message["target"].as_u64().unwrap()
                        {
                            let bytes = message["payload"]
                                .as_array()
                                .ok_or("payload bytes")?
                                .iter()
                                .map(|v| {
                                    v.as_u64()
                                        .and_then(|n| u8::try_from(n).ok())
                                        .ok_or("payload byte")
                                })
                                .collect::<std::result::Result<Vec<_>, _>>()?;
                            if let Ok(value) = serde_json::from_slice::<Value>(&bytes)
                                && let Some(tool) = value["tool_name"].as_str()
                            {
                                let key = (message["source"].as_u64().unwrap(), tool.to_owned());
                                if delivered.insert(key, value).is_some() {
                                    return Err("duplicate delivered domain result".into());
                                }
                            }
                        }
                        if message["source"]
                            .as_u64()
                            .is_some_and(|s| (14..=20).contains(&s))
                            && message["source"].as_u64().unwrap() - 14
                                == message["target"].as_u64().unwrap()
                            && message["payload"] == json!([4])
                        {
                            closes.insert(message["target"].as_u64().unwrap(), message);
                        }
                    }
                    Some("requeue") if row.1 == 0 && row.3 > 0 => {
                        row.1 = 1;
                    }
                    _ => return Err("invalid carrier transition".into()),
                }
            }
            (Some("runtime"), Some("execute")) => {
                let agent = message["agent"].as_str().ok_or("execute agent")?;
                let &(role, assignment) = assignments.get(agent).ok_or("unknown execute agent")?;
                let next = calls.entry(agent).or_default();
                let expected = assignment["calls"]
                    .get(*next)
                    .ok_or("duplicate task operation")?;
                if expected["tool"] != message["tool"]
                    || expected["arguments"] != message["arguments"]
                {
                    return Err("task fidelity mismatch".into());
                }
                if closes.remove(&role).is_none() {
                    return Err("effect without a fresh projected WASI close".into());
                }
                let id = message["id"].as_u64().ok_or("execute id")?;
                if outstanding.insert(id, message).is_some() {
                    return Err("duplicate execute id".into());
                }
                *next += 1;
                effects.insert((role, message["tool"].as_str().unwrap()), index);
                if message["tool"] == "write_to_document" {
                    let sections = assignment["sections"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect::<BTreeSet<_>>();
                    if active.values().any(|other| !other.is_disjoint(&sections)) {
                        return Err("overlapping physical sections".into());
                    }
                    active.insert(id, sections);
                    maximum_writes = maximum_writes.max(active.len());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let id = message["id"].as_u64().ok_or("result id")?;
                let call = outstanding
                    .remove(&id)
                    .ok_or("unmatched simulator result")?;
                let expected_success =
                    !(terminal_review_failure && call["tool"] == "review_own_work");
                if call["tool"] != message["result"]["tool_name"]
                    || message["result"]["success"] != expected_success
                {
                    return Err("unexpected original simulator result".into());
                }
                let key = (
                    call["agent"].as_str().unwrap().to_owned(),
                    call["tool"].as_str().unwrap().to_owned(),
                );
                if responses
                    .insert(key, (call.clone(), message["result"].clone()))
                    .is_some()
                {
                    return Err("duplicate domain result".into());
                }
                let role = assignments.get(call["agent"].as_str().unwrap()).unwrap().0;
                completions.insert((role, call["tool"].as_str().unwrap()), index);
                active.remove(&id);
            }
            (Some("runtime"), Some("exit")) => {
                let agent = message["agent"].as_str().ok_or("exit agent")?;
                if message["status"] != 0 {
                    return Err("failed guest exit".into());
                }
                if !exits.insert(agent) {
                    return Err("duplicate guest exit".into());
                }
            }
            (Some("runtime"), Some("run_result")) => {
                if message["status"] != "complete" || verdict {
                    return Err("failed or duplicate runtime verdict".into());
                }
                verdict = true;
            }
            (Some("simulator"), Some("score")) => {
                let previous = score.replace(message);
                if previous.is_some() {
                    return Err("duplicate score".into());
                }
            }
            _ => {}
        }
    }
    let score = score.ok_or("missing original simulator score")?;
    for (role, assignment) in assignments.values() {
        let mut acquisition_indices = Vec::new();
        let mut grant_indices = Vec::new();
        let mut return_indices = Vec::new();
        let mut returned_indices = Vec::new();
        for section in assignment["sections"]
            .as_array()
            .ok_or("assigned sections")?
        {
            let section_role = *sections
                .get(section.as_str().ok_or("section name")?)
                .ok_or("unknown section")?;
            let sent = resource_sends
                .remove(&(*role, section_role))
                .ok_or("missing resource requests")?;
            let received = resource_receives
                .remove(&(*role, section_role))
                .ok_or("missing resource replies")?;
            let replies = resource_reply_sends
                .remove(&(*role, section_role))
                .ok_or("missing sent resource replies")?;
            if sent.len() != 2 || received.len() != 3 || replies.len() != 3 {
                return Err("incorrect affine resource visit count".into());
            }
            let draft = *completions
                .get(&(*role, "draft_content"))
                .ok_or("draft completion")?;
            let write = *effects
                .get(&(*role, "write_to_document"))
                .ok_or("write effect")?;
            let written = *completions
                .get(&(*role, "write_to_document"))
                .ok_or("write completion")?;
            let review = *effects
                .get(&(*role, "review_own_work"))
                .ok_or("review effect")?;
            if !(draft < sent[0]
                && received[0] < sent[0]
                && sent[0] < received[1]
                && received[1] < write
                && write < written
                && written < sent[1]
                && sent[1] < received[2]
                && received[2] < review)
            {
                return Err("resource lifetime and task dependency disagree".into());
            }
            acquisition_indices.push(sent[0]);
            grant_indices.push(replies[1]);
            return_indices.push(sent[1]);
            returned_indices.push(replies[2]);
        }
        if acquisition_indices.len() != 2
            || acquisition_indices.iter().max() >= grant_indices.iter().min()
        {
            return Err("parallel acquisition was serialized before grant waits".into());
        }
        if return_indices.iter().max() >= returned_indices.iter().min() {
            return Err("parallel return was serialized before acknowledgements".into());
        }
    }
    if !resource_sends.is_empty()
        || !resource_receives.is_empty()
        || !resource_reply_sends.is_empty()
    {
        return Err("unexpected resource participant".into());
    }
    for ((agent, tool), (call, response)) in &responses {
        let matches = score["events"]
            .as_array()
            .ok_or("sim events")?
            .iter()
            .filter(|event| event["agent"] == *agent && event["tool"] == *tool)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err("missing or duplicate original simulator event".into());
        }
        let event = matches[0];
        let arguments = match tool.as_str() {
            "read_literature" => call["arguments"].clone(),
            "draft_content" | "review_own_work" => {
                json!({"section":call["arguments"]["section_name"]})
            }
            "write_to_document" => {
                json!({"sections":call["arguments"]["sections"].as_str().ok_or("sections")?.split(',').map(str::trim).collect::<Vec<_>>()})
            }
            _ => return Err("unexpected domain tool".into()),
        };
        let role = assignments.get(agent.as_str()).ok_or("response agent")?.0;
        if event["result"] != response["data"]
            || event["success"] != response["success"]
            || event["args"] != arguments
            || delivered.remove(&(role, tool.clone())).as_ref() != Some(response)
        {
            return Err("simulator effect, result and delivered frame disagree".into());
        }
    }
    let expected_parallel = match fixture["task"].as_str() {
        Some("2E") => 1,
        Some("2M") => 2,
        Some("2H") => 3,
        _ => return Err("unknown task".into()),
    };
    if !verdict
        || exits.len() != actors.len()
        || calls.len() != actors.len()
        || calls.values().any(|&n| n != 4)
        || !outstanding.is_empty()
        || !delivered.is_empty()
        || !active.is_empty()
        || frames.values().any(|r| r.1 != 0 || r.2 != 1 || r.3 == 0)
        || score["complete"] != !terminal_review_failure
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("sim events")?.len() != 4 * actors.len()
        || maximum_writes != expected_parallel
    {
        return Err("incomplete boundary evidence".into());
    }
    Ok(
        json!({"task":fixture["task"], "kind":"deterministic_direct_wasi_boundary", "model_calls":0,
        "scored_benchmark":false, "status":"passed", "participants":actors.len(), "domain_calls":4 * actors.len(),
        "carrier_frames":frames.len(), "max_disjoint_writes":maximum_writes, "simulator_complete":score["complete"],
        "expected_terminal_review_failure":terminal_review_failure,
        "parallel_acquisition_observed":true,
        "parallel_return_observed":true,
        "violations":0, "guest_exits":exits.len(), "original_test_delays":fixture["test_delays"]}),
    )
}

fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    fs::create_dir(output)?;
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(serde_json::from_str::<Value>(&line?)?))
        .collect::<Result<Vec<_>>>()?;
    let mut checked = Vec::new();
    for mutation in [
        "missing_close",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_exit",
        "false_score",
        "serialized_acquisition",
    ] {
        let mut copy = records.clone();
        let index = copy
            .iter()
            .position(|r| {
                let m = &r["message"];
                match mutation {
                    "missing_close" => {
                        r["direction"] == "runtime"
                            && m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["payload"] == json!([4])
                            && m["target"] == 0
                    }
                    "altered_frame" => {
                        r["direction"] == "runtime"
                            && m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                    }
                    "forged_arguments" => m["type"] == "execute",
                    "forged_result" => {
                        r["direction"] == "simulator" && m["type"] == "simulator_result"
                    }
                    "missing_exit" => m["type"] == "exit",
                    "false_score" => m["type"] == "score",
                    "serialized_acquisition" => true,
                    _ => false,
                }
            })
            .ok_or("mutation target missing")?;
        match mutation {
            "missing_close" | "missing_exit" => {
                copy.remove(index);
            }
            "altered_frame" => copy[index]["message"]["source"] = json!(254),
            "forged_arguments" => copy[index]["message"]["arguments"] = json!({"topic":"forged"}),
            "forged_result" => copy[index]["message"]["result"]["data"] = json!({"forged":true}),
            "false_score" => {
                copy[index]["message"]["complete"] = json!(
                    !copy[index]["message"]["complete"]
                        .as_bool()
                        .ok_or("score completion")?
                )
            }
            "serialized_acquisition" => {
                // For A, the first/second physical section roles are 7/8 at
                // all levels. Move the second request/grant exchange after
                // receipt of the first grant, preserving every frame identity.
                let frame = |source, target, ordinal| {
                    records
                        .iter()
                        .filter(|r| {
                            let m = &r["message"];
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == source
                                && m["target"] == target
                        })
                        .nth(ordinal)
                        .map(|r| r["message"]["frame_id"].clone())
                        .ok_or("resource mutation frame")
                };
                let second_request = frame(json!(0), json!(8), 0)?;
                let second_grant = frame(json!(8), json!(0), 1)?;
                let first_grant = frame(json!(7), json!(0), 1)?;
                let moved = copy
                    .iter()
                    .filter(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && (r["message"]["frame_id"] == second_request
                                || r["message"]["frame_id"] == second_grant)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                copy.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame"
                        && (r["message"]["frame_id"] == second_request
                            || r["message"]["frame_id"] == second_grant))
                });
                let after = copy
                    .iter()
                    .position(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["action"] == "receive"
                            && r["message"]["frame_id"] == first_grant
                    })
                    .ok_or("first grant receipt")?
                    + 1;
                copy.splice(after..after, moved);
            }
            _ => unreachable!(),
        }
        let mutant_path = output.join(format!("{mutation}.jsonl"));
        let mut file = File::create(&mutant_path)?;
        for record in &copy {
            send(&mut file, record)?;
        }
        let error = audit(&mutant_path)
            .err()
            .ok_or("audit accepted mutated evidence")?
            .to_string();
        checked.push(json!({"mutation":mutation, "rejected":true, "error":error}));
    }
    let result = json!({"mutants":checked, "original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    Ok(result)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, path, output] if mode == "--check-mutants" => {
            println!("{}", check_mutants(Path::new(path), Path::new(output))?);
            Ok(())
        }
        [mode, path] if mode == "--audit" => {
            println!("{}", audit(Path::new(path))?);
            Ok(())
        }
        [task, output] if ["2E", "2M", "2H"].contains(&task.split(':').next().unwrap()) => {
            run(task, Path::new(output))
        }
        _ => Err("usage: writing-boundary 2E|2M|2H OUTPUT | --audit EVENTS.jsonl".into()),
    }
}
