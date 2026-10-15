mod audit;
mod evidence;
mod model;
mod process;
mod snapshot;
mod wire;

use evidence::{Result, Trace, save};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinSet;

fn prompt(source: &Path, task: &str, actor: &Value) -> Result<String> {
    let agent = actor["agent"].as_str().ok_or("participant name")?;
    let input = source
        .join("vendor/tracefix/benchmark/descriptions")
        .join(task);
    let tools: Value = serde_json::from_slice(&fs::read(input.join("tools.json"))?)?;
    let schemas: Vec<_> = tools
        .as_array()
        .ok_or("original tools")?
        .iter()
        .map(|v| &v["function"])
        .filter(|v| {
            v["agent_ids"]
                .as_array()
                .is_some_and(|a| a.contains(&json!(agent)))
        })
        .collect();
    let dining = task.starts_with('9');
    let data = if dining {
        json!({"participant":agent,"resources":actor["forks"],"philosopher":agent})
    } else {
        json!({"participant":agent,"resources":actor["sections"],
        "topic":actor["calls"][0]["arguments"]["topic"],
        "section_name":actor["calls"][1]["arguments"]["section_name"]})
    };
    let objects = if dining {
        "/service/think and /service/eat"
    } else {
        "/service/read_literature, /service/draft_content, /service/write_to_document and /service/review_own_work"
    };
    let coordination = if dining {
        "Hibana carries both independent fork requests in parallel and waits for both grants before eating. You do not call acquire/release tools. Your philosopher argument must name your own participant. Both effects run once, then finish."
    } else {
        "Hibana carries both independent resource acquisitions in parallel and waits for both grants before the write. You do not call acquire/release tools. This deliberately replaces the original sequential acquisition procedure. The target must match your assigned topic, section or pair of resource names. The review result is terminal, so do not add retries to a failed review."
    };
    Ok(format!(
        r#"You are {agent}. Carry out your work in the original task below.
{description}

Your task input data: {data}
Your original domain operation schemas: {schemas}

You have one tool, shell. It runs your own scripts in the Rust std WASI wash
guest. All file I/O uses Hibana Endpoints and ChoreoFS. There is no host
filesystem or native process execution. Use help to see shell commands and
syntax. You may use variables, conditions, loops, pipes, redirection, expr,
private files and saved scripts. Choose your own commands and calculations.
Private /work files have names you choose, with eight slots of 8192 bytes.
A script has at most 4096 UTF-8 bytes. Each guest starts with 64 KiB and may
request memory growth through the protocol up to 1 MiB of supplied backing.

Your writable domain objects are {objects}.
Write the matching operation's JSON argument object and close it to request
the operation. Shell redirection closes its output after writing.
Read /service/result for the actual response. These objects are not commands.
{coordination}
Independent participants and domain operations run concurrently. A request
waiting for its dependency does not block the other guests.

Ordinary shell errors and invalid argument data can be corrected. Use actual
responses to reconsider your work. When your assigned work is finished,
close your guest with exit 0 before your final response. After an exit_status
reply, do not call shell again. Leaving before doing the work fails the run.
"#,
        description = fs::read_to_string(input.join("description.md"))?,
        schemas = serde_json::to_string(&schemas)?
    ))
}

async fn run(task: &str, depth: u8, output: &Path) -> Result<Value> {
    if !["2E", "2M", "2H", "9E", "9M", "9H"].contains(&task) || depth > 2 {
        return Err("expected writing or dining task and depth 0..2".into());
    }
    fs::create_dir_all(output.parent().ok_or("output parent")?)?;
    fs::create_dir(output)?;
    let versions = snapshot::freeze(task, output)?;
    let config: Value = serde_json::from_slice(&fs::read("experiment.json")?)?;
    let seconds = config["run_timeout_seconds"]
        .as_u64()
        .ok_or("run timeout")?;
    let budget = config["participant_tool_call_limit"]
        .as_u64()
        .ok_or("tool budget")?;
    let trace = Arc::new(Trace::create(&output.join("events.jsonl"))?);
    let started = Instant::now();
    let (simulator, fixture, simulator_loop) =
        process::simulator(task, depth, output, trace.clone()).await?;
    let (guest, guest_loop) = process::guest(
        &output.join("runtime").canonicalize()?,
        &output.join("wash.wasm").canonicalize()?,
        task,
        output,
        trace,
        simulator,
    )?;
    let mut jobs = JoinSet::new();
    jobs.spawn(async move {
        simulator_loop
            .await
            .map(|()| json!({"kind":"simulator_transport","status":"complete"}))
    });
    jobs.spawn(async move {
        guest_loop
            .await
            .map(|value| json!({"kind":"guest_runtime","result":value}))
    });
    for actor in fixture["fixtures"].as_array().ok_or("actor fixtures")? {
        let agent = actor["agent"].as_str().ok_or("actor name")?.to_owned();
        let instructions = prompt(Path::new("."), task, actor)?;
        fs::write(output.join(format!("{agent}-prompt.txt")), &instructions)?;
        let directory = output.join(&agent);
        let guest = guest.clone();
        jobs.spawn(async move {
            let name = agent.clone();
            model::run(
                &agent,
                &instructions,
                &directory,
                Duration::from_secs(seconds),
                budget,
                move |command| {
                    let guest = guest.clone();
                    let name = name.clone();
                    async move { guest.command(&name, command).await }
                },
            )
            .await
            .map(|value| json!({"kind":"model_participant","result":value}))
        });
    }
    drop(guest);
    let mut outcomes = Vec::new();
    let finished = tokio::time::timeout(Duration::from_secs(seconds + 20), async {
        while let Some(result) = jobs.join_next().await {
            outcomes.push(match result {
                Ok(Ok(value)) => value,
                Ok(Err(error)) => json!({"status":"harness_error","error":error.to_string()}),
                Err(error) => json!({"status":"harness_error","error":error.to_string()}),
            });
            save(&output.join("outcomes.json"), &json!(outcomes))?;
        }
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await;
    let error = match finished {
        Ok(Ok(())) => None,
        Ok(Err(e)) => Some(e.to_string()),
        Err(_) => Some("case timeout; incomplete work retained".to_owned()),
    };
    jobs.abort_all();
    while jobs.join_next().await.is_some() {}
    let result = json!({"task":task,"depth":depth,"kind":"unscored_direct_wasi_model_preflight",
        "scored_benchmark":false,"error":error,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "versions":versions,"outcomes":outcomes,"events_sha256":evidence::digest(&output.join("events.jsonl"))?});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, input, output, verifier] if mode == "--check-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&audit::check_mutants(
                Path::new(input),
                Path::new(output),
                Path::new(verifier)
            )?)?
        ),
        [mode, output, verifier] if mode == "--audit" => println!(
            "{}",
            serde_json::to_string_pretty(&audit::run(Path::new(output), Path::new(verifier))?)?
        ),
        [task, depth, output] => {
            let result = match run(task, depth.parse()?, Path::new(output)).await {
                Ok(result) => result,
                Err(error) => {
                    let directory = Path::new(output);
                    if directory.exists() {
                        save(
                            &directory.join("harness-error.json"),
                            &json!({"kind":"harness_error","error":error.to_string(),"scored_benchmark":false}),
                        )?;
                    }
                    return Err(error);
                }
            };
            println!("{}", serde_json::to_string(&result)?);
        }
        _ => return Err("usage: choreofix-experiments 2E|2M|2H|9E|9M|9H DEPTH OUTPUT | --audit RUN VERIFIER | --check-mutants RUN NEW_OUTPUT VERIFIER".into()),
    }
    Ok(())
}
