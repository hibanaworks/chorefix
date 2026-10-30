//! Checks completed 6E WASI histories against resource transfers and original
//! simulator calls. This module is used only after a run.
use crate::evidence::{Result, digest, save};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

struct Frame {
    identity: Value,
    bytes: Vec<u8>,
    sent: usize,
    received: Option<usize>,
    queued: bool,
}
impl Frame {
    fn direction(&self, from: u64, to: u64) -> bool {
        self.identity["source"] == from && self.identity["target"] == to
    }
    fn received(&self) -> Result<usize> {
        self.received.ok_or_else(|| "unreceived frame".into())
    }
    fn value(&self) -> Result<Value> {
        Ok(serde_json::from_slice(&self.bytes)?)
    }
}
struct Call {
    request: Value,
    emitted: usize,
    started: Option<(usize, f64)>,
    result: Option<(usize, Value)>,
    domain_result: Option<Value>,
    ended: Option<(usize, f64)>,
}
impl Call {
    fn started(&self) -> Result<(usize, f64)> {
        self.started.ok_or_else(|| "missing domain start".into())
    }
    fn ended(&self) -> Result<(usize, f64)> {
        self.ended.ok_or_else(|| "missing domain end".into())
    }
    fn result(&self) -> Result<(usize, &Value)> {
        self.result
            .as_ref()
            .map(|(at, value)| (*at, value))
            .ok_or_else(|| "missing original result".into())
    }
    fn tool(&self) -> Result<&str> {
        self.request["tool"]
            .as_str()
            .ok_or_else(|| "tool name".into())
    }
    fn target(&self) -> Result<&str> {
        let key = match self.tool()? {
            "implement_feature" => "feature_name",
            "code_review" => "pr_id",
            "merge_code" => "branch",
            _ => return Err("unexpected review-task tool".into()),
        };
        self.request["arguments"][key]
            .as_str()
            .ok_or_else(|| "effect target".into())
    }
}
fn one<'a>(items: impl Iterator<Item = &'a Frame>) -> Result<&'a Frame> {
    let mut items = items;
    let frame = items.next().ok_or("missing matching frame")?;
    if items.next().is_some() {
        return Err("multiple matching frames".into());
    }
    Ok(frame)
}
fn records(path: &Path) -> Result<Vec<Value>> {
    fs::read_to_string(path)?
        .lines()
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}
fn time(row: &Value) -> Result<f64> {
    row["elapsed_seconds"]
        .as_f64()
        .filter(|t| t.is_finite() && *t >= 0.0)
        .ok_or_else(|| "invalid observation time".into())
}
fn close_arguments(
    frames: &BTreeMap<u64, Frame>,
    vm: u64,
    io: u64,
    before: usize,
    path: &str,
    arguments: &Value,
) -> Result<()> {
    let closed = frames
        .values()
        .filter(|f| {
            f.direction(io, vm)
                && f.bytes.first() == Some(&4)
                && f.bytes.len() >= 4
                && f.received.is_some_and(|at| at < before)
        })
        .max_by_key(|f| f.received)
        .ok_or("missing actual WASI close result")?;
    let length = usize::from(closed.bytes[3]);
    if closed.bytes[1..3] != [0, 0]
        || closed.bytes.get(4..4 + length) != Some(path.as_bytes())
        || serde_json::from_slice::<Value>(closed.bytes.get(4 + length..).ok_or("close pathname")?)?
            != *arguments
    {
        return Err("WASI close differs from actual business arguments".into());
    }
    if !frames.values().any(|f| {
        f.direction(vm, io) && f.bytes == [4] && f.received.is_some_and(|at| at < closed.sent)
    }) {
        return Err("close result without projected WASI request".into());
    }
    Ok(())
}

