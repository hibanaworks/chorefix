use super::*;

pub(super) fn chat() -> Value {
    json!({"model":MODEL,"messages":[{"role":"system","content":"Original role instructions\nNo additional harness prompt."},
        {"role":"user","content":"Do the assigned work."}],
        "tools":[{"type":"function","function":{"name":"receive_message","description":"Original tool",
            "parameters":{"type":"object","properties":{"channel_id":{"type":"string"},"body":{"type":"string"}},
                "required":["channel_id"]}}}]})
}

pub(super) fn response() -> Value {
    json!({"id":"resp_1","model":MODEL,"reasoning":{"effort":EFFORT},"created_at":1234,"status":"completed",
        "output":[{"type":"reasoning","id":"rs_1","encrypted_content":"opaque reasoning","summary":[]},
            {"type":"message","id":"msg_1","role":"assistant","status":"completed",
                "content":[{"type":"output_text","text":"I will receive both inputs.","annotations":[]}]},
            {"type":"function_call","id":"fc_1","call_id":"call_a","name":"receive_message","arguments":"{\"channel_id\":\"a\"}","status":"completed"},
            {"type":"function_call","id":"fc_2","call_id":"call_b","name":"receive_message","arguments":"{\"channel_id\":\"b\"}","status":"completed"}],
        "usage":{"input_tokens":100,"output_tokens":30,"total_tokens":130,
            "input_tokens_details":{"cached_tokens":60},"output_tokens_details":{"reasoning_tokens":20}}})
}

#[test]
fn preserves_prompts_schemas_optional_arguments_and_parallel_tool_policy() -> Result<()> {
    let mut chat = chat();
    chat["parallel_tool_calls"] = json!(false);
    chat["tool_choice"] = json!({"type":"function","function":{"name":"receive_message"}});
    let request = super::request(&chat)?;
    assert_eq!(request["input"], chat["messages"]);
    assert_eq!(
        request["tools"][0]["parameters"],
        chat["tools"][0]["function"]["parameters"]
    );
    assert_eq!(request["tools"][0]["strict"], false);
    assert_eq!(
        request["tool_choice"],
        json!({"type":"function","name":"receive_message"})
    );
    assert_eq!(request["parallel_tool_calls"], false);
    assert_eq!(request["reasoning"]["effort"], EFFORT);
    assert_eq!(request["store"], false);
    assert!(request.get("previous_response_id").is_none());
    assert!(request.get("conversation").is_none());
    assert!(request.get("instructions").is_none());
    Ok(())
}

#[test]
fn keeps_visible_output_call_ids_result_order_and_inclusive_usage() -> Result<()> {
    let mut chat = chat();
    let completion = super::response(&response())?;
    let message = &completion["choices"][0]["message"];
    let messages = chat["messages"].as_array_mut().unwrap();
    messages.push(message.clone());
    messages.push(
        json!({"role":"tool","tool_call_id":"call_b","content":"second result arrives first"}),
    );
    messages.push(
        json!({"role":"tool","tool_call_id":"call_a","content":"first result arrives second"}),
    );
    assert_eq!(
        super::request(&chat)?["input"],
        json!([
            chat["messages"][0], chat["messages"][1],
            {"role":"assistant","content":"I will receive both inputs."},
            {"type":"function_call","call_id":"call_a","name":"receive_message","arguments":"{\"channel_id\":\"a\"}"},
            {"type":"function_call","call_id":"call_b","name":"receive_message","arguments":"{\"channel_id\":\"b\"}"},
            {"type":"function_call_output","call_id":"call_b","output":"second result arrives first"},
            {"type":"function_call_output","call_id":"call_a","output":"first result arrives second"}
        ])
    );
    assert_eq!(completion["usage"]["total_tokens"], 130);
    assert_eq!(completion["usage"]["completion_tokens"], 30);
    assert_eq!(
        completion["usage"]["completion_tokens_details"]["reasoning_tokens"],
        20
    );
    let mut fragmented = response();
    fragmented["output"][1]["content"] = json!([
        {"type":"output_text","text":"first"},
        {"type":"output_text","text":""},
        {"type":"output_text","text":"second"}
    ]);
    assert_eq!(
        super::response(&fragmented)?["choices"][0]["message"]["content"],
        "first\nsecond"
    );
    Ok(())
}

