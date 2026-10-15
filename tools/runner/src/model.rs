//! Independent Luna participants. Tool transport does not authorize task effects.
use crate::{
    evidence::{Result, Trace, digest, save},
    wire::Wire,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::Command,
};

pub const MODEL: &str = "gpt-6-luna";
pub const EFFORT: &str = "xhigh";
pub const CLI: &str = "/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex";

#[derive(Default)]
struct Observed {
    returned_model: Value,
    returned_effort: Value,
    usage: Value,
    tool_calls: u64,
}

async fn participant<W, R, F, Fut>(
    wire: &mut Wire<W, R>,
    prompt: &str,
    cwd: &Path,
    tool_limit: u64,
    observed: &mut Observed,
    mut dispatch: F,
) -> Result<Value>
where
    W: AsyncWrite + Unpin,
    R: AsyncRead + Unpin,
    F: FnMut(String) -> Fut,
    Fut: Future<Output = Result<Value>>,
{
    wire.request(1, "initialize", json!({"clientInfo":{"name":"choreofix","title":"ChoreoFix","version":"0.1.0"}, "capabilities":{"experimentalApi":true}})).await?;
    wire.send(&json!({"method":"initialized"})).await?;
    let thread = wire.request(2, "thread/start", json!({
        "model":MODEL,"allowProviderModelFallback":false,
        "cwd":cwd,"ephemeral":true,"environments":[],"sandbox":"read-only","approvalPolicy":"never",
        "baseInstructions":format!("{prompt}\nUse only the supplied experiment tools. Do not use native tools or start other agents. Finish with a brief final response."),
        "config":{"model_reasoning_effort":EFFORT,"features.shell_tool":false,"features.multi_agent":false,"project_doc_max_bytes":0},
        "dynamicTools":[{"type":"function","name":"shell","description":"Run your script in the WASI wash guest over ChoreoFS.","inputSchema":{"type":"object","properties":{"command":{"type":"string"}},"required":["command"],"additionalProperties":false}}]
    })).await?;
    observed.returned_model = thread["model"].clone();
    observed.returned_effort = thread["reasoningEffort"].clone();
    if observed.returned_model != MODEL || observed.returned_effort != EFFORT {
        return Err("participant model or reasoning effort was substituted".into());
    }
    wire.send(&json!({"id":3,"method":"turn/start","params":{"threadId":thread["thread"]["id"],"model":MODEL,"effort":EFFORT,"input":[{"type":"text","text":"Begin your assigned work. Use the experiment tools to carry it out."}]}})).await?;
    loop {
        let message = wire.next().await?;
        if let Some(item_type) = message["params"]["item"]["type"].as_str()
            && ![
                "dynamicToolCall",
                "reasoning",
                "userMessage",
                "agentMessage",
            ]
            .contains(&item_type)
        {
            return Err(format!("unexpected native model item: {item_type}").into());
        }
        match message["method"].as_str() {
            Some("item/tool/call") => {
                observed.tool_calls += 1;
                if observed.tool_calls > tool_limit {
                    return Err("participant tool-call budget exhausted".into());
                }
                let arguments = &message["params"]["arguments"];
                let result = if message["params"]["tool"] != "shell"
                    || arguments.as_object().is_none_or(|o| o.len() != 1)
                    || !arguments["command"].is_string()
                {
                    json!({"success":false,"contentItems":[{"type":"inputText","text":json!({"status":"invalid_tool_arguments","error":"expected shell with one command string"}).to_string()}]})
                } else {
                    let command = arguments["command"].as_str().unwrap();
                    let outcome = if command.len() > 4096 {
                        json!({"status":"script_too_long","limit_utf8_bytes":4096})
                    } else {
                        dispatch(command.to_owned()).await?
                    };
                    json!({"success":true,"contentItems":[{"type":"inputText","text":outcome.to_string()}]})
                };
                wire.send(&json!({"id":message["id"],"result":result}))
                    .await?;
            }
            Some("thread/tokenUsage/updated") => {
                observed.usage = message["params"]["tokenUsage"]["total"].clone()
            }
            Some("turn/completed") => return Ok(message["params"]["turn"].clone()),
            _ if message.get("id").is_some() && message.get("method").is_some() => {
                wire.send(&json!({"id":message["id"],"error":{"code":-32601,"message":"Only the supplied experiment tools are available"}})).await?;
            }
            _ if message.get("error").is_some() => {
                return Err(format!("model RPC error: {}", message["error"]).into());
            }
            _ => {}
        }
    }
}

