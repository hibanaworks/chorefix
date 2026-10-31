//! Model API conversion for unchanged comparison clients. No task authority.
use crate::{
    evidence::Result,
    model::{EFFORT, MODEL},
};
use serde_json::{Value, json};

mod history;
pub mod http;

fn assistant_items(message: &Value) -> Result<Vec<Value>> {
    let mut items = Vec::new();
    match &message["content"] {
        Value::Null => {}
        Value::String(text) if text.is_empty() => {}
        Value::String(text) => items.push(json!({"role":"assistant","content":text})),
        _ => return Err("comparison assistant content must be text or null".into()),
    }
    if let Some(value) = message.get("tool_calls").filter(|value| !value.is_null()) {
        for call in value
            .as_array()
            .ok_or("assistant tool calls must be an array")?
        {
            if call["type"] != "function" {
                return Err("only function calls are supported by the comparison clients".into());
            }
            items.push(json!({"type":"function_call",
                "call_id":call["id"].as_str().ok_or("tool call id")?,
                "name":call["function"]["name"].as_str().ok_or("function name")?,
                "arguments":call["function"]["arguments"].as_str().ok_or("function arguments")?
            }));
        }
    }
    Ok(items)
}

/// Convert only the history supplied by the original client. In particular,
/// do not restore text or arguments removed by its context compression.
pub fn request(chat: &Value) -> Result<Value> {
    let object = chat
        .as_object()
        .ok_or("completion request must be an object")?;
    for field in object.keys() {
        if ![
            "model",
            "messages",
            "tools",
            "tool_choice",
            "parallel_tool_calls",
            "reasoning_effort",
            "max_completion_tokens",
            "max_tokens",
            "n",
            "stream",
            "response_format",
        ]
        .contains(&field.as_str())
        {
            return Err(format!("unsupported completion request field: {field}").into());
        }
    }
    if chat["model"] != MODEL
        || chat
            .get("reasoning_effort")
            .is_some_and(|value| value != EFFORT)
        || chat.get("n").is_some_and(|value| value != 1)
        || chat.get("stream").is_some_and(|value| value != false)
    {
        return Err("comparison requires Luna xhigh and one nonstreaming response".into());
    }
    let messages = chat["messages"].as_array().ok_or("completion messages")?;
    let mut input = Vec::new();
    for message in messages {
        match message["role"].as_str() {
            Some("system" | "developer" | "user") => {
                if !message["content"].is_string() {
                    return Err("comparison input messages must contain text".into());
                }
                input.push(message.clone());
            }
            Some("tool") => input.push(json!({"type":"function_call_output",
                    "call_id":message["tool_call_id"].as_str().ok_or("tool result call id")?,
                    "output":message["content"].as_str().ok_or("tool result text")?})),
            Some("assistant") => input.extend(assistant_items(message)?),
            _ => return Err("unsupported completion message role".into()),
        }
    }
    let mut request = json!({"model":MODEL,"reasoning":{"effort":EFFORT},
            "input":input,"store":false});
    if let Some(tools) = chat.get("tools").filter(|value| !value.is_null()) {
        let mut functions = Vec::new();
        for tool in tools.as_array().ok_or("completion tools")? {
            if tool["type"] != "function" {
                return Err("comparison clients require function tools".into());
            }
            let mut function = tool["function"]
                .as_object()
                .ok_or("function schema")?
                .clone();
            if function
                .keys()
                .any(|key| !["name", "description", "parameters", "strict"].contains(&key.as_str()))
            {
                return Err("unsupported function definition field".into());
            }
            function.insert("type".into(), json!("function"));
            // Chat Completions defaults to non-strict schemas. Responses
            // would otherwise make optional arguments required.
            function.entry("strict").or_insert(json!(false));
            functions.push(Value::Object(function));
        }
        request["tools"] = json!(functions);
    }
    if let Some(choice) = chat.get("tool_choice") {
        request["tool_choice"] = if let Some(choice) = choice.as_str() {
            if !["auto", "none", "required"].contains(&choice) {
                return Err("unsupported tool choice".into());
            }
            json!(choice)
        } else if choice["type"] == "function" {
            json!({"type":"function","name":choice["function"]["name"].as_str().ok_or("chosen function")?})
        } else {
            return Err("unsupported tool choice".into());
        };
    }
    if let Some(parallel) = chat.get("parallel_tool_calls") {
        request["parallel_tool_calls"] = json!(parallel.as_bool().ok_or("parallel tool calls")?);
    }
    let maximum = match (chat.get("max_tokens"), chat.get("max_completion_tokens")) {
        (Some(_), Some(_)) => return Err("conflicting output token limits".into()),
        (maximum, None) | (None, maximum) => maximum,
    };
    if let Some(maximum) = maximum {
        request["max_output_tokens"] = json!(maximum.as_u64().ok_or("output token limit")?);
    }
    if let Some(format) = chat.get("response_format") {
        request["text"] = json!({"format":match format["type"].as_str() {
            Some("text" | "json_object") if format.as_object().is_some_and(|fields| fields.len() == 1) => format.clone(),
            Some("json_schema") => {
                let mut schema = format["json_schema"].as_object().ok_or("JSON response schema")?.clone();
                if format.as_object().is_none_or(|fields| fields.len() != 2)
                    || schema.keys().any(|key| !["name", "description", "schema", "strict"].contains(&key.as_str()))
                    || !schema.get("name").is_some_and(Value::is_string)
                    || !schema.get("schema").is_some_and(Value::is_object)
                    || schema.get("strict").is_some_and(|value| !value.is_boolean())
                    || schema.get("description").is_some_and(|value| !value.is_string()) {
                    return Err("unsupported JSON response schema field".into());
                }
                schema.insert("type".into(), json!("json_schema"));
                Value::Object(schema)
            }
            _ => return Err("unsupported response format".into()),
        }});
    }
    Ok(request)
}

