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
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
// A planned list of test inputs is not a second runtime permission mechanism.
fn stimuli(fixture: &Value) -> Result<VecDeque<(String, String)>> {
    let mut commands = VecDeque::new();
    commands.push_back(("echo '{}' > /service/eat".into(), "wash:".into()));
    commands.push_back((
        "x=3; expr $x + 4 > /work/note; cat /work/note".into(),
        "7\n".into(),
    ));
    for call in fixture["calls"].as_array().ok_or("missing calls")? {
        let tool = call["tool"].as_str().ok_or("missing tool")?;
        commands.push_back((
            format!("echo '{{}}' > /service/{tool}; cat /service/result"),
            "invalid_arguments".into(),
        ));
        let key = call["arguments"]
            .as_object()
            .ok_or("argument object")?
            .keys()
            .next()
            .ok_or("argument key")?;
        let wrong = serde_json::to_string(&json!({key: 42}))?;
        commands.push_back((
            format!(
                "echo {} > /service/{tool}; cat /service/result",
                quote(&wrong)
            ),
            "invalid_arguments".into(),
        ));
        // This string is ignored by the original simulator. Caller identity is
        // the projected owner, so another name must not retarget the effect.
        let arguments = serde_json::to_string(&json!({"philosopher":"ANOTHER_DISPLAY_NAME"}))?;
        let padded = format!("{}{}", " ".repeat(512), arguments);
        commands.push_back((
            format!(
                "printf '%s' {} > /service/{tool}; cat /service/result",
                quote(&padded)
            ),
            format!("\"tool_name\":\"{tool}\""),
        ));
        commands.push_back((
            format!("echo {} > /service/{tool}", quote(&arguments)),
            "wash:".into(),
        ));
    }
    commands.push_back(("cat /work/note".into(), "7\n".into()));
    commands.push_back(("exit 0".into(), String::new()));
    Ok(commands)
}

fn run(task: &str, output: &Path) -> Result<()> {
    fs::create_dir(output)?;
    let mut evidence = File::create(output.join("events.jsonl"))?;
    let started = Instant::now();
    let (sender, receiver) = mpsc::channel();
    let mut simulator_command = Command::new(".venv/bin/python");
    simulator_command.args(["tools/original_simulator.py", task]);
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
    let fixtures: Value = serde_json::from_str(&line?)?;
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
    let runtime_path = std::env::current_exe()?.with_file_name("philosophers-wasi");
    let mut runtime = Process(
        Command::new(runtime_path)
            .args([
                "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
                task,
            ])
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
            ("runtime", Some("protocol_frame")) => {}
            ("runtime", Some("execute")) => send(sim_input, &message)?,
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

#[path = "philosophers-boundary/audit.rs"]
mod evidence;
use evidence::audit;

fn check_mutants(path: &Path, output: &Path) -> Result<Value> {
    audit(path)?;
    fs::create_dir(output)?;
    let records = BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(serde_json::from_str::<Value>(&line?)?))
        .collect::<Result<Vec<_>>>()?;
    let mut checked = Vec::new();
    for mutation in [
        "missing_request",
        "altered_frame",
        "forged_arguments",
        "forged_result",
        "missing_exit",
        "false_score",
        "response_before_second_request",
    ] {
        let mut copy = records.clone();
        let index = copy
            .iter()
            .position(|r| {
                let m = &r["message"];
                match mutation {
                    "missing_request" => {
                        r["direction"] == "runtime"
                            && m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["source"] == 28
                            && m["target"] == 0
                    }
                    "altered_frame" => {
                        r["direction"] == "runtime"
                            && m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                    }
                    "forged_arguments" => m["type"] == "execute",
                    "forged_result" => {
                        r["direction"] == "simulator" && m["type"] == "simulator_result"
                    }
                    "missing_exit" => m["type"] == "exit",
                    "false_score" => m["type"] == "score",
                    "response_before_second_request" => true,
                    _ => false,
                }
            })
            .ok_or("mutation target missing")?;
        match mutation {
            "missing_request" | "missing_exit" => {
                copy.remove(index);
            }
            "altered_frame" => copy[index]["message"]["source"] = json!(254),
            "forged_arguments" => copy[index]["message"]["arguments"] = json!({"philosopher":42}),
            "forged_result" => copy[index]["message"]["result"]["data"] = json!({"forged":true}),
            "false_score" => {
                copy[index]["message"]["complete"] = json!(
                    !copy[index]["message"]["complete"]
                        .as_bool()
                        .ok_or("score completion")?
                )
            }
            "response_before_second_request" => {
                use choreofix::tasks::philosophers::choreofs::global::{roles9e, roles9h, roles9m};
                let right = match records[0]["message"]["task"].as_str() {
                    Some("9E") => roles9e::FORK_C_RESOURCE,
                    Some("9M") => roles9m::FORK_E_RESOURCE,
                    Some("9H") => roles9h::FORK_G_RESOURCE,
                    _ => return Err("unknown dining task".into()),
                };
                let request = records
                    .iter()
                    .find(|r| {
                        let m = &r["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "send"
                            && m["source"] == right
                            && m["target"] == 0
                    })
                    .ok_or("actual right-resource request")?["message"]["frame_id"]
                    .clone();
                let moved = copy
                    .iter()
                    .filter(|r| {
                        r["message"]["type"] == "protocol_frame"
                            && r["message"]["frame_id"] == request
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                copy.retain(|r| {
                    !(r["message"]["type"] == "protocol_frame"
                        && r["message"]["frame_id"] == request)
                });
                let after = copy
                    .iter()
                    .position(|r| {
                        let m = &r["message"];
                        m["type"] == "protocol_frame"
                            && m["action"] == "receive"
                            && m["source"] == 0
                            && m["target"] == roles9e::FORK_A_RESOURCE
                    })
                    .ok_or("actual meal receipt")?
                    + 1;
                copy.splice(after..after, moved);
            }
            _ => unreachable!(),
        }
        let mutant_path = output.join(format!("{mutation}.jsonl"));
        let mut file = File::create(&mutant_path)?;
        for record in &copy {
            send(&mut file, record)?;
        }
        let error = audit(&mutant_path)
            .err()
            .ok_or("audit accepted mutated evidence")?
            .to_string();
        checked.push(json!({"mutation":mutation, "rejected":true, "error":error}));
    }
    let result = json!({"mutants":checked, "original_unchanged":true});
    fs::write(
        output.join("mutants.json"),
        format!("{}\n", serde_json::to_string_pretty(&result)?),
    )?;
    Ok(result)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, path, output] if mode == "--check-mutants" => {
            println!("{}", check_mutants(Path::new(path), Path::new(output))?);
            Ok(())
        }
        [mode, path] if mode == "--audit" => {
            println!("{}", audit(Path::new(path))?);
            Ok(())
        }
        [task, output] if ["9E", "9M", "9H"].contains(&task.split(':').next().unwrap()) => {
            run(task, Path::new(output))
        }
        _ => Err("usage: philosophers-boundary 9E|9M|9H OUTPUT | --audit EVENTS.jsonl".into()),
    }
}