pub async fn run<F, Fut>(
    agent: &str,
    prompt: &str,
    output: &Path,
    timeout: Duration,
    tool_limit: u64,
    dispatch: F,
) -> Result<Value>
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = Result<Value>>,
{
    fs::create_dir(output)?;
    let started = Instant::now();
    let mut child = Command::new(CLI)
        .args(["app-server", "--listen", "stdio://"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(fs::File::create_new(output.join("stderr.log"))?)
        .kill_on_drop(true)
        .spawn()?;
    let mut wire = Wire::new(
        child.stdin.take().ok_or("model stdin")?,
        child.stdout.take().ok_or("model stdout")?,
        Trace::create(&output.join("events.jsonl"))?,
    );
    let mut observed = Observed::default();
    let outcome = tokio::time::timeout(
        timeout,
        participant(
            &mut wire,
            prompt,
            &output.canonicalize()?,
            tool_limit,
            &mut observed,
            dispatch,
        ),
    )
    .await;
    let mut result = match outcome {
        Ok(Ok(turn)) => json!({"participant":agent,"status":turn["status"],"error":turn["error"]}),
        Ok(Err(error)) => {
            json!({"participant":agent,"status":"harness_error","error":error.to_string()})
        }
        Err(_) => json!({"participant":agent,"status":"timeout","error":"participant timeout"}),
    };
    let _ = child.kill().await;
    let _ = child.wait().await;
    result["requested_model"] = json!(MODEL);
    result["returned_model"] = observed.returned_model;
    result["requested_effort"] = json!(EFFORT);
    result["returned_effort"] = observed.returned_effort;
    result["usage"] = observed.usage;
    result["tool_calls"] = json!(observed.tool_calls);
    result["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    result["raw_events_sha256"] = json!(digest(&output.join("events.jsonl"))?);
    save(&output.join("participant.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire;
    use tokio::io::{AsyncBufReadExt, BufReader};

    // Exercise the actual client protocol over an in-memory byte stream. This
    // never substitutes for a model in an experiment run.
    async fn exchange(mode: &str) -> Result<(Result<Value>, Observed)> {
        let (client, server) = tokio::io::duplex(32768);
        let (output, input) = tokio::io::split(client);
        let (server_read, mut server_write) = tokio::io::split(server);
        let mut incoming = BufReader::new(server_read).lines();
        let directory = std::env::temp_dir().join(format!(
            "choreofix-wire-{}-{}-{mode}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        fs::create_dir(&directory)?;
        let mut client = Wire::new(
            input,
            output,
            Trace::create(&directory.join("events.jsonl"))?,
        );
        let server = async {
            let init: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("initialize missing")?)?;
            assert_eq!(init["params"]["capabilities"]["experimentalApi"], true);
            wire::send(&mut server_write, &json!({"id":1,"result":{}})).await?;
            incoming.next_line().await?.ok_or("initialized missing")?;
            let thread: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("thread missing")?)?;
            assert_eq!(thread["params"]["model"], MODEL);
            assert_eq!(thread["params"]["allowProviderModelFallback"], false);
            assert_eq!(thread["params"]["config"]["model_reasoning_effort"], EFFORT);
            wire::send(&mut server_write,&json!({"id":2,"result":{"thread":{"id":"t"},"model":if mode=="wrong_model" {"other"} else {MODEL},"reasoningEffort":EFFORT}})).await?;
            if mode == "wrong_model" {
                return Ok::<_, Box<dyn std::error::Error + Send + Sync>>(());
            }
            let turn: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("turn missing")?)?;
            assert_eq!(turn["params"]["effort"], EFFORT);
            if mode == "native" {
                wire::send(
                    &mut server_write,
                    &json!({"method":"item/started","params":{"item":{"type":"commandExecution"}}}),
                )
                .await?;
                return Ok(());
            }
            wire::send(&mut server_write,&json!({"id":"call1","method":"item/tool/call","params":{"tool":"shell","arguments":{"command":"expr 3 + 4"}}})).await?;
            let reply: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("tool reply missing")?)?;
            assert_eq!(
                reply["result"]["contentItems"][0]["text"],
                json!({"stdout":"7\n"}).to_string()
            );
            wire::send(&mut server_write,&json!({"method":"thread/tokenUsage/updated","params":{"tokenUsage":{"total":{"inputTokens":100,"outputTokens":20,"totalTokens":120}}}})).await?;
            wire::send(&mut server_write,&json!({"method":"turn/completed","params":{"turn":{"status":"completed","error":null}}})).await?;
            Ok(())
        };
        let mut observed = Observed::default();
        let client = participant(
            &mut client,
            "test",
            &directory,
            200,
            &mut observed,
            |command| async move {
                assert_eq!(command, "expr 3 + 4");
                Ok(json!({"stdout":"7\n"}))
            },
        );
        let (result, server) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(client, server)
        })
        .await?;
        server?;
        fs::remove_dir_all(directory)?;
        Ok((result, observed))
    }
    #[tokio::test(flavor = "current_thread")]
    async fn tool_response_and_inclusive_usage_cross_actual_wire() -> Result<()> {
        let (result, observed) = exchange("normal").await?;
        assert_eq!(result?["status"], "completed");
        assert_eq!(observed.tool_calls, 1);
        assert_eq!(observed.usage["totalTokens"], 120);
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn substituted_model_is_rejected() -> Result<()> {
        assert!(
            exchange("wrong_model")
                .await?
                .0
                .unwrap_err()
                .to_string()
                .contains("substituted")
        );
        Ok(())
    }
    #[tokio::test(flavor = "current_thread")]
    async fn native_tool_is_rejected() -> Result<()> {
        assert!(
            exchange("native")
                .await?
                .0
                .unwrap_err()
                .to_string()
                .contains("native")
        );
        Ok(())
    }
}
