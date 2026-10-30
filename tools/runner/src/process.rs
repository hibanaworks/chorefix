//! Stdio correlation for the real guest process and original simulator.
//! These queues carry model commands or already-admitted external effects.
//! No task phase, ordering, resource owner or permission lives here.
use crate::{
    evidence::{Result, Trace},
    wire,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs::File, path::Path, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::{mpsc, oneshot},
    task::JoinSet,
};

type Request = (Value, oneshot::Sender<Value>);
pub type Simulator = Arc<crate::oracle::Simulator>;

pub async fn simulator(
    task: &str,
    depth: u8,
    output: &Path,
    trace: Arc<Trace>,
) -> Result<(Simulator, Value)> {
    let config: Value = serde_json::from_slice(&std::fs::read("experiment.json")?)?;
    if config["python_hash_seed"] != 0 {
        return Err("the recorded upstream interpreter seed must be zero".into());
    }
    let delay = config["domain_tool_delay_multiplier"]
        .as_f64()
        .ok_or("tool delay multiplier")?;
    let result = crate::oracle::Simulator::new(task, depth, delay, trace, &[])?;
    crate::evidence::save(&output.join("simulator-fixture.json"), &result.1)?;
    Ok(result)
}

#[derive(Clone)]
pub struct Guest(mpsc::UnboundedSender<Request>);
impl Guest {
    pub async fn command(&self, agent: &str, command: String) -> Result<Value> {
        let (send, receive) = oneshot::channel();
        self.0
            .send((json!({"agent":agent,"command":command}), send))
            .map_err(|_| "guest runtime closed")?;
        Ok(receive
            .await
            .map_err(|_| "guest command direction closed")?)
    }
}

pub fn guest(
    runtime: &Path,
    module: &Path,
    task: &str,
    output: &Path,
    trace: Arc<Trace>,
    simulator: Simulator,
) -> Result<(Guest, impl Future<Output = Result<Value>> + use<>)> {
    let mut child = Command::new(runtime)
        .arg(module)
        .arg(task)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(File::create_new(output.join("runtime-stderr.log"))?)
        .kill_on_drop(true)
        .spawn()?;
    let input = child.stdin.take().ok_or("runtime stdin")?;
    let lines = BufReader::new(child.stdout.take().ok_or("runtime stdout")?).lines();
    let (send, receive) = mpsc::unbounded_channel();
    Ok((
        Guest(send),
        runtime_loop(child, input, lines, receive, trace, simulator),
    ))
}

