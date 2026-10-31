use super::*;

#[hibana_runtime::main]
#[test]
async fn identical_concurrent_effect_arguments_keep_distinct_rpc_replies() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    async {
        let value = bus
            .execute("A", "inspect", json!({"target": 7}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(value, json!("first"));
    };
    async {
        let value = bus
            .execute("A", "inspect", json!({"target": 7}))
            .await
            .map_err(ShellError::message)?;
        assert_eq!(value, json!("second"));
    };
    async {
        hibana_runtime::yield_now().await;
        let requests =
            bus.connection(|connection| connection.outgoing.drain(..).collect::<Vec<_>>());
        assert_eq!(requests.len(), 2);
        let first = requests[0].0.last().unwrap();
        let second = requests[1].0.last().unwrap();
        assert_eq!(first["arguments"], second["arguments"]);
        assert_ne!(first["id"], second["id"]);
        bus.respond(second["id"].as_u64().unwrap(), json!("second"))?;
        bus.respond(first["id"].as_u64().unwrap(), json!("first"))?;
    };
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn output_receipt_waits_for_actual_write_completion() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    async {
        bus.emit(json!({"type": "exit", "status": 0})).await?;
    };
    async {
        hibana_runtime::yield_now().await;
        let (values, flushed) = bus
            .connection(|connection| connection.outgoing.pop_front())
            .unwrap();
        assert_eq!(values.last().unwrap()["type"], "exit");
        bus.respond(flushed.unwrap(), Value::Null)?;
    };
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn cancelling_one_rpc_removes_only_its_own_receipt() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let cancelled = bus.reply()?;
    let id = cancelled.id;
    let retained = bus.reply()?;
    let retained_id = retained.id;
    hibana_runtime::race(cancelled.receive(), async { Ok(Value::Null) }).await?;
    assert!(bus.respond(id, json!("late")).is_err());
    bus.respond(retained_id, json!("live"))?;
    assert_eq!(retained.receive().await?, json!("live"));
    assert!(bus.connection(|connection| connection.responses.is_empty()));
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn duplicate_rpc_reply_cannot_replace_the_first_value() -> ShellResult<()> {
    let (bus, _, _) = Bus::open([], []);
    let receipt = bus.reply()?;
    let id = receipt.id;
    bus.respond(id, json!("first"))?;
    assert!(bus.respond(id, json!("replacement")).is_err());
    assert_eq!(receipt.receive().await?, json!("first"));
    assert!(bus.respond(id, json!("replay")).is_err());
    Ok(())
}
