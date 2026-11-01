//! Inspect persisted observations. This code never authorizes a task operation.
use crate::evidence::Result;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
};

struct Frame {
    identity: Value,
    queued: bool,
    received: usize,
}

fn bytes(message: &Value) -> Result<Vec<u8>> {
    message["payload"]
        .as_array()
        .ok_or("payload bytes")?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "payload byte".into())
        })
        .collect()
}

pub fn audit(path: &Path) -> Result<Value> {
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(serde_json::from_str::<Value>(&line?)?))
        .collect::<Result<Vec<_>>>()?;
    let projection = crate::projection::load(path)?;
    let mut result = check(&records, &projection)?;
    result["source_events_sha256"] = json!(crate::evidence::digest(path)?);
    result["compiled_projection_sha256"] = json!(crate::evidence::digest(
        &path
            .parent()
            .ok_or("event directory")?
            .join("projection.json"),
    )?);
    result["runtime_binary_sha256"] = projection["runtime_binary_sha256"].clone();
    result["wire_validation"] =
        json!("static compiled header membership; dynamic eligibility is not replayed");
    Ok(result)
}

fn check(records: &[Value], projection: &Value) -> Result<Value> {
    use crate::philosopher_identity::{roles9e as e, roles9h as h, roles9m as m};
    crate::projection::frames(projection, records)?;
    if projection["compiled"]["task"] != records.first().ok_or("empty evidence")?["message"]["task"]
    {
        return Err("compiled projection belongs to a different task".into());
    }
    let fixture = &records.first().ok_or("empty evidence")?["message"];
    let actors = fixture["fixtures"].as_array().ok_or("fixtures")?;
    if fixture["terminal_review_failure"] != false {
        return Err("invalid dining trial configuration".into());
    }
    let layout: &[(&str, [u8; 5])] = match fixture["task"].as_str() {
        Some("9E") => &[
            (
                "FORK_A",
                [e::A, e::FORK_A, e::A_WASI, e::A_IO, e::FORK_A_RESOURCE],
            ),
            (
                "FORK_B",
                [e::B, e::FORK_B, e::B_WASI, e::B_IO, e::FORK_B_RESOURCE],
            ),
            (
                "FORK_C",
                [e::C, e::FORK_C, e::C_WASI, e::C_IO, e::FORK_C_RESOURCE],
            ),
        ],
        Some("9M") => &[
            (
                "FORK_A",
                [m::A, m::FORK_A, m::A_WASI, m::A_IO, m::FORK_A_RESOURCE],
            ),
            (
                "FORK_B",
                [m::B, m::FORK_B, m::B_WASI, m::B_IO, m::FORK_B_RESOURCE],
            ),
            (
                "FORK_C",
                [m::C, m::FORK_C, m::C_WASI, m::C_IO, m::FORK_C_RESOURCE],
            ),
            (
                "FORK_D",
                [m::D, m::FORK_D, m::D_WASI, m::D_IO, m::FORK_D_RESOURCE],
            ),
            (
                "FORK_E",
                [m::E, m::FORK_E, m::E_WASI, m::E_IO, m::FORK_E_RESOURCE],
            ),
        ],
        Some("9H") => &[
            (
                "FORK_A",
                [h::A, h::FORK_A, h::A_WASI, h::A_IO, h::FORK_A_RESOURCE],
            ),
            (
                "FORK_B",
                [h::B, h::FORK_B, h::B_WASI, h::B_IO, h::FORK_B_RESOURCE],
            ),
            (
                "FORK_C",
                [h::C, h::FORK_C, h::C_WASI, h::C_IO, h::FORK_C_RESOURCE],
            ),
            (
                "FORK_D",
                [h::D, h::FORK_D, h::D_WASI, h::D_IO, h::FORK_D_RESOURCE],
            ),
            (
                "FORK_E",
                [h::E, h::FORK_E, h::E_WASI, h::E_IO, h::FORK_E_RESOURCE],
            ),
            (
                "FORK_F",
                [h::F, h::FORK_F, h::F_WASI, h::F_IO, h::FORK_F_RESOURCE],
            ),
            (
                "FORK_G",
                [h::G, h::FORK_G, h::G_WASI, h::G_IO, h::FORK_G_RESOURCE],
            ),
        ],
        _ => return Err("unknown dining task".into()),
    };
    if actors.len() != layout.len() {
        return Err("participant count".into());
    }
    let roles: BTreeSet<_> = layout
        .iter()
        .flat_map(|(_, roles)| roles)
        .map(|&role| u64::from(role))
        .collect();
    if roles.len() != 5 * layout.len() {
        return Err("dining role identities collide".into());
    }
    let forks: BTreeMap<_, _> = layout
        .iter()
        .map(|&(name, [_, owner, _, _, resource])| (name, (u64::from(owner), u64::from(resource))))
        .collect();
    let assignments: BTreeMap<_, _> = actors
        .iter()
        .enumerate()
        .map(|(index, actor)| {
            Ok((
                actor["agent"].as_str().ok_or("actor name")?,
                (u64::from(layout[index].1[0]), actor),
            ))
        })
        .collect::<Result<_>>()?;
    // These maps reconstruct the saved trace. They are not runtime permissions.
    let mut held: BTreeMap<u64, Option<u64>> = forks
        .values()
        .map(|&(owner, resource)| (resource, Some(owner)))
        .collect();
    let mut in_flight = BTreeMap::<u64, u64>::new();
    let mut loans = BTreeMap::<u64, usize>::new();
    let mut returns = BTreeMap::<u64, usize>::new();
    let mut frames = BTreeMap::<u64, Frame>::new();
    let mut requests = BTreeMap::<(u64, String), Vec<(u64, Value)>>::new();
    let mut delivered = BTreeMap::<(u64, String), Vec<Value>>::new();
    let mut meals = BTreeSet::<(u64, u64)>::new();
    let mut calls = BTreeMap::<&str, usize>::new();
    let mut outstanding = BTreeMap::<u64, Value>::new();
    let mut responses = BTreeMap::<(u64, String), Value>::new();
    let mut active = BTreeMap::<u64, BTreeSet<String>>::new();
    let mut maximum_eaters = 0;
    let mut exits = BTreeSet::new();
    let mut verdict = false;
    let mut score = None;
    for record in records {
        let message = &record["message"];
        match (record["direction"].as_str(), message["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                if message["session"] != crate::philosopher_identity::SESSION {
                    return Err("frame belongs to another dining session".into());
                }
                u8::try_from(message["lane"].as_u64().ok_or("frame lane")?)
                    .map_err(|_| "frame lane exceeds its wire width")?;
                u16::try_from(message["frame_label"].as_u64().ok_or("frame label")?)
                    .map_err(|_| "frame label exceeds its wire width")?;
                let id = message["frame_id"].as_u64().ok_or("frame id")?;
                let source = message["source"].as_u64().ok_or("source")?;
                let target = message["target"].as_u64().ok_or("target")?;
                if !roles.contains(&source) || !roles.contains(&target) {
                    return Err("frame refers to a role absent from this dining protocol".into());
                }
                let payload = bytes(message)?;
                let mut identity = message.clone();
                identity
                    .as_object_mut()
                    .ok_or("frame object")?
                    .remove("action");
                let first_receive = match message["action"].as_str() {
                    Some("send") => {
                        if frames
                            .insert(
                                id,
                                Frame {
                                    identity: identity.clone(),
                                    queued: true,
                                    received: 0,
                                },
                            )
                            .is_some()
                        {
                            return Err("duplicate frame send".into());
                        }
                        false
                    }
                    Some("receive" | "requeue") => {
                        let frame = frames.get_mut(&id).ok_or("frame has no send")?;
                        if frame.identity != identity {
                            return Err("altered frame identity".into());
                        }
                        if message["action"] == "receive" {
                            if !frame.queued {
                                return Err("duplicate receive".into());
                            }
                            frame.queued = false;
                            frame.received += 1;
                            frame.received == 1
                        } else {
                            if frame.queued || frame.received == 0 {
                                return Err("invalid requeue".into());
                            }
                            frame.queued = true;
                            false
                        }
                    }
                    _ => return Err("unknown frame action".into()),
                };
                if forks
                    .values()
                    .any(|&(owner, _)| owner == source || owner == target)
                {
                    if payload.len() != 3 {
                        return Err("owned Endpoint envelope".into());
                    }
                    let resource = u64::from(payload[0]);
                    let owner = forks
                        .values()
                        .find(|&&(_, r)| r == resource)
                        .ok_or("unknown resource")?
                        .0;
                    let vm = if source == owner {
                        target
                    } else if target == owner {
                        source
                    } else {
                        return Err("wrong resource owner".into());
                    };
                    let actor = actors
                        .get(vm.checked_sub(14).ok_or("guest role")? as usize)
                        .ok_or("guest role")?;
                    if !actor["forks"]
                        .as_array()
                        .ok_or("assigned forks")?
                        .iter()
                        .any(|name| {
                            forks
                                .get(name.as_str().unwrap_or(""))
                                .is_some_and(|&(_, r)| r == resource)
                        })
                    {
                        return Err("Endpoint delivered to unassigned philosopher".into());
                    }
                    if message["action"] == "send" {
                        if held.insert(resource, None) != Some(Some(source))
                            || in_flight.insert(resource, id).is_some()
                        {
                            return Err("duplicate or unheld Endpoint transfer".into());
                        }
                        if target == owner {
                            if !meals.remove(&(resource, vm - 14)) {
                                return Err("return precedes actual meal reply".into());
                            }
                            *returns.entry(resource).or_default() += 1;
                        } else {
                            *loans.entry(resource).or_default() += 1;
                        }
                    } else if first_receive
                        && (in_flight.remove(&resource) != Some(id)
                            || held.insert(resource, Some(target)) != Some(None))
                    {
                        return Err("Endpoint receive has no matching transfer".into());
                    }
                }
                if first_receive && target < actors.len() as u64 {
                    let arguments = serde_json::from_slice::<Value>(&payload).ok();
                    if source == target + 14
                        && arguments
                            .as_ref()
                            .is_some_and(|v| v["philosopher"].is_string())
                    {
                        requests
                            .entry((target, "think".into()))
                            .or_default()
                            .push((source, arguments.unwrap()));
                    } else if held.contains_key(&source) {
                        if held[&source] != Some(target + 14) {
                            return Err("request without held resource".into());
                        }
                        requests
                            .entry((target, "eat".into()))
                            .or_default()
                            .push((source, arguments.ok_or("actual resource request JSON")?));
                    }
                }
                if first_receive
                    && source < actors.len() as u64
                    && (target == source + 14 || held.contains_key(&target))
                {
                    let result: Value = serde_json::from_slice(&payload)?;
                    let tool = result["tool_name"]
                        .as_str()
                        .ok_or("actual result tool")?
                        .to_owned();
                    if held.contains_key(&target) {
                        if tool != "eat"
                            || held[&target] != Some(source + 14)
                            || !meals.insert((target, source))
                        {
                            return Err("meal reply without its unique resource".into());
                        }
                    } else if tool != "think" {
                        return Err("result bypassed resource Endpoint".into());
                    }
                    delivered.entry((source, tool)).or_default().push(result);
                }
            }
            (Some("runtime"), Some("execute")) => {
                let agent = message["agent"].as_str().ok_or("execute agent")?;
                let &(role, assignment) = assignments.get(agent).ok_or("unknown execute agent")?;
                let tool = message["tool"].as_str().ok_or("execute tool")?;
                let count = calls.entry(agent).or_default();
                if assignment["calls"]
                    .get(*count)
                    .ok_or("duplicate task effect")?["tool"]
                    != tool
                    || !message["arguments"]["philosopher"].is_string()
                {
                    return Err("task fidelity mismatch".into());
                }
                let observed = requests
                    .remove(&(role, tool.to_owned()))
                    .ok_or("effect without actual projected request")?;
                if observed
                    .iter()
                    .any(|(_, arguments)| arguments != &message["arguments"])
                {
                    return Err("request/effect argument mismatch".into());
                }
                if tool == "think" {
                    if observed.len() != 1 || observed[0].0 != role + 14 {
                        return Err("think request count".into());
                    }
                } else if tool == "eat" {
                    let required = assignment["forks"]
                        .as_array()
                        .ok_or("forks")?
                        .iter()
                        .map(|name| {
                            forks
                                .get(name.as_str().ok_or("fork name")?)
                                .map(|&(_, r)| r)
                                .ok_or_else(|| "unknown fork".into())
                        })
                        .collect::<Result<BTreeSet<_>>>()?;
                    if observed.len() != 2
                        || observed.iter().map(|&(r, _)| r).collect::<BTreeSet<_>>() != required
                        || required.iter().any(|r| held[r] != Some(role + 14))
                        || delivered.get(&(role, "think".into())).map(Vec::len) != Some(1)
                    {
                        return Err(
                            "effect without both unique Endpoints and completed think".into()
                        );
                    }
                    let names = assignment["forks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect::<BTreeSet<_>>();
                    if active.values().any(|other| !other.is_disjoint(&names)) {
                        return Err("overlapping physical forks".into());
                    }
                    active.insert(message["id"].as_u64().ok_or("execute id")?, names);
                    maximum_eaters = maximum_eaters.max(active.len());
                } else {
                    return Err("unknown effect".into());
                }
                *count += 1;
                if outstanding
                    .insert(message["id"].as_u64().ok_or("execute id")?, message.clone())
                    .is_some()
                {
                    return Err("duplicate execute id".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let id = message["id"].as_u64().ok_or("result id")?;
                let call = outstanding
                    .remove(&id)
                    .ok_or("unmatched simulator result")?;
                if call["tool"] != message["result"]["tool_name"]
                    || message["result"]["success"] != true
                {
                    return Err("unexpected simulator result".into());
                }
                let role = assignments[call["agent"].as_str().ok_or("agent")?].0;
                if responses
                    .insert(
                        (role, call["tool"].as_str().unwrap().to_owned()),
                        message["result"].clone(),
                    )
                    .is_some()
                {
                    return Err("duplicate domain result".into());
                }
                active.remove(&id);
            }
            (Some("runtime"), Some("exit")) => {
                if message["status"] != 0
                    || !exits.insert(message["agent"].as_str().ok_or("exit agent")?.to_owned())
                {
                    return Err("failed or duplicate guest exit".into());
                }
            }
            (Some("runtime"), Some("run_result")) => {
                if message["status"] != "complete" || verdict {
                    return Err("failed or duplicate runtime verdict".into());
                }
                verdict = true;
            }
            (Some("simulator"), Some("score")) if score.is_none() => {
                score = Some(message.clone());
            }
            (Some("simulator"), Some("score")) => return Err("duplicate score".into()),
            _ => {}
        }
    }
    let score = score.ok_or("missing score")?;
    for (&agent, &(role, assignment)) in &assignments {
        for tool in ["think", "eat"] {
            let result = responses
                .get(&(role, tool.to_owned()))
                .ok_or("missing effect result")?;
            let values = delivered
                .remove(&(role, tool.to_owned()))
                .ok_or("missing received result")?;
            if values.len() != if tool == "eat" { 2 } else { 1 }
                || values.iter().any(|v| v != result)
            {
                return Err("effect and delivered result disagree".into());
            }
            let events = score["events"]
                .as_array()
                .ok_or("score events")?
                .iter()
                .filter(|e| e["agent"] == agent && e["tool"] == tool)
                .collect::<Vec<_>>();
            let expected_args = if tool == "think" {
                json!({})
            } else {
                json!({"left_fork":assignment["forks"][0],"right_fork":assignment["forks"][1]})
            };
            if events.len() != 1
                || events[0]["args"] != expected_args
                || events[0]["result"] != result["data"]
                || events[0]["success"] != result["success"]
                || result["data"]["philosopher"] != agent
            {
                return Err("original simulator result mismatch".into());
            }
        }
    }
    let expected_parallel = actors.len() / 2;
    if !verdict
        || exits.len() != actors.len()
        || calls.len() != actors.len()
        || calls.values().any(|&n| n != 2)
        || !outstanding.is_empty()
        || !requests.is_empty()
        || !delivered.is_empty()
        || !meals.is_empty()
        || !in_flight.is_empty()
        || !active.is_empty()
        || frames.values().any(|f| f.queued || f.received == 0)
        || forks.values().any(|&(owner, resource)| {
            held[&resource] != Some(owner)
                || loans.get(&resource) != Some(&2)
                || returns.get(&resource) != Some(&2)
        })
        || score["complete"] != true
        || score["violations"] != json!([])
        || score["events"].as_array().ok_or("events")?.len() != 2 * actors.len()
        || maximum_eaters == 0
        || maximum_eaters > expected_parallel
    {
        return Err("incomplete delegation boundary evidence".into());
    }
    Ok(
        json!({"task":fixture["task"],"kind":"actual_wasi_resource_delegation","model_calls":null,"scored_benchmark":false,"status":"passed","participants":actors.len(),"domain_calls":2*actors.len(),"carrier_frames":frames.len(),"max_disjoint_eaters":maximum_eaters,"simulator_complete":true,"actual_resource_endpoint_requests":2*actors.len(),"actual_resource_endpoint_returns":2*actors.len(),"violations":0,"guest_exits":exits.len(),"original_test_delays":fixture["test_delays"]}),
    )
}

fn send(output: &mut impl Write, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    Ok(())
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    let projection = crate::projection::load(path)?;
    fs::create_dir(output)?;
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(serde_json::from_str::<Value>(&line?)?))
        .collect::<Result<Vec<_>>>()?;
    let mut checked = Vec::new();
    for mutation in [
        "wrong_frame_label",
        "wrong_lane",
        "missing_request",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_exit",
        "false_score",
        "response_before_second_request",
        "consistent_unknown_private_role",
        "consistent_wrong_session",
    ] {
        let mut copy = records.clone();
        if matches!(mutation, "wrong_frame_label" | "wrong_lane") {
            let frame = copy
                .iter()
                .find(|row| row["message"]["type"] == "protocol_frame")
                .ok_or("frame for header mutation")?["message"]
                .clone();
            let field = if mutation == "wrong_frame_label" {
                "frame_label"
            } else {
                "lane"
            };
            let replacement = if mutation == "wrong_frame_label" {
                65535
            } else {
                (frame["lane"].as_u64().ok_or("observed lane")? + 1) % 256
            };
            for row in &mut copy {
                if row["message"]["type"] == "protocol_frame"
                    && row["message"]["frame_id"] == frame["frame_id"]
                {
                    row["message"][field] = json!(replacement);
                }
            }
        } else {
            let index = copy
                .iter()
                .position(|r| {
                    let m = &r["message"];
                    match mutation {
                        "missing_request" => {
                            r["direction"] == "runtime"
                                && m["type"] == "protocol_frame"
                                && m["action"] == "receive"
                                && m["source"] == 28
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
                        "response_before_second_request" => true,
                        "consistent_unknown_private_role" => {
                            r["direction"] == "runtime"
                                && m["type"] == "protocol_frame"
                                && m["source"].as_u64().is_some_and(|role| role >= 14)
                                && m["target"].as_u64().is_some_and(|role| role >= 14)
                        }
                        "consistent_wrong_session" => m["type"] == "protocol_frame",
                        _ => false,
                    }
                })
                .ok_or("mutation target missing")?;
            match mutation {
                "missing_request" | "missing_exit" => {
                    copy.remove(index);
                }
                "altered_frame" => copy[index]["message"]["source"] = json!(254),
                "forged_arguments" => {
                    copy[index]["message"]["arguments"] = json!({"philosopher":42})
                }
                "forged_result" => {
                    copy[index]["message"]["result"]["data"] = json!({"forged":true})
                }
                "false_score" => {
                    copy[index]["message"]["complete"] = json!(
                        !copy[index]["message"]["complete"]
                            .as_bool()
                            .ok_or("score completion")?
                    )
                }
                "response_before_second_request" => {
                    use crate::philosopher_identity::{roles9e, roles9h, roles9m};
                    let right = match records[0]["message"]["task"].as_str() {
                        Some("9E") => roles9e::FORK_C_RESOURCE,
                        Some("9M") => roles9m::FORK_E_RESOURCE,
                        Some("9H") => roles9h::FORK_G_RESOURCE,
                        _ => return Err("unknown dining task".into()),
                    };
                    let request = records
                        .iter()
                        .find(|r| {
                            let m = &r["message"];
                            m["type"] == "protocol_frame"
                                && m["action"] == "send"
                                && m["source"] == right
                                && m["target"] == 0
                        })
                        .ok_or("actual right-resource request")?["message"]["frame_id"]
                        .clone();
                    let moved = copy
                        .iter()
                        .filter(|r| {
                            r["message"]["type"] == "protocol_frame"
                                && r["message"]["frame_id"] == request
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    copy.retain(|r| {
                        !(r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == request)
                    });
                    let after = copy
                        .iter()
                        .position(|r| {
                            let m = &r["message"];
                            m["type"] == "protocol_frame"
                                && m["action"] == "receive"
                                && m["source"] == 0
                                && m["target"] == roles9e::FORK_A_RESOURCE
                        })
                        .ok_or("actual meal receipt")?
                        + 1;
                    copy.splice(after..after, moved);
                }
                "consistent_unknown_private_role" => {
                    let id = copy[index]["message"]["frame_id"].clone();
                    for row in &mut copy {
                        if row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == id
                        {
                            row["message"]["source"] = json!(254);
                        }
                    }
                }
                "consistent_wrong_session" => {
                    let id = copy[index]["message"]["frame_id"].clone();
                    for row in &mut copy {
                        if row["message"]["type"] == "protocol_frame"
                            && row["message"]["frame_id"] == id
                        {
                            row["message"]["session"] =
                                json!(crate::philosopher_identity::SESSION + 1);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        let mutant_path = output.join(format!("{mutation}.jsonl"));
        let mut file = File::create(&mutant_path)?;
        for record in &copy {
            send(&mut file, record)?;
        }
        let error = check(&copy, &projection)
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