async fn runtime_loop(
    mut child: Child,
    mut input: tokio::process::ChildStdin,
    mut lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    mut commands: mpsc::UnboundedReceiver<Request>,
    trace: Arc<Trace>,
    simulator: Simulator,
) -> Result<Value> {
    let mut backend = JoinSet::new();
    let verdict: Result<Value> = async {
        let mut sequence = 0u64;
        let mut pending = BTreeMap::<u64,oneshot::Sender<Value>>::new();
        loop {
            tokio::select! {
                Some((mut command, response)) = commands.recv() => {
                    sequence += 1;
                    command["type"] = json!("command");
                    command["id"] = json!(sequence);
                    pending.insert(sequence,response);
                    trace.record("to_runtime",&command)?;
                    wire::send(&mut input,&command).await?;
                }
                line = lines.next_line() => {
                    let message: Value = serde_json::from_str(&line?.ok_or("guest runtime closed before verdict")?)?;
                    trace.record("runtime",&message)?;
                    match message["type"].as_str() {
                        Some("protocol_frame") => {},
                        Some("execute") => {
                            let sim = simulator.clone();
                            backend.spawn(async move { sim.call(message).await });
                        }
                        Some("terminal" | "exit" | "terminal_closed") => {
                            if message["id"].is_null() { continue; }
                            let id = message["id"].as_u64().ok_or("terminal reply id")?;
                            let response = match message["type"].as_str() {
                                Some("terminal") => json!({"stdout":message["stdout"]}),
                                Some("exit") => json!({"stdout":message["stdout"],"exit_status":message["status"]}),
                                _ => json!({"status":"guest_closed"}),
                            };
                            pending.remove(&id).ok_or("unknown terminal reply id")?.send(response).map_err(|_| "terminal receiver dropped")?;
                        }
                        Some("run_result") => break Ok(message),
                        _ => return Err(format!("unknown guest event: {message}").into()),
                    }
                }
                Some(reply) = backend.join_next() => {
                    let response = reply??;
                    wire::send(&mut input,&response).await?;
                }
            }
        }
    }.await;
    // Closing the transport cancels pending tool requests without granting or
    // guessing any protocol step. Already admitted effects are still observed.
    drop(commands);
    drop(input);
    if !verdict.as_ref().is_ok_and(|v| v["status"] == "complete") {
        let _ = child.kill().await;
    }
    let exit = child.wait().await?;
    let mut errors = Vec::new();
    while let Some(reply) = backend.join_next().await {
        if let Err(error) = reply
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })
            .and_then(|v| v)
        {
            errors.push(error.to_string());
        }
    }
    let score = simulator.call(json!({"type":"score"})).await?;
    Ok(
        json!({"protocol":verdict.as_ref().ok(),"error":verdict.err().map(|e|e.to_string()),"runtime_exit_code":exit.code(),"score":score,"backend_errors":errors}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    #[ignore = "requires the built writing runtime, wash module and original simulator environment"]
    async fn original_simulator_overlaps_disjoint_actual_wasi_effects() -> Result<()> {
        std::env::set_current_dir(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()?,
        )?;
        let output = Path::new("output").join(format!(
            "writing-rust-runner-probe-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir_all(output.parent().ok_or("probe output parent")?)?;
        std::fs::create_dir(&output)?;
        let trace = Arc::new(Trace::create(&output.join("events.jsonl"))?);
        let (simulator, fixture) = simulator("2M", 2, &output, trace.clone()).await?;
        let runtime = std::env::var_os("CHOREOFIX_WRITING_RUNTIME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "target/debug/writing-wasi".into());
        crate::evidence::save(
            &output.join("case.json"),
            &json!({
                "task":"2M","depth":2,"model_calls":0,"scored_benchmark":false,
                "kind":"rust_upstream_api_and_existing_wasi_process_probe",
                "runtime_binary_sha256":crate::evidence::digest(&runtime)?,
                "guest_wasm_sha256":crate::evidence::digest(Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"))?,
                "runtime_source_scope":"existing binary; this bridge test does not establish its Hibana source pin"
            }),
        )?;
        let (guest, guest_loop) = guest(
            &runtime,
            Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
            "2M",
            &output,
            trace,
            simulator,
        )?;
        let mut subjects = JoinSet::new();
        for actor in fixture["fixtures"].as_array().ok_or("fixtures")? {
            let actor = actor.clone();
            let guest = guest.clone();
            subjects.spawn(async move {
                let name = actor["agent"].as_str().ok_or("actor")?;
                for call in actor["calls"].as_array().ok_or("calls")? {
                    let arguments = serde_json::to_string(&call["arguments"])?;
                    let quote = format!("'{}'", arguments.replace('\'', "'\"'\"'"));
                    let response = guest
                        .command(
                            name,
                            format!(
                                "printf '%s' {quote} > /service/{}; cat /service/result",
                                call["tool"].as_str().ok_or("tool")?
                            ),
                        )
                        .await?;
                    if !response["stdout"]
                        .as_str()
                        .ok_or("stdout")?
                        .contains("\"success\":true")
                    {
                        return Err(format!("probe operation failed: {response}").into());
                    }
                }
                let exit = guest.command(name, "exit 0".into()).await?;
                if exit["exit_status"] != 0 {
                    return Err("probe guest exit failed".into());
                }
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            });
        }
        drop(guest);
        let participants = async {
            while let Some(result) = subjects.join_next().await {
                result??;
            }
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        };
        let (result, actors) = tokio::time::timeout(std::time::Duration::from_secs(120), async {
            tokio::join!(guest_loop, participants)
        })
        .await?;
        actors?;
        let result = result?;
        if result["protocol"]["status"] != "complete" || result["score"]["complete"] != true {
            return Err(format!("probe failed: {result}").into());
        }
        let audit = crate::writing_audit::audit(&output.join("events.jsonl"))?;
        assert_eq!(audit["max_disjoint_writes"], 2);
        crate::evidence::save(
            &output.join("probe.json"),
            &json!({"kind":"deterministic_rust_process_transport_probe","model_calls":0,"scored_benchmark":false,"audit":audit,"result":result}),
        )?;
        println!("{}", output.display());
        Ok(())
    }
}
