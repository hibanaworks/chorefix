//! Role-bearing items for App Server's `thread/inject_items` input.
use crate::evidence::Result;
use serde_json::{Value, json};

/// Convert the client's current visible input, including its context trimming.
/// This does not establish support for the request's generation settings.
pub(super) fn items(request: &Value) -> Result<Vec<Value>> {
    request["input"]
        .as_array()
        .ok_or("comparison input must be an array")?
        .iter()
        .map(|item| {
            let fields = item.as_object().ok_or("comparison history item")?;
            match item.get("type").and_then(Value::as_str) {
                None => {
                    if fields
                        .keys()
                        .any(|key| !["role", "content"].contains(&key.as_str()))
                    {
                        return Err("unsupported comparison message field".into());
                    }
                    let role = item["role"].as_str().ok_or("comparison message role")?;
                    let kind = match role {
                        "system" | "developer" | "user" => "input_text",
                        "assistant" => "output_text",
                        _ => return Err("unsupported comparison message role".into()),
                    };
                    let text = item["content"].as_str().ok_or("comparison message text")?;
                    Ok(json!({"type":"message","role":role,
                        "content":[{"type":kind,"text":text}]}))
                }
                Some("function_call") => {
                    if fields.len() != 4
                        || ["call_id", "name", "arguments"]
                            .iter()
                            .any(|key| !item[key].is_string())
                    {
                        return Err("unsupported comparison function call".into());
                    }
                    Ok(item.clone())
                }
                Some("function_call_output") => {
                    if fields.len() != 3
                        || !item["call_id"].is_string()
                        || !item["output"].is_string()
                    {
                        return Err("unsupported comparison function result".into());
                    }
                    Ok(item.clone())
                }
                _ => Err("unsupported comparison history item".into()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evidence::{Trace, digest, save},
        model::{CLI, EFFORT, MODEL},
        wire::Wire,
    };
    use std::{
        fs,
        path::Path,
        process::Stdio,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio::process::Command;

    #[test]
    fn retains_roles_call_bytes_and_reverse_result_arrival() -> Result<()> {
        let source = json!({"input":[
            {"role":"system","content":" original system\n"},
            {"role":"developer","content":"developer instructions"},
            {"role":"user","content":""},
            {"role":"assistant","content":"calling two tools"},
            {"type":"function_call","call_id":"a","name":"first","arguments":"{ \"x\" : 1 }"},
            {"type":"function_call","call_id":"b","name":"second","arguments":"{}"},
            {"type":"function_call_output","call_id":"b","output":"second result first"},
            {"type":"function_call_output","call_id":"a","output":""}
        ]});
        let actual = items(&source)?;
        assert_eq!(
            actual[..4],
            [
                json!({"type":"message","role":"system","content":[{"type":"input_text","text":" original system\n"}]}),
                json!({"type":"message","role":"developer","content":[{"type":"input_text","text":"developer instructions"}]}),
                json!({"type":"message","role":"user","content":[{"type":"input_text","text":""}]}),
                json!({"type":"message","role":"assistant","content":[{"type":"output_text","text":"calling two tools"}]})
            ]
        );
        assert_eq!(&actual[4..], &source["input"].as_array().unwrap()[4..]);
        Ok(())
    }

    #[test]
    fn rejects_unrepresented_metadata_and_nontext_history() {
        for item in [
            json!({"role":"user","content":"text","name":"named speaker"}),
            json!({"role":"user","content":[{"type":"image_url","image_url":"image"}]}),
            json!({"role":"tool","content":"unmatched result"}),
            json!({"type":"function_call","call_id":"a","name":"first","arguments":{}}),
            json!({"type":"function_call_output","call_id":"a","output":[],"status":"completed"}),
            json!({"type":"reasoning","encrypted_content":"hidden previous reasoning"}),
        ] {
            assert!(items(&json!({"input":[item.clone()]})).is_err(), "{item}");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "checks the installed App Server history RPC; does not start model generation"]
    async fn installed_app_server_accepts_role_bearing_history_without_generation() -> Result<()> {
        let parent = std::env::var_os("CHOREOFIX_EVIDENCE_ROOT")
            .ok_or("CHOREOFIX_EVIDENCE_ROOT is required")?;
        let directory = Path::new(&parent).join(format!(
            "comparison-history-{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&directory)?;
        let mut child = Command::new(CLI)
            .args([
                "-c",
                "forced_login_method=\"chatgpt\"",
                "-c",
                "model_provider=\"openai\"",
                "app-server",
                "--listen",
                "stdio://",
            ])
            .env_remove("OPENAI_API_KEY")
            .env_remove("CODEX_API_KEY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(fs::File::create_new(directory.join("stderr.log"))?)
            .kill_on_drop(true)
            .spawn()?;
        let mut wire = Wire::new(
            child.stdin.take().ok_or("App Server stdin")?,
            child.stdout.take().ok_or("App Server stdout")?,
            Trace::create(&directory.join("events.jsonl"))?,
        );
        let outcome = tokio::time::timeout(Duration::from_secs(30), async {
            wire.request(1, "initialize", json!({"clientInfo":{"name":"choreofix_comparison_history","version":"0.1.0"},"capabilities":{"experimentalApi":true}})).await?;
            wire.send(&json!({"method":"initialized"})).await?;
            let account = wire.request(2, "account/read", json!({"refreshToken":false})).await?;
            if account["account"]["type"] != "chatgpt" {
                return Err("comparison history check requires ChatGPT login".into());
            }
            let thread = wire.request(3, "thread/start", json!({
                "model":MODEL,"allowProviderModelFallback":false,
                "cwd":directory.canonicalize()?,"ephemeral":true,
                "environments":[],"sandbox":"read-only","approvalPolicy":"never",
                "baseInstructions":"","developerInstructions":"",
                "config":{"model_reasoning_effort":EFFORT,"features.shell_tool":false,
                    "features.multi_agent":false,"project_doc_max_bytes":0}
            })).await?;
            if thread["model"] != MODEL || thread["reasoningEffort"] != EFFORT {
                return Err("App Server substituted comparison model or effort".into());
            }
            let mut chat = super::super::tests::chat();
            let messages = chat["messages"].as_array_mut().unwrap();
            messages.push(json!({"role":"assistant","content":"Original visible explanation.","tool_calls":[
                {"type":"function","id":"first","function":{"name":"receive_message","arguments":"{ \"channel_id\": \"a\" }"}},
                {"type":"function","id":"second","function":{"name":"receive_message","arguments":"{\"channel_id\":\"b\"}"}}
            ]}));
            messages.push(json!({"role":"tool","tool_call_id":"second","content":"second result arrives first"}));
            messages.push(json!({"role":"tool","tool_call_id":"first","content":""}));
            let prepared = super::super::request(&chat)?;
            let items = items(&prepared)?;
            save(&directory.join("client-request.json"), &chat)?;
            save(&directory.join("history.json"), &json!({"items":items}))?;
            wire.request(4, "thread/inject_items", json!({
                "threadId":thread["thread"]["id"],"items":items
            })).await?;
            save(&directory.join("receipt.json"), &json!({
                "scope":"App Server history RPC acceptance; no inference or tool execution",
                "cli_sha256":digest(Path::new(CLI))?,"requested_model":MODEL,
                "harness_sha256":digest(&std::env::current_exe()?)?,
                "history_sha256":digest(&directory.join("history.json"))?,
                "rpc_events_sha256":digest(&directory.join("events.jsonl"))?,
                "returned_model":thread["model"],"requested_effort":EFFORT,
                "returned_effort":thread["reasoningEffort"],"model_calls":0,
                "history_items":items.len(),"authentication":"chatgpt",
                "generation_settings_qualified":false,
                "comparison_connection_qualified":false
            }))?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        }).await;
        let _ = child.kill().await;
        let _ = child.wait().await;
        outcome??;
        Ok(())
    }
}
