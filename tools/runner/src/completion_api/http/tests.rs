use super::*;
use crate::completion_api::tests::{chat, response};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyModule},
};
use tokio::{io::AsyncReadExt, net::TcpStream, sync::Notify};

fn directory() -> Result<PathBuf> {
    let directory = std::env::temp_dir().join(format!(
        "choreofix-api-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    Ok(directory)
}

fn check_recorded_history(directory: &Path, expected: &Value) -> Result<()> {
    let saved: Value =
        serde_json::from_slice(&fs::read(directory.join("app-server-history.json"))?)?;
    let saved = saved["items"].as_array().ok_or("recorded history items")?;
    let expected = expected.as_array().ok_or("original visible input")?;
    assert_eq!(saved.len(), expected.len());
    for (saved, expected) in saved.iter().zip(expected) {
        if expected.get("role").is_some() {
            assert_eq!(saved["type"], "message");
            assert_eq!(saved["role"], expected["role"]);
            assert_eq!(saved["content"].as_array().unwrap().len(), 1);
            assert_eq!(saved["content"][0]["text"], expected["content"]);
            assert_eq!(
                saved["content"][0]["type"],
                if expected["role"] == "assistant" {
                    "output_text"
                } else {
                    "input_text"
                }
            );
        } else {
            assert_eq!(saved, expected);
        }
    }
    Ok(())
}

async fn request(address: std::net::SocketAddr, value: Value) -> Result<(u16, Value)> {
    let mut stream = TcpStream::connect(address).await?;
    let body = value.to_string();
    stream.write_all(format!("POST /v1/chat/completions HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    let response = std::str::from_utf8(&response)?;
    let (header, body) = response
        .split_once("\r\n\r\n")
        .ok_or("HTTP response header")?;
    let status = header
        .split_whitespace()
        .nth(1)
        .ok_or("HTTP status")?
        .parse()?;
    Ok((status, serde_json::from_str(body)?))
}

#[tokio::test(flavor = "current_thread")]
async fn independent_completions_run_while_another_request_waits() -> Result<()> {
    let output = directory()?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let address = listener.local_addr()?;
    let entered = Rc::new(Notify::new());
    let release = Rc::new(Notify::new());
    let forward = {
        let entered = entered.clone();
        let release = release.clone();
        move |request: Value, _: PathBuf| {
            let entered = entered.clone();
            let release = release.clone();
            async move {
                assert_eq!(request["reasoning"]["effort"], EFFORT);
                if request["input"][1]["content"] == "wait" {
                    entered.notify_one();
                    release.notified().await;
                }
                Ok((StatusCode::OK, response()))
            }
        }
    };
    let client = async {
        let mut slow = chat();
        slow["messages"][1]["content"] = json!("wait");
        let slow = request(address, slow);
        let fast = async {
            entered.notified().await;
            let result = request(address, chat()).await?;
            assert_eq!(result.0, 200);
            assert_eq!(result.1["usage"]["total_tokens"], 130);
            release.notify_one();
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        };
        let (slow, fast) = tokio::join!(slow, fast);
        assert_eq!(slow?.0, 200);
        fast?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        tokio::select! {
            result = connections(listener, output.clone(), forward) => {
                result?;
                Err("HTTP server stopped unexpectedly".into())
            }
            result = client => result,
        }
    })
    .await??;
    for id in 1..=2 {
        let record = output.join(format!("request-{id:08}"));
        for file in [
            "client-request.json",
            "responses-request.json",
            "app-server-history.json",
            "responses-response.json",
            "client-response.json",
            "transport.json",
        ] {
            assert!(record.join(file).is_file());
        }
    }
    fs::remove_dir_all(output)?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn unqualified_request_does_not_reach_the_upstream() -> Result<()> {
    let output = directory()?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let address = listener.local_addr()?;
    let forward = |_: Value, _: PathBuf| async { panic!("invalid request reached upstream") };
    let mut invalid = chat();
    invalid["model"] = json!("different-model");
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        tokio::select! {
            result = connections(listener, output.clone(), forward) => {
                result?;
                Err("HTTP server stopped unexpectedly".into())
            }
            result = request(address, invalid) => {
                assert_eq!(result?.0, 400);
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            }
        }
    })
    .await??;
    assert!(
        !output
            .join("request-00000001/responses-request.json")
            .exists()
    );
    fs::remove_dir_all(output)?;
    Ok(())
}

// Invoke the original comparison clients, not a reimplementation of their SDK
// calls. The upstream here is a wire fixture and makes no model requests.
fn original_module<'py>(py: Python<'py>, module: &str) -> Result<Bound<'py, PyModule>> {
    if std::env::var("PYTHONHASHSEED").as_deref() != Ok("0") {
        return Err("PYTHONHASHSEED=0 is required before loading comparison clients".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let paths = PyModule::import(py, "sys")?.getattr("path")?;
    paths.call_method1(
        "insert",
        (0, root.join("vendor/tracefix-paper").to_str().unwrap()),
    )?;
    paths.call_method1(
        "insert",
        (0, root.join("vendor/tracefix").to_str().unwrap()),
    )?;
    for library in fs::read_dir(root.join(".venv/lib"))? {
        let packages = library?.path().join("site-packages");
        if packages.is_dir() {
            paths.call_method1("append", (packages.to_str().unwrap(),))?;
        }
    }
    let source = PyModule::import(py, module)?;
    let sdk = PyModule::import(py, "openai")?;
    let sdk_version = sdk.getattr("__version__")?.extract::<String>()?;
    let contract: Value =
        serde_json::from_slice(&fs::read(root.join("benchmark/experiment-contract.json"))?)?;
    assert_eq!(
        contract["participant_api"]["original_client_sdk"]["version"],
        sdk_version
    );
    println!(
        "comparison client={module} SDK={sdk_version} source={}",
        source.getattr("__file__")?.extract::<String>()?
    );
    let expected_source = if module.starts_with("tracefix.") {
        root.join("vendor/tracefix")
    } else {
        root.join("vendor/tracefix-paper")
    };
    let loaded = PathBuf::from(source.getattr("__file__")?.extract::<String>()?).canonicalize()?;
    assert!(loaded.starts_with(expected_source));
    Ok(source)
}

fn original_client(module: &str, base_url: &str) -> Result<()> {
    Python::initialize();
    Python::attach(|py| -> Result<()> {
        let source = original_module(py, module)?;
        let json_module = PyModule::import(py, "json")?;
        let mut request = chat();
        request["messages"][0]["content"] = json!(format!("Original role: {module}"));
        let messages = json_module.call_method1("loads", (request["messages"].to_string(),))?;
        let tools = json_module.call_method1("loads", (request["tools"].to_string(),))?;
        let messages = messages.cast::<PyList>().map_err(pyo3::PyErr::from)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("model", MODEL)?;
        kwargs.set_item("api_key", "fixture-not-a-credential")?;
        if module == "runtime_A.llm_policy" {
            let policy = source
                .getattr("LLMPolicy")?
                .call((PyDict::new(py),), Some(&kwargs))?;
            let client = policy.call_method0("_get_async_client")?;
            client.setattr("base_url", base_url)?;
            let event_loop = PyModule::import(py, "asyncio")?.call_method0("new_event_loop")?;
            let first = event_loop.call_method1(
                "run_until_complete",
                (policy.call_method1("_call_openai_async", (messages, &tools))?,),
            )?;
            let message = first.getattr("choices")?.get_item(0)?.getattr("message")?;
            // Match the original mediator's manual assistant dictionary.
            let serialized: Value = serde_json::from_str(
                &message
                    .call_method0("model_dump_json")?
                    .extract::<String>()?,
            )?;
            let manual = json!({"role":"assistant","content":serialized["content"],"tool_calls":serialized["tool_calls"]});
            messages.append(json_module.call_method1("loads", (manual.to_string(),))?)?;
            for call in serialized["tool_calls"]
                .as_array()
                .ok_or("fixture tool calls")?
            {
                let result =
                    json!({"role":"tool","tool_call_id":call["id"],"content":"fixture result"});
                messages.append(json_module.call_method1("loads", (result.to_string(),))?)?;
            }
            let second = event_loop.call_method1(
                "run_until_complete",
                (policy.call_method1("_call_openai_async", (messages, &tools))?,),
            )?;
            assert_eq!(
                second
                    .getattr("usage")?
                    .getattr("total_tokens")?
                    .extract::<u64>()?,
                130
            );
            event_loop.call_method1("run_until_complete", (client.call_method0("close")?,))?;
            event_loop.call_method0("close")?;
        } else {
            kwargs.set_item("agent_id", "fixture")?;
            kwargs.set_item("system_prompt", "Original role instructions")?;
            kwargs.set_item("tool_schemas", &tools)?;
            let config = source.getattr("AgentConfig")?.call((), Some(&kwargs))?;
            let runner = source.getattr("AgentRunner")?.call1((config, py.None()))?;
            let client = runner.getattr("_llm_client")?;
            client.setattr("base_url", base_url)?;
            runner.setattr("messages", messages)?;
            let first = runner.call_method0("_call_llm")?;
            let message = first.getattr("choices")?.get_item(0)?.getattr("message")?;
            let serialized = message.call_method0("to_dict")?;
            messages.append(&serialized)?;
            let serialized: Value = serde_json::from_str(
                &json_module
                    .call_method1("dumps", (&serialized,))?
                    .extract::<String>()?,
            )?;
            for call in serialized["tool_calls"]
                .as_array()
                .ok_or("fixture tool calls")?
            {
                let result =
                    json!({"role":"tool","tool_call_id":call["id"],"content":"fixture result"});
                messages.append(json_module.call_method1("loads", (result.to_string(),))?)?;
            }
            let second = runner.call_method0("_call_llm")?;
            assert_eq!(
                second
                    .getattr("usage")?
                    .getattr("total_tokens")?
                    .extract::<u64>()?,
                130
            );
            client.call_method0("close")?;
        }
        Ok(())
    })
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "loads all five unchanged comparison clients and their OpenAI SDK"]
async fn original_sync_and_async_comparison_clients_replay_tool_rounds() -> Result<()> {
    let output = directory()?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let base_url = format!("http://{}/v1", listener.local_addr()?);
    let forward = |request: Value, directory: PathBuf| async move {
        check_recorded_history(&directory, &request["input"])?;
        let label = request["input"][0]["content"]
            .as_str()
            .ok_or("original prompt")?;
        let mut upstream = response();
        upstream["output"][0]["encrypted_content"] =
            json!(format!("fixture reasoning for {label}"));
        for index in 2..=3 {
            upstream["output"][index]["call_id"] = json!(format!("call_{index}_{label}"));
        }
        if request["input"].as_array().unwrap().len() > 2 {
            assert_eq!(
                request["input"][2]["content"],
                upstream["output"][1]["content"][0]["text"]
            );
            assert_eq!(request["input"][5]["type"], "function_call_output");
            assert_eq!(request["input"][6]["type"], "function_call_output");
            assert!(
                request["input"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|item| item["type"] != "reasoning")
            );
            upstream["output"].as_array_mut().unwrap().truncate(2);
            upstream["output"][1]["content"][0]["text"] = json!("Fixture complete.");
        }
        Ok((StatusCode::OK, upstream))
    };
    let client = tokio::task::spawn_blocking(move || -> Result<()> {
        for module in [
            "runtime_B.agent_runner",
            "runtime_base_1.orchestrator",
            "runtime_base_2.orchestrator",
            "runtime_A.llm_policy",
            "tracefix.runtime.monitoring.agent_runner",
        ] {
            original_client(module, &base_url)?;
        }
        Ok(())
    });
    tokio::time::timeout(std::time::Duration::from_secs(45), async {
        tokio::select! {
            result = connections(listener, output.clone(), forward) => {
                result?;
                Err("HTTP server stopped unexpectedly".into())
            }
            result = client => result?,
        }
    })
    .await??;
    assert_eq!(fs::read_dir(&output)?.count(), 10);
    fs::remove_dir_all(output)?;
    Ok(())
}

fn python_value(value: &Bound<'_, PyAny>) -> Result<Value> {
    let json = PyModule::import(value.py(), "json")?;
    Ok(serde_json::from_str(
        &json.call_method1("dumps", (value,))?.extract::<String>()?,
    )?)
}

// Exercise both published generator clients and their own history conversion
// and compression. This fixture never runs a model or a protocol compiler.
fn original_pipeline(prefix: &str, base_url: &str) -> Result<Vec<Value>> {
    let scratch = directory()?;
    Python::initialize();
    let expected = Python::attach(|py| -> Result<Vec<Value>> {
        let json = PyModule::import(py, "json")?;
        let config_source = original_module(py, &format!("{prefix}.pipeline.llm_client"))?;
        let tools_source = original_module(py, &format!("{prefix}.tool_client"))?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("provider", "openai")?;
        kwargs.set_item("model", MODEL)?;
        kwargs.set_item("reasoning_effort", EFFORT)?;
        kwargs.set_item("api_key", "fixture-not-a-credential")?;
        kwargs.set_item("base_url", base_url)?;
        let config = config_source
            .getattr("LLMConfig")?
            .call((), Some(&kwargs))?;
        let definitions = json!([chat()["tools"][0]["function"]]);
        let tools = json.call_method1("loads", (definitions.to_string(),))?;
        let client = tools_source
            .getattr("ToolClient")?
            .call1((&config, tools))?;
        let initial = json!([
            {"role":"system","content":format!("Original pipeline: {prefix}")},
            {"role":"user","content":"Generate the assigned protocol."}
        ]);
        let messages = json.call_method1("loads", (initial.to_string(),))?;
        let messages = messages.cast::<PyList>().map_err(pyo3::PyErr::from)?;
        let mut expected = vec![
            json!({"input":python_value(&client.call_method1("_to_responses_input", (messages,))?)?,"format":null}),
        ];
        let first = client.call_method1("chat", (messages,))?;
        assert_eq!(python_value(&first.getattr("usage")?)?["total_tokens"], 130);
        let mut calls = Vec::new();
        for call in first.getattr("tool_calls")?.try_iter()? {
            let call = call?;
            calls.push(json!({"id":call.getattr("id")?.extract::<String>()?,
                "name":call.getattr("name")?.extract::<String>()?,
                "arguments":python_value(&call.getattr("arguments")?)?}));
        }
        assert_eq!(calls.len(), 2);
        let assistant = json!({"role":"assistant","content":first.getattr("text")?.extract::<Option<String>>()?,"tool_calls":calls});
        messages.append(json.call_method1("loads", (assistant.to_string(),))?)?;
        for call in &calls {
            let result = json!({"role":"tool_result","tool_call_id":call["id"],"content":"original tool result ".repeat(30)});
            messages.append(json.call_method1("loads", (result.to_string(),))?)?;
        }
        let loop_source = original_module(py, &format!("{prefix}.loop"))?;
        let workspace_source = original_module(py, &format!("{prefix}.workspace"))?;
        let workspace_kwargs = PyDict::new(py);
        workspace_kwargs.set_item("session_id", "compression-fixture")?;
        workspace_kwargs.set_item("base_dir", scratch.to_str().unwrap())?;
        let workspace = workspace_source
            .getattr("Workspace")?
            .call((), Some(&workspace_kwargs))?;
        let loop_kwargs = PyDict::new(py);
        loop_kwargs.set_item("tool_client", &client)?;
        loop_kwargs.set_item("workspace", workspace)?;
        loop_kwargs.set_item("system_prompt", format!("Original pipeline: {prefix}"))?;
        loop_kwargs.set_item("context_limit_chars", 1)?;
        let agent_loop = loop_source
            .getattr("AgentLoop")?
            .call((), Some(&loop_kwargs))?;
        let recent = loop_source
            .getattr("KEEP_RECENT_MESSAGES")?
            .extract::<usize>()?;
        for index in 0..recent {
            let message = json!({"role":"user","content":format!("recent message {index}")});
            messages.append(json.call_method1("loads", (message.to_string(),))?)?;
        }
        agent_loop.setattr("messages", messages)?;
        let before = python_value(messages)?;
        agent_loop.call_method0("_maybe_compress_context")?;
        let compressed = agent_loop.getattr("messages")?;
        let after = python_value(&compressed)?;
        assert!(after.to_string().len() < before.to_string().len());
        assert_ne!(after[2]["content"], before[2]["content"]);
        assert_ne!(
            after[2]["tool_calls"][0]["arguments"],
            before[2]["tool_calls"][0]["arguments"]
        );
        let oracle = python_value(&client.call_method1("_to_responses_input", (&compressed,))?)?;
        expected.push(json!({"input":oracle,"format":null}));
        let second = client.call_method1("chat", (&compressed,))?;
        assert_eq!(
            second.getattr("text")?.extract::<String>()?,
            "Fixture complete."
        );
        client.getattr("_openai_client")?.call_method0("close")?;

        let inner = config_source.getattr("LLMClient")?.call1((&config,))?;
        let format = json.call_method1("loads", ("{\"type\":\"json_object\"}",))?;
        let inner_kwargs = PyDict::new(py);
        inner_kwargs.set_item("response_format", format)?;
        let system = format!("Original inner generator: {prefix}");
        let user = "Return the original IR JSON.";
        let generated = inner.call_method("chat", (&system, user), Some(&inner_kwargs))?;
        assert_eq!(
            generated.getattr("content")?.extract::<String>()?,
            "{\"agents\":[]}"
        );
        assert_eq!(generated.getattr("model")?.extract::<String>()?, MODEL);
        assert_eq!(
            python_value(&generated.getattr("usage")?)?["total_tokens"],
            130
        );
        expected.push(json!({"input":[{"role":"system","content":system},{"role":"user","content":user}],"format":{"type":"json_object"}}));
        inner.getattr("_openai_client")?.call_method0("close")?;
        Ok(expected)
    })?;
    fs::remove_dir_all(scratch)?;
    Ok(expected)
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "loads the two unchanged protocol generators, context compression and JSON clients"]
async fn original_pipeline_clients_keep_compressed_history_and_json_output() -> Result<()> {
    let output = directory()?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let base_url = format!("http://{}/v1", listener.local_addr()?);
    let forward = |request: Value, _: PathBuf| async move {
        let mut upstream = response();
        if request["text"]["format"]["type"] == "json_object" {
            upstream["output"].as_array_mut().unwrap().truncate(2);
            upstream["output"][1]["content"][0]["text"] = json!("{\"agents\":[]}");
        } else if request["input"].as_array().unwrap().len() > 2 {
            upstream["output"].as_array_mut().unwrap().truncate(2);
            upstream["output"][1]["content"][0]["text"] = json!("Fixture complete.");
        } else {
            upstream["output"][1]["content"][0]["text"] =
                json!("Original assistant explanation ".repeat(20));
            upstream["output"][2]["arguments"] = json!(json!({"channel_id":"a","body":format!("{}火花", "Original file contents ".repeat(20))}).to_string());
        }
        Ok((StatusCode::OK, upstream))
    };
    let client = tokio::task::spawn_blocking(move || -> Result<Vec<Value>> {
        let mut expected = Vec::new();
        for prefix in ["v3_agent_test", "tracefix.pipeline"] {
            expected.extend(original_pipeline(prefix, &base_url)?);
        }
        Ok(expected)
    });
    let expected = tokio::time::timeout(std::time::Duration::from_secs(45), async {
        tokio::select! {
            result = connections(listener, output.clone(), forward) => {
                result?;
                Err("HTTP server stopped unexpectedly".into())
            }
            result = client => result?,
        }
    })
    .await??;
    assert_eq!(expected.len(), 6);
    assert_eq!(fs::read_dir(&output)?.count(), 6);
    for (index, expected) in expected.iter().enumerate() {
        let request: Value = serde_json::from_slice(&fs::read(
            output.join(format!("request-{:08}/responses-request.json", index + 1)),
        )?)?;
        assert_eq!(request["input"], expected["input"]);
        assert_eq!(request["text"]["format"], expected["format"]);
        assert_eq!(request["reasoning"]["effort"], EFFORT);
        assert_eq!(request["store"], false);
        check_recorded_history(
            &output.join(format!("request-{:08}", index + 1)),
            &expected["input"],
        )?;
    }
    fs::remove_dir_all(output)?;
    Ok(())
}