pub fn response(response: &Value) -> Result<Value> {
    if response["status"] != "completed"
        || response["model"] != MODEL
        || response["reasoning"]["effort"] != EFFORT
    {
        return Err("upstream response is incomplete or substituted the model or effort".into());
    }
    let usage = &response["usage"];
    let input_tokens = usage["input_tokens"]
        .as_u64()
        .ok_or("missing input token usage")?;
    let output_tokens = usage["output_tokens"]
        .as_u64()
        .ok_or("missing output token usage")?;
    let total_tokens = usage["total_tokens"]
        .as_u64()
        .ok_or("missing inclusive token usage")?;
    if input_tokens.checked_add(output_tokens) != Some(total_tokens) {
        return Err("inconsistent inclusive token usage".into());
    }
    let items = response["output"]
        .as_array()
        .ok_or("Responses output items")?;
    let mut text = Vec::new();
    let mut refusal = String::new();
    let mut calls = Vec::new();
    for item in items {
        match item["type"].as_str() {
            // The original client's native Responses path also keeps only
            // visible messages and function calls in its next input.
            Some("reasoning") => {}
            Some("function_call") => calls.push(
                json!({"id":item["call_id"].as_str().ok_or("output call id")?,
                "type":"function","function":{
                    "name":item["name"].as_str().ok_or("output function name")?,
                    "arguments":item["arguments"].as_str().ok_or("output function arguments")?
                }}),
            ),
            Some("message") if item["role"] == "assistant" => {
                for content in item["content"]
                    .as_array()
                    .ok_or("assistant output content")?
                {
                    match content["type"].as_str() {
                        Some("output_text") => {
                            let fragment =
                                content["text"].as_str().ok_or("assistant output text")?;
                            if !fragment.is_empty() {
                                text.push(fragment);
                            }
                        }
                        Some("refusal") => refusal
                            .push_str(content["refusal"].as_str().ok_or("assistant refusal")?),
                        _ => return Err("unsupported assistant output content".into()),
                    }
                }
            }
            _ => {
                return Err("unexpected Responses item outside the original function tools".into());
            }
        }
    }
    let text = text.join("\n");
    let finish_reason = if calls.is_empty() {
        "stop"
    } else {
        "tool_calls"
    };
    let message = json!({"role":"assistant","content":if text.is_empty() { Value::Null } else { json!(text) },
            "tool_calls":if calls.is_empty() { Value::Null } else { json!(calls) },
            "refusal":if refusal.is_empty() { Value::Null } else { json!(refusal) }});
    let completion = json!({"id":response["id"].as_str().ok_or("response id")?,
            "object":"chat.completion","created":response["created_at"].as_u64().ok_or("response creation time")?,
            "model":response["model"],"choices":[{"index":0,"message":message,"finish_reason":finish_reason}],
            "usage":{"prompt_tokens":input_tokens,"completion_tokens":output_tokens,"total_tokens":total_tokens,
                "prompt_tokens_details":usage["input_tokens_details"],
                "completion_tokens_details":usage["output_tokens_details"]}});
    Ok(completion)
}

#[cfg(test)]
mod tests;