#[test]
fn client_reserialization_compression_and_history_trimming_are_preserved() -> Result<()> {
    let mut chat = chat();
    let completion = super::response(&response())?;
    let message = completion["choices"][0]["message"].clone();
    chat["messages"].as_array_mut().unwrap().push(message);
    let arguments = "{ \"channel_id\" : \"a\", \"body\" : \"\\u706b\\u82b1\" }";
    chat["messages"][2]["tool_calls"][0]["function"]["arguments"] = json!(arguments);
    assert_eq!(super::request(&chat)?["input"][3]["arguments"], arguments);
    chat["messages"][2]["content"] = json!("(compressed)");
    chat["messages"][2]["tool_calls"][0]["function"]["arguments"] =
        json!("{\"body\":\"(500 chars, compressed)\"}");
    let input = super::request(&chat)?["input"].clone();
    assert_eq!(input[2]["content"], "(compressed)");
    assert_eq!(
        input[3]["arguments"],
        "{\"body\":\"(500 chars, compressed)\"}"
    );
    assert!(
        input
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["type"] != "reasoning")
    );
    chat["messages"].as_array_mut().unwrap().remove(1);
    assert_eq!(super::request(&chat)?["input"][1], input[2]);
    // Identical visible answers need no shared or inferred conversation state.
    chat["messages"][0]["content"] = json!("Other original participant instructions");
    assert_eq!(super::request(&chat)?["input"][1], input[2]);
    Ok(())
}

#[test]
fn structured_output_formats_preserve_the_original_schema_and_strictness() -> Result<()> {
    let mut chat = chat();
    for format in [json!({"type":"text"}), json!({"type":"json_object"})] {
        chat["response_format"] = format.clone();
        assert_eq!(super::request(&chat)?["text"]["format"], format);
    }
    for strict in [false, true] {
        let schema = json!({"name":"original_ir","description":"Original IR schema","strict":strict,
            "schema":{"type":"object","properties":{"agents":{"type":"array","items":{"type":"string"}}},"required":["agents"]}});
        chat["response_format"] = json!({"type":"json_schema","json_schema":schema});
        let format = super::request(&chat)?["text"]["format"].clone();
        assert_eq!(format["type"], "json_schema");
        for field in ["name", "description", "schema", "strict"] {
            assert_eq!(format[field], schema[field]);
        }
    }
    for invalid in [
        json!({"type":"json_object","unexpected":true}),
        json!({"type":"other"}),
        json!({"type":"json_schema","json_schema":{"name":"missing_schema"}}),
        json!({"type":"json_schema","json_schema":{"name":"x","schema":{},"strict":"yes"}}),
    ] {
        chat["response_format"] = invalid;
        assert!(super::request(&chat).is_err());
    }
    Ok(())
}

#[test]
fn unqualified_inputs_outputs_and_native_tool_types_are_rejected() -> Result<()> {
    let initial = chat();
    for (field, value) in [
        ("model", json!("other-model")),
        ("reasoning_effort", json!("none")),
        ("n", json!(2)),
        ("stream", json!(true)),
        ("temperature", json!(0.5)),
    ] {
        let mut request = initial.clone();
        request[field] = value;
        assert!(super::request(&request).is_err());
    }
    for (field, value) in [
        ("model", json!("other-model")),
        ("reasoning", json!({"effort":"high"})),
        ("status", json!("incomplete")),
        ("id", Value::Null),
        ("created_at", Value::Null),
        (
            "usage",
            json!({"input_tokens":100,"output_tokens":30,"total_tokens":150}),
        ),
        ("output", json!([{"type":"web_search_call"}])),
    ] {
        let mut upstream = response();
        upstream[field] = value;
        assert!(super::response(&upstream).is_err());
    }
    Ok(())
}
