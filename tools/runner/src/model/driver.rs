//! JSON-RPC correlation and concurrent tool replies, without task authority.
use super::{Conversation, EFFORT, MODEL};
use crate::{
    evidence::Result,
    process::Guest,
    wire::{self, Wire},
};
use hibana_runtime::{host::notification, race};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::mpsc};

enum Incoming {
    Frame(Value),
    Reply(Value, Result<Value>),
}

pub(super) async fn run(
    wire: &mut Wire,
    people: &mut [Conversation],
    completed: &mut Vec<Value>,
    tool_limit: u64,
    guest: &Guest,
) -> Result<()> {
    wire.request(1,"initialize",json!({"clientInfo":{"name":"choreofix","title":"ChoreoFix","version":"0.1.0"},"capabilities":{"experimentalApi":true}})).await?;
    wire.send(&json!({"method":"initialized"})).await?;
    let mut threads = BTreeMap::new();
    for (index, person) in people.iter_mut().enumerate() {
        let id = 2u64
            .checked_add(u64::try_from(index)?)
            .ok_or("RPC id overflow")?;
        let request = json!({"id":id,"method":"thread/start","params":{
            "model":MODEL,"allowProviderModelFallback":false,
            "cwd":person.participant.output.canonicalize()?,"ephemeral":true,
            "environments":[],"sandbox":"read-only","approvalPolicy":"never",
            "baseInstructions":format!("{}\nUse only the supplied experiment tools. Do not use native tools or start other agents. Finish with a brief final response.",person.participant.instructions),
            "config":{"model_reasoning_effort":EFFORT,"features.shell_tool":false,"features.multi_agent":false,"project_doc_max_bytes":0},
            "dynamicTools":[{"type":"function","name":"shell","description":"Run your script in the WASI wash guest over ChoreoFS.","inputSchema":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"],"additionalProperties":false}}]
        }});
        person.trace.record("send", &request)?;
        let reply = wire
            .request(id, "thread/start", request["params"].clone())
            .await?;
        person
            .trace
            .record("receive", &json!({"id":id,"result":reply}))?;
        person.observed.returned_model = reply["model"].clone();
        person.observed.returned_effort = reply["reasoningEffort"].clone();
        if person.observed.returned_model != MODEL || person.observed.returned_effort != EFFORT {
            return Err("participant model or reasoning effort was substituted".into());
        }
        let thread = reply["thread"]["id"]
            .as_str()
            .ok_or("thread id missing")?
            .to_owned();
        person.observed.thread_id = json!(thread);
        if threads.insert(thread, index).is_some() {
            return Err("duplicate thread id".into());
        }
    }
    let mut starts = BTreeMap::new();
    let first_turn = 2u64
        .checked_add(u64::try_from(people.len())?)
        .ok_or("RPC id overflow")?;
    for (index, person) in people.iter().enumerate() {
        let id = first_turn
            .checked_add(u64::try_from(index)?)
            .ok_or("RPC id overflow")?;
        starts.insert(id, index);
        let request = json!({"id":id,"method":"turn/start","params":{
            "threadId":person.observed.thread_id,"model":MODEL,"effort":EFFORT,
            "input":[{"type":"text","text":"Begin your assigned work. Use the experiment tools to carry it out."}]
        }});
        person.trace.record("send", &request)?;
        wire.send(&request).await?;
    }
    let (replies, receive) = mpsc::channel::<(Value, Result<Value>)>();
    let (notify, mut ready) = notification()?;
    let mut pending = BTreeMap::<String, usize>::new();
    while !threads.is_empty() {
        let incoming = race(async { wire.next().await.map(Incoming::Frame) }, async {
            let (id, result) = wire::receive(&receive, &mut ready)
                .await?
                .ok_or("tool response direction closed")?;
            Ok::<Incoming, Box<dyn std::error::Error + Send + Sync>>(Incoming::Reply(id, result))
        })
        .await?;
        let message = match incoming {
            Incoming::Frame(message) => message,
            Incoming::Reply(id, outcome) => {
                let index = pending
                    .remove(&id.to_string())
                    .ok_or("tool reply has no matching request")?;
                let outcome = outcome?;
                let reply = json!({"id":id,"result":{"success":true,"contentItems":[{"type":"inputText","text":outcome.to_string()}]}});
                people[index].trace.record("send", &reply)?;
                wire.send(&reply).await?;
                continue;
            }
        };
        if message.get("method").is_none() {
            let id = message["id"].as_u64().ok_or("RPC response id")?;
            let index = starts.remove(&id).ok_or("unexpected RPC response")?;
            people[index].trace.record("receive", &message)?;
            if message.get("error").is_some() {
                return Err(format!("model RPC error: {}", message["error"]).into());
            }
            if !message["result"]["turn"]["id"].is_string() {
                return Err("turn start result missing".into());
            }
            continue;
        }
        let index = message["params"]["threadId"]
            .as_str()
            .and_then(|id| threads.get(id))
            .copied();
        let Some(index) = index else {
            if message.get("id").is_some() || message["method"] == "turn/completed" {
                return Err("model request or completion has no active conversation".into());
            }
            continue;
        };
        let person = &mut people[index];
        person.trace.record("receive", &message)?;
        if let Some(kind) = message["params"]["item"]["type"].as_str()
            && ![
                "dynamicToolCall",
                "reasoning",
                "userMessage",
                "agentMessage",
            ]
            .contains(&kind)
        {
            return Err(format!("unexpected native model item: {kind}").into());
        }
        match message["method"].as_str() {
            Some("item/tool/call") => {
                person.observed.tool_calls += 1;
                if person.observed.tool_calls > tool_limit {
                    return Err("participant tool-call budget exhausted".into());
                }
                let id = message
                    .get("id")
                    .cloned()
                    .ok_or("tool request id missing")?;
                if pending.insert(id.to_string(), index).is_some() {
                    return Err("duplicate pending tool id".into());
                }
                let arguments = &message["params"]["arguments"];
                if message["params"]["tool"] != "shell"
                    || arguments.as_object().is_none_or(|o| o.len() != 1)
                    || !arguments["command"].is_string()
                {
                    pending.remove(&id.to_string());
                    let reply = json!({"id":id,"result":{"success":false,"contentItems":[{"type":"inputText","text":"expected shell with one command string"}]}});
                    person.trace.record("send", &reply)?;
                    wire.send(&reply).await?;
                } else {
                    let command = arguments["command"].as_str().unwrap().to_owned();
                    let reply = (replies.clone(), notify.try_clone()?);
                    if command.len() > 4096 {
                        wire::respond(
                            &reply,
                            id,
                            Ok(json!({"status":"script_too_long","limit_utf8_bytes":4096})),
                        )?;
                    } else {
                        guest.send(
                            json!({"id":id,"agent":person.participant.agent,"command":command}),
                            reply,
                        )?;
                    }
                }
            }
            Some("thread/tokenUsage/updated") => {
                person.observed.usage = message["params"]["tokenUsage"]["total"].clone()
            }
            Some("model/rerouted") if message["params"]["toModel"] != MODEL => {
                return Err("participant model was rerouted".into());
            }
            Some("turn/completed") => {
                if pending.values().any(|&owner| owner == index) {
                    return Err("turn completed with a pending tool request".into());
                }
                threads.remove(
                    message["params"]["threadId"]
                        .as_str()
                        .ok_or("completed thread id")?,
                );
                completed.push(person.finish(&message["params"]["turn"])?);
            }
            _ if message.get("id").is_some() => {
                let reply = json!({"id":message["id"],"error":{"code":-32601,"message":"Only the supplied experiment tools are available"}});
                person.trace.record("send", &reply)?;
                wire.send(&reply).await?;
            }
            _ => {}
        }
    }
    if !pending.is_empty() || !starts.is_empty() {
        return Err("unfinished model RPC requests".into());
    }
    Ok(())
}
