use super::*;
use crate::process::Guest;
use crate::wire;
use hibana_runtime::host::{Io, deadline, sleep};
use std::os::unix::net::UnixStream;

// These byte-stream fixtures exercise the same multiplexed client as live runs.
// They do not replace participants in a benchmark trial.
#[hibana_runtime::scope]
async fn exchange(mode: &str) -> Result<(Vec<Conversation>, Vec<Value>)> {
    let count = if mode == "parallel" { 2 } else { 1 };
    let calls = if mode == "parallel_same_thread" {
        2
    } else {
        count
    };
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
    let (client, server) = UnixStream::pair()?;
    let mut client = Wire::new(
        Io::new(client.try_clone()?.into())?,
        Io::new(client.into())?,
        Trace::create(&root.join("connection.jsonl"))?,
    );
    let mut incoming = wire::Lines::new(Io::new(server.try_clone()?.into())?);
    let mut server_write = Io::new(server.try_clone()?.into())?;
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
            if mode != "missing_start_reply" && mode != "early_tool" {
                wire::send(&mut server_write,&json!({"id":turn["id"],"result":{"turn":{"id":format!("turn-{index}"),"status":"inProgress"}}})).await?;
            }
        }
        if mode == "native" {
            wire::send(&mut server_write,&json!({"method":"item/started","params":{"threadId":"t0","item":{"type":"commandExecution"}}})).await?;
            return Ok(());
        }
        for index in 0..calls {
            let thread = if mode == "parallel_same_thread" {
                0
            } else {
                index
            };
            wire::send(&mut server_write,&json!({"id":format!("call-{index}"),"method":"item/tool/call","params":{"threadId":format!("t{thread}"),"tool":"shell","arguments":{"command":format!("work-{index}")}}})).await?;
        }
        for slot in 0..calls {
            let reply: Value =
                serde_json::from_str(&incoming.next_line().await?.ok_or("tool reply missing")?)?;
            let index = if mode.starts_with("parallel") {
                calls - 1 - slot
            } else {
                0
            };
            assert_eq!(reply["id"], format!("call-{index}"));
            assert_eq!(
                reply["result"]["contentItems"][0]["text"],
                shell_result(
                    mode,
                    &format!(
                        "agent-{}",
                        if mode == "parallel_same_thread" {
                            0
                        } else {
                            index
                        }
                    ),
                    &format!("work-{index}")
                )
                .to_string()
            );
            if mode == "late_disconnect" {
                server.shutdown(std::net::Shutdown::Write)?;
                return Ok(());
            }
            if mode == "late_timeout" {
                sleep(Duration::from_millis(100)).await?;
                return Ok(());
            }
            if mode == "early_tool" {
                wire::send(&mut server_write, &json!({"id":2 + count,"result":{"turn":{"id":"turn-0","status":"inProgress"}}})).await?;
            }
            if mode == "parallel_same_thread" && slot + 1 != calls {
                continue;
            }
            wire::send(&mut server_write,&json!({"method":"thread/tokenUsage/updated","params":{"threadId":format!("t{index}"),"tokenUsage":{"total":{"inputTokens":100,"outputTokens":20+index,"totalTokens":120+index}}}})).await?;
            wire::send(&mut server_write,&json!({"method":"turn/completed","params":{"threadId":format!("t{index}"),"turn":{"status":"completed","error":null}}})).await?;
        }
        Ok(())
    };
    let (guest, commands, mut ready) = Guest::test_input()?;
    let shell = async {
        if ["wrong_model", "wrong_effort", "native"].contains(&mode) {
            return Ok(());
        }
        let first = wire::receive(&commands, &mut ready)
            .await?
            .ok_or("first shell request missing")?;
        let mut requests = Vec::new();
        if mode.starts_with("parallel") {
            requests.push(
                wire::receive(&commands, &mut ready)
                    .await?
                    .ok_or("independent shell request was blocked")?,
            );
        }
        requests.push(first);
        for (request, reply) in requests {
            wire::respond(
                &reply,
                request["id"].clone(),
                Ok(shell_result(
                    mode,
                    request["agent"].as_str().ok_or("shell agent")?,
                    request["command"].as_str().ok_or("shell command")?,
                )),
            )?;
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };
    let mut completed = Vec::new();
    let timeout = if mode == "late_timeout" {
        Duration::from_millis(20)
    } else {
        Duration::from_secs(4)
    };
    let result = hibana_runtime::race(
        async {
            async {
                server.await?;
            };
            async {
                shell.await?;
            };
            let result: io::Result<Result<()>> = hibana_runtime::race(
                async {
                    Ok(driver::run(&mut client, &mut people, &mut completed, 200, &guest).await)
                },
                deadline(timeout),
            )
            .await;
            Ok(result)
        },
        deadline(Duration::from_secs(5)),
    )
    .await?;
    let completed = finish_reports(&people, completed, result);
    fs::remove_dir_all(root)?;
    Ok((people, completed?))
}