fn check(rows: &[Value]) -> Result<Value> {
    let fixture = &rows.first().ok_or("empty evidence")?["message"];
    if fixture["type"] != "fixtures"
        || fixture["task"] != "6E"
        || fixture["depth"].as_u64().is_none_or(|d| d > 2)
    {
        return Err("6E fixture".into());
    }
    let actors = fixture["fixtures"].as_array().ok_or("original actors")?;
    if actors.len() != 3
        || actors
            .iter()
            .map(|a| a["agent"].as_str().unwrap_or(""))
            .collect::<BTreeSet<_>>()
            != BTreeSet::from(["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"])
    {
        return Err("6E participants".into());
    }
    let mut frames = BTreeMap::<u64, Frame>::new();
    let mut calls = BTreeMap::<u64, Call>::new();
    let mut exits = BTreeSet::new();
    let mut verdict = None;
    let mut score = None;
    for (index, row) in rows.iter().enumerate() {
        let m = &row["message"];
        match (row["direction"].as_str(), m["type"].as_str()) {
            (Some("runtime"), Some("protocol_frame")) => {
                if m["session"] != 6 {
                    return Err("wrong session".into());
                }
                let id = m["frame_id"].as_u64().ok_or("frame ID")?;
                let mut identity = m.clone();
                identity
                    .as_object_mut()
                    .ok_or("frame object")?
                    .remove("action");
                match m["action"].as_str() {
                    Some("send") => {
                        let bytes = m["payload"]
                            .as_array()
                            .ok_or("frame payload")?
                            .iter()
                            .map(|v| {
                                v.as_u64()
                                    .and_then(|v| u8::try_from(v).ok())
                                    .ok_or("frame byte")
                            })
                            .collect::<std::result::Result<Vec<_>, _>>()?;
                        if frames
                            .insert(
                                id,
                                Frame {
                                    identity,
                                    bytes,
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
                        let frame = frames.get_mut(&id).ok_or("frame without send")?;
                        if frame.identity != identity {
                            return Err("altered frame identity".into());
                        }
                        if m["action"] == "receive" && frame.queued {
                            frame.received = Some(index);
                            frame.queued = false;
                        } else if m["action"] == "requeue"
                            && !frame.queued
                            && frame.received.is_some()
                        {
                            frame.queued = true;
                        } else {
                            return Err("invalid frame observation".into());
                        }
                    }
                    _ => return Err("unknown frame action".into()),
                }
            }
            (Some("runtime"), Some("execute")) => {
                let id = m["id"].as_u64().ok_or("effect ID")?;
                let actor = actors
                    .iter()
                    .find(|a| a["agent"] == m["agent"])
                    .ok_or("unknown effect actor")?;
                if !actor["calls"]
                    .as_array()
                    .ok_or("original assignments")?
                    .iter()
                    .any(|a| a["tool"] == m["tool"] && a["arguments"] == m["arguments"])
                {
                    return Err("effect differs from original assignment".into());
                }
                if calls
                    .insert(
                        id,
                        Call {
                            request: m.clone(),
                            emitted: index,
                            started: None,
                            result: None,
                            domain_result: None,
                            ended: None,
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate effect ID".into());
                }
            }
            (Some("domain_start"), Some("execute")) => {
                let call = calls
                    .get_mut(&m["id"].as_u64().ok_or("domain ID")?)
                    .ok_or("domain start without runtime request")?;
                if *m != call.request || call.started.replace((index, time(row)?)).is_some() {
                    return Err("altered domain request".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let call = calls
                    .get_mut(&m["id"].as_u64().ok_or("result ID")?)
                    .ok_or("result without request")?;
                if call.domain_result.as_ref() != Some(&m["result"])
                    || call.result.replace((index, m["result"].clone())).is_some()
                {
                    return Err("duplicate original result".into());
                }
            }
            (Some("domain_end"), _) => {
                let call = calls
                    .get_mut(&m["id"].as_u64().ok_or("domain end ID")?)
                    .ok_or("domain end without request")?;
                if call.request["agent"] != m["agent"]
                    || call.request["tool"] != m["tool"]
                    || call.ended.replace((index, time(row)?)).is_some()
                {
                    return Err("altered domain result".into());
                }
                call.domain_result = Some(m["result"].clone());
            }
            (Some("runtime"), Some("exit")) => {
                let agent = m["agent"].as_str().ok_or("exit agent")?;
                if !actors.iter().any(|a| a["agent"] == agent)
                    || m["status"] != 0
                    || !exits.insert(agent.to_owned())
                {
                    return Err("invalid guest exit".into());
                }
            }
            (Some("runtime"), Some("run_result")) => {
                if m["status"] != "complete" || verdict.replace(index).is_some() {
                    return Err("incomplete runtime verdict".into());
                }
            }
            (Some("simulator"), Some("score")) if score.replace(m).is_some() => {
                return Err("duplicate original score".into());
            }
            _ => {}
        }
    }
    if frames.values().any(|f| f.queued || f.received.is_none())
        || exits.len() != 3
        || verdict.is_none()
    {
        return Err("incomplete observed execution".into());
    }
    let expected = 6 + 2 * fixture["depth"].as_u64().ok_or("depth")? as usize;
    let score = score.ok_or("missing original score")?;
    if score["complete"] != true || score["violations"] != json!([]) || calls.len() != expected {
        return Err("original completion or call count".into());
    }
    let mut scored = score["events"]
        .as_array()
        .ok_or("original scored events")?
        .clone();
    if scored.len() != expected {
        return Err("original event count".into());
    }
    for call in calls.values() {
        let (start, started) = call.started()?;
        let (end, finished) = call.ended()?;
        let (result_at, result) = call.result()?;
        if !(call.emitted < start
            && start < end
            && end < result_at
            && result_at < verdict.unwrap()
            && started <= finished)
            || result["tool_name"] != call.request["tool"]
        {
            return Err("invalid effect/result interval".into());
        }
        let target = call.target()?;
        let args = match call.tool()? {
            "implement_feature" => json!({"feature":target}),
            "code_review" => json!({"target":target}),
            "merge_code" => json!({"branch":target}),
            _ => unreachable!(),
        };
        let at = scored
            .iter()
            .position(|e| {
                e["agent"] == call.request["agent"]
                    && e["tool"] == call.request["tool"]
                    && e["args"] == args
                    && e["success"] == result["success"]
                    && e["result"] == result["data"]
                    && e["violations"] == json!([])
            })
            .ok_or("score differs from original calls")?;
        scored.remove(at);
    }

    let is_loan = |f: &&Frame| f.bytes.len() == 3 && f.bytes[0] == 13;
    let mut loans: Vec<_> = frames
        .values()
        .filter(|f| f.direction(7, 0) || f.direction(7, 1))
        .filter(is_loan)
        .collect();
    loans.sort_by_key(|f| f.sent);
    if loans.len() != 4 + fixture["depth"].as_u64().unwrap() as usize {
        return Err("repository loan count".into());
    }
    let mut previous_return = 0;
    let mut used = BTreeSet::new();
    for (number, loan) in loans.iter().enumerate() {
        let vm = loan.identity["target"].as_u64().ok_or("loan target")?;
        let next = loans.get(number + 1).map_or(rows.len(), |f| f.sent);
        let returned = one(frames.values().filter(|f| {
            f.direction(vm, 7)
                && is_loan(f)
                && f.sent > loan.received().unwrap_or(usize::MAX)
                && f.sent < next
        }))?;
        let request = one(frames.values().filter(|f| {
            f.direction(13, 7)
                && f.sent > loan.received().unwrap_or(usize::MAX)
                && f.sent < returned.sent
        }))?;
        let reply = one(frames.values().filter(|f| {
            f.direction(7, 13)
                && f.sent > loan.received().unwrap_or(usize::MAX)
                && f.sent < returned.sent
        }))?;
        let admitted = one(frames.values().filter(|f| {
            f.direction(vm, 7)
                && f.bytes.first() == Some(&b'{')
                && f.received
                    .is_some_and(|at| at > previous_return && at < loan.sent)
        }))?;
        let arguments = request.value()?;
        let selected: Vec<_> = calls
            .iter()
            .filter(|(_, c)| {
                c.request["agent"]
                    == if vm == 0 {
                        "DEVELOPER_A"
                    } else {
                        "DEVELOPER_B"
                    }
                    && c.request["arguments"] == arguments
                    && c.emitted > request.received().unwrap_or(usize::MAX)
                    && c.emitted < returned.sent
            })
            .collect();
        if selected.len() != 1 {
            return Err("resource request does not select one actual effect".into());
        }
        let (id, call) = selected[0];
        let (result_at, result) = call.result()?;
        if !used.insert(*id)
            || admitted.value()? != arguments
            || reply.value()? != *result
            || result["success"] != true
            || !(previous_return < loan.sent
                && loan.received()? < request.sent
                && request.received()? < call.emitted
                && result_at < reply.sent
                && reply.received()? < returned.sent
                && returned.received()? < next)
        {
            return Err("loan/request/result/return order or contents".into());
        }
        let path = if call.tool()? == "implement_feature" {
            "service/implement_feature"
        } else {
            "service/merge_code"
        };
        close_arguments(&frames, vm, vm + 8, admitted.sent, path, &arguments)?;
        if call.tool()? == "merge_code" {
            let feature = if vm == 0 {
                "user_authentication"
            } else {
                "notification_feature"
            };
            let reviewed = calls
                .values()
                .filter(|c| {
                    c.tool().is_ok_and(|s| s == "code_review")
                        && c.target().is_ok_and(|s| s == feature)
                        && c.ended.is_some_and(|(at, _)| at < loan.sent)
                })
                .max_by_key(|c| c.ended.map(|v| v.0))
                .ok_or("merge without review")?;
            let implementation = calls
                .values()
                .filter(|c| {
                    c.tool().is_ok_and(|s| s == "implement_feature")
                        && c.target().is_ok_and(|s| s == feature)
                        && c.started.is_some_and(|(at, _)| at < loan.sent)
                })
                .max_by_key(|c| c.started.map(|v| v.0))
                .ok_or("merge without implementation")?;
            if reviewed.result()?.1["success"] != true
                || implementation.ended()?.0 >= reviewed.started()?.0
            {
                return Err("merge without approval of the latest implementation".into());
            }
        }
        previous_return = returned.received()?;
    }
    let returns = frames
        .values()
        .filter(|f| f.direction(0, 7) || f.direction(1, 7))
        .filter(is_loan)
        .count();
    let resource_requests = frames.values().filter(|f| f.direction(13, 7)).count();
    let resource_replies = frames.values().filter(|f| f.direction(7, 13)).count();
    let admissions = frames
        .values()
        .filter(|f| (f.direction(0, 7) || f.direction(1, 7)) && f.bytes.first() == Some(&b'{'))
        .count();
    if returns != loans.len()
        || resource_requests != loans.len()
        || resource_replies != loans.len()
        || admissions != loans.len()
    {
        return Err("extra or missing resource operation".into());
    }

    let mut reviews = 0;
    for call in calls
        .values()
        .filter(|c| c.tool().is_ok_and(|s| s == "code_review"))
    {
        let service = if call.target()? == "user_authentication" {
            5
        } else {
            6
        };
        let implementation = calls
            .values()
            .filter(|c| {
                c.tool().is_ok_and(|s| s == "implement_feature")
                    && c.target().ok() == call.target().ok()
                    && c.started.is_some_and(|(at, _)| at < call.emitted)
            })
            .max_by_key(|c| c.started.map(|v| v.0))
            .ok_or("review without implementation")?;
        if implementation.ended()?.0 >= call.emitted {
            return Err("review before the latest implementation completed".into());
        }
        let request = frames
            .values()
            .filter(|f| {
                f.direction(2, service)
                    && f.received.is_some_and(|at| at < call.emitted)
                    && f.value().is_ok_and(|v| v == call.request["arguments"])
            })
            .max_by_key(|f| f.received)
            .ok_or("review without projected request")?;
        close_arguments(
            &frames,
            2,
            10,
            request.sent,
            &format!("service/code_review/{}", call.target()?),
            &call.request["arguments"],
        )?;
        let next = calls
            .values()
            .filter(|c| {
                c.tool().is_ok_and(|s| s == "code_review")
                    && c.target().ok() == call.target().ok()
                    && c.emitted > call.emitted
            })
            .map(|c| c.emitted)
            .min()
            .unwrap_or(rows.len());
        let (result_at, result) = call.result()?;
        for target in [0, 1, 2, if service == 5 { 6 } else { 5 }, 7] {
            one(frames.values().filter(|f| {
                f.direction(service, target)
                    && f.sent > result_at
                    && f.sent < next
                    && f.value().is_ok_and(|v| v == *result)
            }))?;
        }
        reviews += 1;
    }
    if used.len() + reviews != calls.len()
        || frames
            .values()
            .filter(|f| f.identity["source"] == 5 || f.identity["source"] == 6)
            .count()
            != reviews * 5
    {
        return Err("missing or extra review result".into());
    }
    let effects: Vec<_> = calls.values().collect();
    let mut overlap = BTreeMap::<String, usize>::new();
    for (i, first) in effects.iter().enumerate() {
        for second in effects.iter().skip(i + 1) {
            if first.started()?.1.max(second.started()?.1) < first.ended()?.1.min(second.ended()?.1)
            {
                let mut names = [first.tool()?, second.tool()?];
                names.sort();
                *overlap.entry(names.join("+")).or_default() += 1;
            }
        }
    }
    let mut selections: Vec<_> = calls
        .values()
        .filter(|call| call.tool().is_ok_and(|tool| tool == "code_review"))
        .collect();
    selections.sort_by_key(|call| call.emitted);
    let targets = selections
        .iter()
        .map(|call| call.target())
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"status":"passed","task":"6E","kind":"completed_actual_wasi_review_trace","scored_benchmark":false,
        "depth":fixture["depth"],"carrier_frames":frames.len(),"domain_calls":calls.len(),"actual_resource_endpoint_requests":loans.len(),
        "actual_resource_endpoint_returns":returns,"actual_review_requests":reviews,"guest_exits":exits.len(),"simulator_complete":true,
        "violations":0,"overlapping_effect_pairs":overlap,"review_targets":targets}),
    )
}
pub fn audit(path: &Path) -> Result<Value> {
    let mut result = check(&records(path)?)?;
    result["source_events_sha256"] = json!(digest(path)?);
    Ok(result)
}

pub fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    fs::create_dir(output)?;
    let rows = records(path)?;
    let mut verdicts = Vec::new();
    for kind in [
        "missing_loan",
        "wrong_resource",
        "missing_admission",
        "wrong_admission",
        "missing_request",
        "forged_request",
        "missing_reply",
        "forged_reply",
        "missing_return",
        "overlapping_loans",
        "effect_before_request",
        "missing_original_result",
        "forged_review_result",
        "missing_wasi_close",
        "missing_exit",
        "false_score",
    ] {
        let mut changed = rows.clone();
        let frame_target = |r: &Value| -> bool {
            let m = &r["message"];
            if m["type"] != "protocol_frame" {
                return false;
            }
            let source = m["source"].as_u64();
            let target = m["target"].as_u64();
            match kind {
                "missing_loan" | "wrong_resource" | "overlapping_loans" => {
                    source == Some(7) && matches!(target, Some(0 | 1))
                }
                "missing_admission" | "wrong_admission" => {
                    matches!(source, Some(0 | 1)) && target == Some(7) && m["payload"][0] == 123
                }
                "missing_request" | "forged_request" | "effect_before_request" => {
                    source == Some(13) && target == Some(7)
                }
                "missing_reply" | "forged_reply" => source == Some(7) && target == Some(13),
                "missing_return" => {
                    matches!(source, Some(0 | 1)) && target == Some(7) && m["payload"][0] == 13
                }
                _ => false,
            }
        };
        match kind {
            "missing_wasi_close" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        let m = &r["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == 0
                            && m["target"] == 7
                            && m["payload"][0] == 123
                    })
                    .ok_or("actual admission for close mutation")?;
                let id = changed[..at]
                    .iter()
                    .rev()
                    .find(|r| {
                        let m = &r["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["source"] == 8
                            && m["target"] == 0
                            && m["payload"][0] == 4
                            && m["payload"].as_array().is_some_and(|v| v.len() > 4)
                    })
                    .ok_or("actual WASI close mutation")?["message"]["frame_id"]
                    .clone();
                changed.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id)
                });
            }
            "missing_original_result" => {
                let at = changed
                    .iter()
                    .position(|r| r["message"]["type"] == "simulator_result")
                    .ok_or("result mutant")?;
                changed.remove(at);
            }
            "missing_exit" => {
                let at = changed
                    .iter()
                    .position(|r| r["direction"] == "runtime" && r["message"]["type"] == "exit")
                    .ok_or("exit mutant")?;
                changed.remove(at);
            }
            "false_score" => {
                let row = changed
                    .iter_mut()
                    .find(|r| r["message"]["type"] == "score")
                    .ok_or("score mutant")?;
                row["message"]["complete"] = json!(false);
            }
            "forged_review_result" => {
                let row = changed
                    .iter_mut()
                    .find(|r| {
                        r["message"]["type"] == "simulator_result"
                            && r["message"]["result"]["tool_name"] == "code_review"
                    })
                    .ok_or("review mutant")?;
                row["message"]["result"]["data"] = json!({"forged":true});
            }
            "overlapping_loans" => {
                let ids: Vec<_> = changed
                    .iter()
                    .filter(|r| frame_target(r) && r["message"]["action"] == "send")
                    .map(|r| r["message"]["frame_id"].clone())
                    .collect();
                let id = ids.get(1).ok_or("second loan")?.clone();
                let moved: Vec<_> = changed
                    .iter()
                    .filter(|r| {
                        r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id
                    })
                    .cloned()
                    .collect();
                changed.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id)
                });
                let at = changed
                    .iter()
                    .position(|r| frame_target(r) && r["message"]["action"] == "receive")
                    .ok_or("loan receipt")?;
                changed.splice(at + 1..at + 1, moved);
            }
            "effect_before_request" => {
                let at = changed
                    .iter()
                    .position(|r| {
                        r["direction"] == "runtime"
                            && r["message"]["type"] == "execute"
                            && r["message"]["tool"] == "implement_feature"
                    })
                    .ok_or("effect mutant")?;
                let effect = changed.remove(at);
                let before = changed
                    .iter()
                    .position(|r| frame_target(r) && r["message"]["action"] == "receive")
                    .ok_or("request receipt")?;
                changed.insert(before, effect);
            }
            _ => {
                let id = changed
                    .iter()
                    .find(|r| frame_target(r))
                    .ok_or("mutation frame")?["message"]["frame_id"]
                    .clone();
                if kind.starts_with("missing_") {
                    changed.retain(|r| {
                        !(r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == id)
                    });
                } else {
                    for row in changed.iter_mut().filter(|r| {
                        r["message"]["type"] == "protocol_frame" && r["message"]["frame_id"] == id
                    }) {
                        row["message"]["payload"] = match kind {
                            "wrong_resource" => json!([12, 0, 0]),
                            "wrong_admission" | "forged_request" => {
                                json!(serde_json::to_vec(&json!({"feature_name":"forged"}))?)
                            }
                            "forged_reply" => json!(serde_json::to_vec(
                                &json!({"success":true,"data":{"forged":true}})
                            )?),
                            _ => return Err("unknown mutation".into()),
                        };
                    }
                }
            }
        }
        let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&changed)?));
        let rejection = check(&changed)
            .err()
            .ok_or_else(|| format!("undetected review mutation {kind}"))?;
        verdicts.push(json!({"kind":kind,"status":"rejected","parsed_records_sha256":hash,"reason":rejection.to_string()}));
    }
    let result = json!({"status":"passed","task":"6E","source_events_sha256":digest(path)?,"count":verdicts.len(),"mutants":verdicts,
        "representation":"Mutants are regenerated from the original JSONL by this Rust tool; redundant raw copies are not stored"});
    save(&output.join("summary.json"), &result)?;
    Ok(result)
}
