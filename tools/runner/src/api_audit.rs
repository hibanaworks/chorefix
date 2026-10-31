//! Independent checks of saved API-task messages and original simulator results.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const OUTCOME_FD: u8 = 16;
fn send(output: &mut impl Write, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
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
pub fn audit(path: &Path) -> Result<Value> {
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
    let binding = |a: &str| -> Result<(u64, u64)> {
        match a {
            "DB_ADMIN" => Ok((0, 5)),
            "BACKEND_DEV" => Ok((1, 6)),
            "TESTER" => Ok((2, 7)),
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
    let mut loans = BTreeMap::<u64, Vec<usize>>::new();
    let mut returns = BTreeMap::<u64, Vec<usize>>::new();
    let mut resource_requests = BTreeMap::<u64, (usize, Value)>::new();
    let mut resource_replies = BTreeMap::<u64, (usize, Value)>::new();
    let mut received_replies = BTreeMap::<u64, usize>::new();
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
                        if matches!((source, target), (3, 5) | (4, 7) | (5, 3) | (7, 4)) {
                            let resource = if source == 3 || target == 3 { 15 } else { 16 };
                            let bytes = payload(m)?;
                            if bytes.len() != 3 || u64::from(bytes[0]) != resource {
                                return Err("invalid Endpoint loan payload".into());
                            }
                            let list = if source == 3 || source == 4 {
                                &mut loans
                            } else {
                                &mut returns
                            };
                            list.entry(resource).or_default().push(index);
                        }
                        if matches!((source, target), (0, 15) | (2, 16)) {
                            let result = serde_json::from_slice(&payload(m)?)?;
                            if resource_replies.insert(target, (index, result)).is_some() {
                                return Err("duplicate resource response".into());
                            }
                        }
                        if (source == 11 && target == 13) || (source == 12 && target == 14) {
                            first_reply.entry(source).or_insert(index);
                            if pending_read.remove(&source) {
                                let bytes = payload(m)?;
                                if bytes.len() < 4 || bytes[3] as usize != bytes.len() - 4 {
                                    return Err("invalid result chunk length".into());
                                }
                                if bytes[0] != OUTCOME_FD || bytes[1..3] != [0, 0] {
                                    return Err("result read failed".into());
                                }
                                let expected = notification
                                    .get(&source)
                                    .ok_or("read before notification")?
                                    .1
                                    .clone();
                                let canonical = serde_json::to_vec(&expected)?;
                                let current = read_bytes.entry(source).or_default();
                                current.extend_from_slice(&bytes[4..]);
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
                        if matches!((source, target), (15, 0) | (16, 2)) {
                            let arguments = serde_json::from_slice(&payload(m)?)?;
                            if resource_requests
                                .insert(source, (index, arguments))
                                .is_some()
                            {
                                return Err("duplicate resource request".into());
                            }
                        }
                        if matches!((source, target), (0, 15) | (2, 16)) {
                            received_replies.insert(target, index);
                        }
                        if ((source == 5 && target == 0)
                            || (source == 6 && target == 1)
                            || (source == 7 && target == 2))
                            && m["payload"] == json!([4])
                        {
                            closes.insert(target, index);
                        }
                        if ((source == 0 && target == 15)
                            || (source == 1 && target == 6)
                            || (source == 2 && target == 16))
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
                            if bytes.len() != 2 {
                                return Err("invalid result read request".into());
                            }
                            if bytes[0] != OUTCOME_FD || !pending_read.insert(target) {
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
    for (resource, agent, tool) in [
        (15, "DB_ADMIN", "migrate_schema"),
        (16, "TESTER", "run_integration_tests"),
    ] {
        let loan = loans.remove(&resource).ok_or("missing Endpoint loan")?;
        let returned = returns.remove(&resource).ok_or("missing Endpoint return")?;
        let (requested, arguments) = resource_requests
            .remove(&resource)
            .ok_or("missing resource request")?;
        let (replied, result) = resource_replies
            .remove(&resource)
            .ok_or("missing resource response")?;
        let received = received_replies
            .remove(&resource)
            .ok_or("resource response not received")?;
        let (original, response) = responses.get(agent).ok_or("missing original operation")?;
        if loan.len() != 1
            || returned.len() != 1
            || arguments != original["arguments"]
            || &result != response
            || !(loan[0] < requested
                && requested < effect(tool)?
                && done(tool)? < replied
                && replied < received
                && received < returned[0])
        {
            return Err("resource Endpoint request, result or lifetime differs".into());
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
        if fixture["require_early_result_open"] == true
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
        || !loans.is_empty()
        || !returns.is_empty()
        || !resource_requests.is_empty()
        || !resource_replies.is_empty()
        || !received_replies.is_empty()
        || score["complete"] != passes
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("events")?.len() != 3
    {
        return Err("incomplete API boundary evidence".into());
    }
    let test_finished = done("run_integration_tests")?;
    let early_opens = open_sent.values().filter(|i| **i < test_finished).count();
    Ok(
        json!({"task":"8E","depth":depth,"kind":"actual_wasi_resource_delegation","model_calls":null,"scored_benchmark":false,"status":"passed","participants":3,"domain_calls":3,"actual_resource_endpoint_requests":2,"actual_resource_endpoint_returns":2,"simulator_complete":score["complete"],"expected_terminal_test_failure":!passes,"carrier_frames":frames.len(),"result_receipts":2,"read_dependency_observed":true,"result_opens":open_sent.len(),"early_result_opens":early_opens,"complete_result_reads":complete_reads,"violations":0,"guest_exits":3,"original_test_delays":fixture["test_delays"]}),
    )
}
pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
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
        "missing_loan",
        "wrong_loan_resource",
        "missing_resource_request",
        "forged_resource_request",
        "forged_resource_result",
        "missing_resource_return",
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
                    "missing_loan" | "wrong_loan_resource" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 3
                            && m["target"] == 5
                    }
                    "missing_resource_request" | "forged_resource_request" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 15
                            && m["target"] == 0
                    }
                    "forged_resource_result" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 0
                            && m["target"] == 15
                    }
                    "missing_resource_return" => {
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 5
                            && m["target"] == 3
                    }
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
            "missing_loan" | "missing_resource_request" | "missing_resource_return" => {
                let id = copy[index]["message"]["frame_id"].clone();
                copy.retain(|row| {
                    !(row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == id)
                });
            }
            "wrong_loan_resource" | "forged_resource_request" | "forged_resource_result" => {
                let id = copy[index]["message"]["frame_id"].clone();
                let mut bytes = payload(&copy[index]["message"])?;
                match kind {
                    "wrong_loan_resource" => bytes[0] = 16,
                    "forged_resource_request" => {
                        bytes = serde_json::to_vec(&json!({"migration_name":"wrong_target"}))?
                    }
                    "forged_resource_result" => {
                        let mut response: Value = serde_json::from_slice(&bytes)?;
                        response["data"]["forged"] = json!(true);
                        bytes = serde_json::to_vec(&response)?;
                    }
                    _ => unreachable!(),
                }
                for row in &mut copy {
                    if row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == id
                    {
                        row["message"]["payload"] = json!(bytes);
                    }
                }
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