fn shell_result(mode: &str, agent: &str, command: &str) -> Value {
    let mut result = json!({"stdout":format!("{agent}/{command}")});
    if mode.starts_with("late") {
        result["exit_status"] = json!(0);
    }
    result
}

#[hibana_runtime::main]
#[test]
async fn tool_response_and_inclusive_usage_cross_actual_wire() -> Result<()> {
    let (people, completed) = exchange("normal").await?;
    assert_eq!(completed[0]["status"], "completed");
    assert_eq!(people[0].observed.tool_calls, 1);
    assert_eq!(people[0].observed.usage["totalTokens"], 120);
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn tools_can_arrive_before_the_turn_start_acknowledgement() -> Result<()> {
    let (people, completed) = exchange("early_tool").await?;
    assert_eq!(completed[0]["status"], "completed");
    assert_eq!(people[0].observed.tool_calls, 1);
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn parallel_tool_calls_from_one_conversation_keep_their_reply_ids() -> Result<()> {
    let (people, completed) = exchange("parallel_same_thread").await?;
    assert_eq!(completed[0]["status"], "completed");
    assert_eq!(people[0].observed.tool_calls, 2);
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn independent_conversations_dispatch_tools_concurrently_without_crossing_results()
-> Result<()> {
    let (people, completed) = exchange("parallel").await?;
    assert_eq!(completed[0]["participant"], "agent-1");
    assert_eq!(completed[1]["participant"], "agent-0");
    for (index, person) in people.iter().enumerate() {
        assert_eq!(person.observed.usage["totalTokens"], 120 + index);
        assert_eq!(person.observed.tool_calls, 1);
    }
    assert!(
        completed
            .iter()
            .all(|result| result["status"] == "completed")
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn substituted_model_is_rejected() -> Result<()> {
    let (_, results) = exchange("wrong_model").await?;
    assert_eq!(results[0]["status"], "harness_error");
    assert!(
        results[0]["error"]
            .as_str()
            .unwrap()
            .contains("substituted")
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn substituted_effort_is_rejected() -> Result<()> {
    let (_, results) = exchange("wrong_effort").await?;
    assert_eq!(results[0]["status"], "harness_error");
    assert!(
        results[0]["error"]
            .as_str()
            .unwrap()
            .contains("substituted")
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn native_tool_is_rejected() -> Result<()> {
    let (_, results) = exchange("native").await?;
    assert_eq!(results[0]["status"], "harness_error");
    assert!(results[0]["error"].as_str().unwrap().contains("native"));
    Ok(())
}

fn completed_guest(score_complete: bool) -> Value {
    json!({"kind":"guest_runtime","result":{
        "error":null,"runtime_exit_code":0,"protocol":{"status":"complete"},
        "backend_errors":[],"score":{"complete":score_complete}
    }})
}

#[hibana_runtime::main]
#[test]
async fn late_model_disconnect_after_guest_exit_fails_trial() -> Result<()> {
    let (_, results) = exchange("late_disconnect").await?;
    assert_eq!(results[0]["status"], "harness_error");
    let outcomes = [
        completed_guest(true),
        json!({"kind":"model_participant","result":results[0]}),
    ];
    assert!(
        crate::trial_error(&outcomes)
            .unwrap()
            .contains("harness_error")
    );
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn late_model_timeout_after_guest_exit_fails_trial() -> Result<()> {
    let (_, results) = exchange("late_timeout").await?;
    assert_eq!(results[0]["status"], "timeout");
    let outcomes = [
        completed_guest(true),
        json!({"kind":"model_participant","result":results[0]}),
    ];
    assert!(crate::trial_error(&outcomes).unwrap().contains("timeout"));
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn completed_model_and_terminal_domain_failure_remain_a_normal_trial() -> Result<()> {
    let (_, results) = exchange("normal").await?;
    let outcomes = [
        completed_guest(false),
        json!({"kind":"model_participant","result":results[0]}),
    ];
    assert_eq!(crate::trial_error(&outcomes), None);
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn group_error_is_preserved_after_every_participant_reports_completion() -> Result<()> {
    let error = exchange("missing_start_reply")
        .await
        .err()
        .expect("unfinished start RPC must fail the group");
    assert!(error.to_string().contains("unfinished model RPC requests"));
    Ok(())
}

#[test]
fn trial_rejects_model_error_and_guest_infrastructure_failure() {
    assert!(
        crate::trial_error(&[json!({"kind":"model_participant","result":{
            "participant":"agent","status":"completed","error":"connection failed"
        }})])
        .is_some()
    );
    for (field, value) in [
        ("error", json!("runtime closed")),
        ("runtime_exit_code", json!(1)),
        ("protocol", json!({"status":"failed"})),
        ("backend_errors", json!(["simulator closed"])),
    ] {
        let mut outcome = completed_guest(false);
        outcome["result"][field] = value;
        assert!(crate::trial_error(&[outcome]).is_some(), "{field}");
    }
    assert!(
        crate::trial_error(&[json!({"status":"harness_error","error":"job failed"})]).is_some()
    );
}
