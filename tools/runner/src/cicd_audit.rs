//! Offline checks of actual CI/CD resource transfers and unchanged simulator results.
use crate::evidence::{Result, digest};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

struct Frame {
    identity: Value,
    sent: usize,
    received: Option<usize>,
    queued: bool,
}
fn bytes(frame: &Frame) -> Result<Vec<u8>> {
    frame.identity["payload"]
        .as_array()
        .ok_or("frame payload")?
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "frame byte".into())
        })
        .collect()
}
fn one(frames: &BTreeMap<u64, Frame>, from: u64, to: u64) -> Result<&Frame> {
    let selected: Vec<_> = frames
        .values()
        .filter(|f| f.identity["source"] == from && f.identity["target"] == to)
        .collect();
    if selected.len() != 1 {
        return Err(format!("expected one {from}->{to}, got {}", selected.len()).into());
    }
    Ok(selected[0])
}
fn received(frame: &Frame) -> Result<usize> {
    frame.received.ok_or_else(|| "unreceived frame".into())
}
pub fn audit(path: &Path) -> Result<Value> {
    let records: Vec<Value> = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let projection = crate::projection::load(path)?;
    let mut result = check(&records, &projection)?;
    result["source_events_sha256"] = json!(digest(path)?);
    result["compiled_projection_sha256"] = json!(digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json")
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    result["wire_validation"] =
        json!("static compiled header membership; dynamic eligibility is not replayed");
    Ok(result)
}

fn check(records: &[Value], projection: &Value) -> Result<Value> {
    crate::projection::frames(projection, records)?;
    if projection["compiled"]["task"] != "16E" {
        return Err("CI/CD compiled protocol task".into());
    }
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    if fixture["type"] != "fixtures"
        || fixture["task"] != "16E"
        || fixture["depth"].as_u64().is_none_or(|d| d > 2)
    {
        return Err("CI/CD fixture".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("original actors")?;
    if actors.len() != 3 {
        return Err("CI/CD participant count".into());
    }
    let mut frames = BTreeMap::<u64, Frame>::new();
    let mut calls = BTreeMap::<String, (usize, Value)>::new();
    let mut outstanding = BTreeMap::<u64, String>::new();
    let mut responses = BTreeMap::<String, (usize, Value)>::new();
    let mut exits = BTreeMap::new();
    let mut verdict = None;
    let mut score = None;
    for (index, row) in records.iter().enumerate() {
        let m = &row["message"];
        match (row["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                if m["session"] != 16 {
                    return Err("wrong session".into());
                }
                let id = m["frame_id"].as_u64().ok_or("frame ID")?;
                let mut identity = m.clone();
                identity.as_object_mut().ok_or("frame")?.remove("action");
                match m["action"].as_str() {
                    Some("send") => {
                        if frames
                            .insert(
                                id,
                                Frame {
                                    identity,
                                    sent: index,
                                    received: None,
                                    queued: true,
                                },
                            )
                            .is_some()
                        {
                            return Err("duplicate frame send".into());
                        }
                    }
                    Some("receive" | "requeue") => {
                        let f = frames.get_mut(&id).ok_or("frame without send")?;
                        if f.identity != identity {
                            return Err("altered frame identity".into());
                        }
                        if m["action"] == "receive" && f.queued {
                            f.queued = false;
                            f.received = Some(index);
                        } else if m["action"] == "requeue" && !f.queued && f.received.is_some() {
                            f.queued = true;
                        } else {
                            return Err("invalid frame transition".into());
                        }
                    }
                    _ => return Err("unknown frame observation".into()),
                }
            }
            (Some("runtime"), Some("execute")) => {
                let agent = m["agent"].as_str().ok_or("effect agent")?;
                let actor = actors
                    .iter()
                    .find(|a| a["agent"] == agent)
                    .ok_or("unexpected effect agent")?;
                if actor["calls"].as_array().ok_or("original calls")?.len() != 1
                    || actor["calls"][0]["tool"] != m["tool"]
                    || actor["calls"][0]["arguments"] != m["arguments"]
                    || calls.insert(agent.to_owned(), (index, m.clone())).is_some()
                {
                    return Err("effect differs from original assignment".into());
                }
                let id = m["id"].as_u64().ok_or("effect ID")?;
                if outstanding.insert(id, agent.to_owned()).is_some() {
                    return Err("duplicate effect ID".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let agent = outstanding
                    .remove(&m["id"].as_u64().ok_or("result ID")?)
                    .ok_or("unmatched result")?;
                if m["result"]["success"] != true
                    || responses
                        .insert(agent, (index, m["result"].clone()))
                        .is_some()
                {
                    return Err("bad original result".into());
                }
            }
            (Some("runtime"), Some("exit")) => {
                let agent = m["agent"].as_str().ok_or("exit agent")?;
                if !actors.iter().any(|a| a["agent"] == agent)
                    || m["status"] != 0
                    || exits.insert(agent.to_owned(), index).is_some()
                {
                    return Err("bad guest exit".into());
                }
            }
            (Some("runtime"), Some("run_result")) => {
                if m["status"] != "complete" || verdict.replace(index).is_some() {
                    return Err("bad runtime verdict".into());
                }
            }
            (Some("simulator"), Some("score")) if score.replace(m.clone()).is_some() => {
                return Err("duplicate score".into());
            }
            _ => {}
        }
    }
    if frames.values().any(|f| f.queued || f.received.is_none())
        || calls.len() != 3
        || responses.len() != 3
        || !outstanding.is_empty()
        || exits.len() != 3
        || verdict.is_none()
    {
        return Err("incomplete CI/CD trace".into());
    }
    let score = score.ok_or("missing original score")?;
    if score["complete"] != true
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("score events")?.len() != 3
    {
        return Err("original score mismatch".into());
    }
    let mut intervals = Vec::new();
    for (agent, vm, service, owner, resource) in [
        ("DEVELOPER", 0, 3, 6, 11),
        ("TESTER", 1, 4, 6, 11),
        ("DEPLOYER", 2, 5, 7, 12),
    ] {
        let loan = one(&frames, owner, vm)?;
        let returned = one(&frames, vm, owner)?;
        for f in [loan, returned] {
            let data = bytes(f)?;
            if data.len() != 3 || u64::from(data[0]) != resource {
                return Err("resource Endpoint envelope".into());
            }
        }
        let request = one(&frames, resource, service)?;
        let result = one(&frames, service, resource)?;
        let (effect_at, effect) = calls.get(agent).ok_or("missing effect")?;
        let (response_at, response) = responses.get(agent).ok_or("missing result")?;
        if serde_json::from_slice::<Value>(&bytes(request)?)? != effect["arguments"]
            || serde_json::from_slice::<Value>(&bytes(result)?)? != *response
            || response["tool_name"] != effect["tool"]
            || !(received(loan)? < request.sent
                && received(request)? < *effect_at
                && effect_at < response_at
                && *response_at < result.sent
                && received(result)? < returned.sent
                && returned.sent < *exits.get(agent).ok_or("exit")?)
        {
            return Err("resource request/result/return mismatch".into());
        }
        let closes: Vec<_> = frames
            .values()
            .filter(|f| {
                f.identity["source"] == vm
                    && f.identity["target"] == service
                    && bytes(f).is_ok_and(|b| b == [4])
                    && f.received.is_some_and(|at| at < request.sent)
            })
            .collect();
        if closes.is_empty() {
            return Err("effect without projected WASI close".into());
        }
        let actual = score["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["agent"] == agent)
            .ok_or("original event")?;
        if actual["tool"] != effect["tool"]
            || actual["args"] != effect["arguments"]
            || actual["success"] != response["success"]
            || actual["result"] != response["data"]
        {
            return Err("score event differs from original response".into());
        }
        intervals.push((loan, returned));
    }
    if received(intervals[0].1)? >= intervals[1].0.sent {
        return Err("build server simultaneously loaned".into());
    }
    for (from, to, producer, consumer) in [
        (3, 4, "DEVELOPER", Some("TESTER")),
        (4, 5, "TESTER", Some("DEPLOYER")),
        (5, 3, "DEPLOYER", None),
        (5, 4, "DEPLOYER", None),
    ] {
        let notification = one(&frames, from, to)?;
        if !bytes(notification)?.is_empty() || responses[producer].0 >= notification.sent {
            return Err("notification before original result".into());
        }
        if let Some(consumer) = consumer
            && received(notification)? >= calls[consumer].0
        {
            return Err("dependent effect before notification".into());
        }
    }
    Ok(
        json!({"status":"passed","task":"16E","depth":fixture["depth"],
        "kind":"actual_wasi_resource_delegation","scored_benchmark":false,"model_calls":null,
        "actual_resource_endpoint_requests":3,"actual_resource_endpoint_returns":3,
        "domain_calls":3,"guest_exits":3,"notifications":4,"carrier_frames":frames.len(),
        "simulator_complete":true,"violations":0}),
    )
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    let projection = crate::projection::load(path)?;
    fs::create_dir(output)?;
    let rows: Vec<Value> = fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let mut results = Vec::new();
    for kind in [
        "missing_loan",
        "wrong_loan_resource",
        "missing_request",
        "forged_request",
        "missing_reply",
        "forged_reply",
        "missing_return",
        "overlapping_loans",
        "missing_notification",
        "missing_close",
        "forged_arguments",
        "forged_original_result",
        "missing_exit",
        "false_score",
        "wrong_frame_label",
        "wrong_lane",
    ] {
        let mut copy = rows.clone();
        match kind {
            "wrong_frame_label" | "wrong_lane" => {
                let selected = copy
                    .iter()
                    .find(|row| row["message"]["type"] == "protocol_frame")
                    .ok_or("header mutation frame")?["message"]["frame_id"]
                    .clone();
                let field = if kind == "wrong_frame_label" {
                    "frame_label"
                } else {
                    "lane"
                };
                let changed = if kind == "wrong_frame_label" {
                    65535
                } else {
                    255
                };
                for row in copy.iter_mut().filter(|row| {
                    row["message"]["type"] == "protocol_frame"
                        && row["message"]["frame_id"] == selected
                }) {
                    row["message"][field] = json!(changed);
                }
                // Change every observation together: send/receive equality alone still holds.
            }
            "missing_loan"
            | "wrong_loan_resource"
            | "missing_request"
            | "forged_request"
            | "missing_reply"
            | "forged_reply"
            | "missing_return"
            | "missing_notification" => {
                let (from, to) = match kind {
                    "missing_loan" | "wrong_loan_resource" => (6, 0),
                    "missing_request" | "forged_request" => (11, 3),
                    "missing_reply" | "forged_reply" => (3, 11),
                    "missing_return" => (0, 6),
                    "missing_notification" => (3, 4),
                    _ => unreachable!(),
                };
                let frame = copy
                    .iter()
                    .find(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["source"] == from
                            && r["message"]["target"] == to
                    })
                    .ok_or("mutation frame")?["message"]["frame_id"]
                    .clone();
                if kind.starts_with("missing_") {
                    copy.retain(|r| {
                        !(r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == frame)
                    });
                } else {
                    for row in copy.iter_mut().filter(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == frame
                    }) {
                        if kind == "wrong_loan_resource" {
                            row["message"]["payload"][0] = json!(12);
                        } else {
                            let data = if kind == "forged_request" {
                                json!({"component":"forged"})
                            } else {
                                json!({"success":true,"data":"forged"})
                            };
                            row["message"]["payload"] = json!(serde_json::to_vec(&data)?);
                        }
                    }
                }
            }
            "overlapping_loans" => {
                let is_next_loan = |r: &Value| {
                    r["message"]["type"] == "protocol_frame"
                        && r["message"]["source"] == 6
                        && r["message"]["target"] == 1
                };
                let next: Vec<_> = copy.iter().filter(|r| is_next_loan(r)).cloned().collect();
                if next.is_empty() {
                    return Err("second loan mutation target".into());
                }
                copy.retain(|r| !is_next_loan(r));
                let position = copy
                    .iter()
                    .position(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["action"] == "receive"
                            && r["message"]["source"] == 6
                            && r["message"]["target"] == 0
                    })
                    .ok_or("first loan receipt")?;
                copy.splice(position + 1..position + 1, next);
            }
            "missing_close" => {
                let ids: Vec<_> = copy
                    .iter()
                    .filter(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["source"] == 0
                            && r["message"]["target"] == 3
                            && r["message"]["payload"] == json!([4])
                    })
                    .map(|r| r["message"]["frame_id"].clone())
                    .collect();
                if ids.is_empty() {
                    return Err("close mutation target".into());
                }
                copy.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame"
                        && ids.contains(&r["message"]["frame_id"]))
                });
            }
            "missing_exit" => {
                let index = copy
                    .iter()
                    .position(|r| r["message"]["type"] == "exit")
                    .ok_or("exit mutation target")?;
                copy.remove(index);
            }
            "forged_arguments" => {
                let row = copy
                    .iter_mut()
                    .find(|r| r["message"]["type"] == "execute")
                    .ok_or("effect mutation target")?;
                row["message"]["arguments"] = json!({"component":"forged"});
            }
            "forged_original_result" => {
                let row = copy
                    .iter_mut()
                    .find(|r| r["message"]["type"] == "simulator_result")
                    .ok_or("result mutation target")?;
                row["message"]["result"]["data"] = json!({"forged":true});
            }
            "false_score" => {
                let row = copy
                    .iter_mut()
                    .find(|r| r["message"]["type"] == "score")
                    .ok_or("score mutation target")?;
                row["message"]["complete"] = json!(false);
            }
            _ => unreachable!(),
        }
        let mutant = output.join(format!("{kind}.jsonl"));
        use std::io::Write;
        let mut file = fs::File::create_new(&mutant)?;
        for row in &copy {
            serde_json::to_writer(&mut file, row)?;
            writeln!(file)?;
        }
        drop(file);
        let error = check(&copy, &projection)
            .err()
            .ok_or_else(|| format!("accepted mutant {kind}"))?;
        results.push(json!({"kind":kind,"status":"rejected","error":error.to_string()}));
    }
    let result = json!({"control":"passed","mutants":results});
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}
