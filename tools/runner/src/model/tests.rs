use super::*;
use crate::wire;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Notify,
};

// These byte-stream fixtures exercise the same multiplexed client as live runs.
// They do not replace participants in a benchmark trial.
async fn exchange(mode: &str) -> Result<(Result<()>, Vec<Conversation>, Vec<Value>)> {
    let count = if mode == "parallel" { 2 } else { 1 };
    let root = std::env::temp_dir().join(format!(
        "choreofix-model-wire-{}-{}-{mode}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    let mut people = Vec::new();
    for index in 0..count {
        let output = root.join(format!("agent-{index}"));
        fs::create_dir(&output)?;
        people.push(Conversation {
            trace: Trace::create(&output.join("events.jsonl"))?,
            participant: Participant {
                agent: format!("agent-{index}"),
                instructions: "test".into(),
                output,
            },
            observed: Observed::default(),
            started: Instant::now(),
        });
    }
    let (client, server) = tokio::io::duplex(65536);
    let (output, input) = tokio::io::split(client);
    let (server_read, mut server_write) = tokio::io::split(server);
    let mut incoming = BufReader::new(server_read).lines();
    let mut client = Wire::new(
        input,
        output,
        Trace::create(&root.join("connection.jsonl"))?,
    );
    let server = async {
        let init: Value =
            serde_json::from_str(&incoming.next_line().await?.ok_or("initialize missing")?)?;
        assert_eq!(init["params"]["capabilities"]["experimentalApi"], true);
        wire::send(&mut server_write, &json!({"id":1,"result":{}})).await?;
        incoming.next_line().await?.ok_or("initialized missing")?;
        for index in 0..count {
            let thread: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("thread missing")?)?;
            assert_eq!(thread["params"]["model"], MODEL);
            assert_eq!(thread["params"]["allowProviderModelFallback"], false);
            assert_eq!(thread["params"]["config"]["model_reasoning_effort"], EFFORT);
            wire::send(&mut server_write,&json!({"id":thread["id"],"result":{"thread":{"id":format!("t{index}")},"model":if mode=="wrong_model" {"other"} else {MODEL},"reasoningEffort":if mode=="wrong_effort" {"low"} else {EFFORT}}})).await?;
            if mode == "wrong_model" || mode == "wrong_effort" {
                return Ok::<_, Box<dyn std::error::Error + Send + Sync>>(());
            }
        }
        // All turns must start before the first tool response, including when a
        // different participant's command cannot yet finish.
        for index in 0..count {
            let turn: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("turn missing")?)?;
            assert_eq!(turn["params"]["threadId"], format!("t{index}"));
            assert_eq!(turn["params"]["effort"], EFFORT);
            wire::send(&mut server_write,&json!({"id":turn["id"],"result":{"turn":{"id":format!("turn-{index}"),"status":"inProgress"}}})).await?;
        }
        if mode == "native" {
            wire::send(&mut server_write,&json!({"method":"item/started","params":{"threadId":"t0","item":{"type":"commandExecution"}}})).await?;
            return Ok(());
        }
        for index in 0..count {
            wire::send(&mut server_write,&json!({"id":format!("call-{index}"),"method":"item/tool/call","params":{"threadId":format!("t{index}"),"tool":"shell","arguments":{"command":format!("work-{index}")}}})).await?;
        }
        for slot in 0..count {
            let reply: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("tool reply missing")?)?;
            let index = if mode == "parallel" {
                count - 1 - slot
            } else {
                0
            };
            assert_eq!(reply["id"], format!("call-{index}"));
            assert_eq!(
                reply["result"]["contentItems"][0]["text"],
                json!({"stdout":format!("agent-{index}/work-{index}")}).to_string()
            );
            wire::send(&mut server_write,&json!({"method":"thread/tokenUsage/updated","params":{"threadId":format!("t{index}"),"tokenUsage":{"total":{"inputTokens":100,"outputTokens":20+index,"totalTokens":120+index}}}})).await?;
            wire::send(&mut server_write,&json!({"method":"turn/completed","params":{"threadId":format!("t{index}"),"turn":{"status":"completed","error":null}}})).await?;
        }
        Ok(())
    };
    let released = Arc::new(Notify::new());
    let parallel = mode == "parallel";
    let mut completed = Vec::new();
    let run = driver::run(
        &mut client,
        &mut people,
        &mut completed,
        200,
        move |agent, command| {
            let released = released.clone();
            async move {
                if parallel {
                    if agent == "agent-0" {
                        released.notified().await;
                    } else {
                        released.notify_one();
                    }
                }
                Ok(json!({"stdout":format!("{agent}/{command}")}))
            }
        },
    );
    let (result, server) =
        tokio::time::timeout(Duration::from_secs(5), async { tokio::join!(run, server) }).await?;
    server?;
    fs::remove_dir_all(root)?;
    Ok((result, people, completed))
}

#[tokio::test(flavor = "current_thread")]
async fn tool_response_and_inclusive_usage_cross_actual_wire() -> Result<()> {
    let (result, people, completed) = exchange("normal").await?;
    result?;
    assert_eq!(completed[0]["status"], "completed");
    assert_eq!(people[0].observed.tool_calls, 1);
    assert_eq!(people[0].observed.usage["totalTokens"], 120);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn independent_conversations_dispatch_tools_concurrently_without_crossing_results()
-> Result<()> {
    let (result, people, completed) = exchange("parallel").await?;
    result?;
    assert_eq!(completed[0]["participant"], "agent-1");
    assert_eq!(completed[1]["participant"], "agent-0");
    for (index, person) in people.iter().enumerate() {
        assert_eq!(person.observed.usage["totalTokens"], 120 + index);
        assert_eq!(person.observed.tool_calls, 1);
    }
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
async fn substituted_effort_is_rejected() -> Result<()> {
    assert!(
        exchange("wrong_effort")
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
