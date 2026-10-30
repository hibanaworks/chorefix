//! Deterministic external stimuli and persisted-event audit, never task authority.
//! Python is used only to call the unchanged original Python simulator.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
#[path = "../../tools/runner/src/api_audit.rs"]
mod saved_evidence;
use saved_evidence::{audit, check_mutants};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn send(output: &mut impl Write, message: &Value) -> Result<()> {
    serde_json::to_writer(&mut *output, message)?;
    writeln!(output)?;
    output.flush()?;
    Ok(())
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
// Fixed external stimuli, not task admission or progress authority.
fn stimuli(fixture: &Value) -> Result<VecDeque<(String, String)>> {
    let mut commands = VecDeque::new();
    let call = &fixture["calls"][0];
    let tool = call["tool"].as_str().ok_or("tool")?;
    commands.push_back((
        "echo '{}' > /service/an_unknown_operation".into(),
        "wash:".into(),
    ));
    commands.push_back((
        "x=3; expr $x + 4 > /work/note; cat /work/note".into(),
        "7\n".into(),
    ));
    commands.push_back((
        format!("echo '{{}}' > /service/{tool}; cat /service/result"),
        "invalid_arguments".into(),
    ));
    let key = call["arguments"]
        .as_object()
        .ok_or("arguments")?
        .keys()
        .next()
        .ok_or("key")?;
    commands.push_back((
        format!(
            "echo {} > /service/{tool}; cat /service/result",
            quote(&json!({key:"wrong_target"}).to_string())
        ),
        "invalid_arguments".into(),
    ));
    commands.push_back((
        format!(
            "printf '%s' {} > /service/{tool}; cat /service/result",
            quote(&format!("{}{}", " ".repeat(512), call["arguments"]))
        ),
        format!("\"tool_name\":\"{tool}\""),
    ));
    commands.push_back((
        format!(
            "echo {} > /service/{tool}",
            quote(&call["arguments"].to_string())
        ),
        "wash:".into(),
    ));
    if fixture["agent"] != "TESTER" {
        commands.push_back((
            "cat /service/outcome".into(),
            "\"tool_name\":\"run_integration_tests\"".into(),
        ));
        commands.push_back(("echo wrong > /service/outcome".into(), "wash:".into()));
        commands.push_back((
            "cat /service/outcome".into(),
            "\"tool_name\":\"run_integration_tests\"".into(),
        ));
    }
    commands.push_back(("cat /work/note".into(), "7\n".into()));
    commands.push_back(("exit 0".into(), String::new()));
    Ok(commands)
}
fn run(task: &str, output: &Path) -> Result<()> {
    let depth: u8 = task.parse()?;
    if depth > 2 {
        return Err("invalid depth".into());
    }
    let task = "8E";
    fs::create_dir(output)?;
    let mut evidence = File::create(output.join("events.jsonl"))?;
    let started = Instant::now();
    let (sender, receiver) = mpsc::channel();
    let mut simulator_command = Command::new(".venv/bin/python");
    simulator_command.env("PYTHONHASHSEED", "0").args([
        "tools/original_simulator.py",
        task,
        "boundary",
        &depth.to_string(),
    ]);
    let mut simulator = Process(
        simulator_command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(output.join("simulator-stderr.log"))?)
            .spawn()?,
    );
    let sim_stdout = simulator.0.stdout.take().ok_or("simulator stdout")?;
    let sim_sender = sender.clone();
    std::thread::spawn(move || {
        for line in BufReader::new(sim_stdout).lines() {
            if sim_sender.send(("simulator", line)).is_err() {
                break;
            }
        }
    });
    let (direction, line) = receiver.recv_timeout(Duration::from_secs(10))?;
    let mut fixtures: Value = serde_json::from_str(&line?)?;
    fixtures["require_early_result_open"] = json!(true);
    if fixtures["type"] != "fixtures" {
        return Err("missing simulator fixtures".into());
    }
    send(
        &mut evidence,
        &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "direction":direction, "message":fixtures}),
    )?;
    let mut scripts: BTreeMap<_, _> = fixtures["fixtures"]
        .as_array()
        .ok_or("fixtures array")?
        .iter()
        .map(|f| Ok((f["agent"].as_str().ok_or("agent")?.to_owned(), stimuli(f)?)))
        .collect::<Result<_>>()?;
    let mut runtime = Process(
        Command::new("target/debug/api-wasi")
            .args(["vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(output.join("runtime-stderr.log"))?)
            .spawn()?,
    );
    let runtime_stdout = runtime.0.stdout.take().ok_or("runtime stdout")?;
    std::thread::spawn(move || {
        for line in BufReader::new(runtime_stdout).lines() {
            if sender.send(("runtime", line)).is_err() {
                break;
            }
        }
    });
    let sim_input = simulator.0.stdin.as_mut().ok_or("simulator stdin")?;
    let runtime_input = runtime.0.stdin.as_mut().ok_or("runtime stdin")?;
    let mut expected = BTreeMap::<u64, String>::new();
    let mut command_id = 0u64;
    let mut early_reads = std::collections::BTreeSet::new();
    let mut test_request = None;
    loop {
        if started.elapsed() > Duration::from_secs(120) {
            return Err("boundary timeout".into());
        }
        let (direction, line) = receiver.recv_timeout(Duration::from_secs(30))?;
        let message: Value = serde_json::from_str(&line?)?;
        send(
            &mut evidence,
            &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "direction":direction, "message":message}),
        )?;
        match (direction, message["type"].as_str()) {
            ("runtime", Some("protocol_frame")) => {
                if message["action"] == "send"
                    && [13, 14].into_iter().any(|r| message["source"] == r)
                {
                    early_reads.insert(message["source"].as_u64().ok_or("outcome requester")?);
                }
                if early_reads.len() == 2
                    && let Some(call) = test_request.take()
                {
                    send(sim_input, &call)?;
                }
            }
            ("runtime", Some("execute")) => {
                if message["tool"] == "run_integration_tests" && early_reads.len() != 2 {
                    if test_request.replace(message.clone()).is_some() {
                        return Err("duplicate test stimulus".into());
                    }
                } else {
                    send(sim_input, &message)?;
                }
            }
            ("simulator", Some("simulator_result")) => send(runtime_input, &message)?,
            ("runtime", Some("terminal" | "exit")) => {
                if let Some(id) = message["id"].as_u64() {
                    let needle = expected.remove(&id).ok_or("unknown terminal reply")?;
                    let stdout = message["stdout"]
                        .as_str()
                        .ok_or("missing terminal output")?;
                    let normalized = stdout.replace(' ', "");
                    if !normalized.contains(&needle) {
                        return Err(format!(
                            "unexpected terminal output: expected {needle:?}, got {stdout:?}"
                        )
                        .into());
                    }
                }
                if message["type"] == "exit" {
                    if message["status"] != 0 {
                        return Err("guest exit failed".into());
                    }
                    continue;
                }
                let agent = message["agent"].as_str().ok_or("terminal agent")?;
                let (command, needle) = scripts
                    .get_mut(agent)
                    .ok_or("unknown agent")?
                    .pop_front()
                    .ok_or("extra terminal read")?;
                command_id += 1;
                expected.insert(command_id, needle);
                let input =
                    json!({"type":"command", "id":command_id, "agent":agent, "command":command});
                send(
                    &mut evidence,
                    &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "direction":"to_runtime", "message":input}),
                )?;
                send(runtime_input, &input)?;
            }
            ("runtime", Some("run_result")) => {
                if message["status"] != "complete" {
                    return Err(format!("runtime failed: {message}").into());
                }
                if !expected.is_empty() || scripts.values().any(|s| !s.is_empty()) {
                    return Err("unexecuted shell stimuli".into());
                }
                send(sim_input, &json!({"type":"score"}))?;
            }
            ("simulator", Some("score")) => break,
            _ => return Err(format!("unknown boundary event: {direction} {message}").into()),
        }
    }
    if !runtime.0.wait()?.success() || !simulator.0.wait()?.success() {
        return Err("boundary subprocess failed".into());
    }
    let audit = audit(&output.join("events.jsonl"))?;
    fs::write(
        output.join("audit.json"),
        format!("{}\n", serde_json::to_string_pretty(&audit)?),
    )?;
    println!("{}", serde_json::to_string(&audit)?);
    Ok(())
}

fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    match a.as_slice() {
        [mode, input] if mode == "--audit" => println!("{}", audit(Path::new(input))?),
        [mode, input, output] if mode == "--check-mutants" => {
            println!("{}", check_mutants(Path::new(input), Path::new(output))?)
        }
        [depth, output] => run(depth, Path::new(output))?,
        _ => {
            return Err(
                "usage: api-boundary DEPTH OUTPUT | --audit EVENTS | --check-mutants EVENTS OUTPUT"
                    .into(),
            );
        }
    }
    Ok(())
}
