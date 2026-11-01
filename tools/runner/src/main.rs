mod api_audit;
#[cfg(test)]
mod api_boundary;
mod audit;
mod cicd_audit;
#[cfg(test)]
mod cicd_boundary;
mod codebase_audit;
#[cfg(test)]
mod codebase_boundary;
mod evidence;
mod model;
#[cfg(test)]
mod model_boundary;
mod oracle;
#[path = "../../../src/tasks/philosophers/choreofs/identity.rs"]
mod philosopher_identity;
mod philosophers_audit;
#[cfg(test)]
mod philosophers_boundary;
mod process;
mod projection;
mod qualification;
mod review_audit;
#[cfg(test)]
mod review_boundary;
mod snapshot;
mod wire;
mod writing_audit;
#[cfg(test)]
mod writing_boundary;

use evidence::{Result, Trace, save};
use hibana_runtime::{host::deadline, race};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

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
    let family = task.trim_end_matches(['E', 'M', 'H']).parse::<u8>()?;
    let dining = family == 9;
    let coding = family == 1;
    let api = task == "8E";
    let cicd = task == "16E";
    let review = task == "6E";
    if !(dining || coding || api || cicd || review || family == 2) {
        return Err(format!("task {task} has no model prompt implementation").into());
    }
    let data = if dining {
        json!({"participant":agent,"resources":actor["forks"],"philosopher":agent})
    } else if coding {
        json!({"participant":agent,"resources":actor["modules"],
        "feature_name":actor["calls"][0]["arguments"]["feature_name"]})
    } else if review {
        json!({"participant":agent,"resources":actor["resources"],"assignments":actor["calls"]})
    } else if api || cicd {
        json!({"participant":agent,"resources":actor["resources"],
        "arguments":actor["calls"][0]["arguments"]})
    } else {
        json!({"participant":agent,"resources":actor["sections"],
        "topic":actor["calls"][0]["arguments"]["topic"],
        "section_name":actor["calls"][1]["arguments"]["section_name"]})
    };
    let objects = if review {
        match agent {
            "DEVELOPER_A" | "DEVELOPER_B" => "/service/implement_feature and /service/merge_code",
            "REVIEWER" => {
                "/service/code_review/<pr_id>, using an assigned pr_id as the final path component"
            }
            _ => return Err("unknown review participant".into()),
        }
    } else if cicd {
        match agent {
            "DEVELOPER" => "/service/compile_and_test",
            "TESTER" => "/service/run_integration_tests",
            "DEPLOYER" => "/service/deploy_application",
            _ => return Err("unknown CI/CD participant".into()),
        }
    } else if dining {
        "/service/think and /service/eat"
    } else if coding {
        "/service/design_feature, /service/implement_code, /service/commit_changes and /service/run_local_tests"
    } else if api {
        match agent {
            "DB_ADMIN" => "/service/migrate_schema",
            "BACKEND_DEV" => "/service/implement_backend",
            "TESTER" => "/service/run_integration_tests",
            _ => return Err("unknown API task participant".into()),
        }
    } else {
        "/service/read_literature, /service/draft_content, /service/write_to_document and /service/review_own_work"
    };
    let coordination = if review {
        "Hibana delegates the repository Endpoint during implementation and merging; each actual request and result travels through it before it returns. Code review does not hold the repository. You do not call resource acquisition, release or notification tools. A developer's implementation response includes the actual review under coordination.code_review. If review fails, reconsider and implement again; merge your assigned branch after approval. The reviewer chooses the order of the assigned features and may switch to the other feature after a rejection while the developer revises. Review both features until each passes. Reviewing one feature can overlap independent work on the other feature. Read every actual result and finish after your own assigned work is complete."
    } else if cicd {
        "Hibana passes the build-server or deployment-environment Endpoint to your WASI participant. Your actual request and result travel through that Endpoint before it returns. The developer's build precedes integration testing, which precedes deployment. Hibana sends build-complete, tests-passed and both deployment-complete notifications. You do not call acquire/release or notification tools. Read your actual result, then finish. This original task has no retry loop or injected domain failures."
    } else if dining {
        "Hibana passes both fork endpoints to your guest. Eating requires the requests sent through both endpoints, and both endpoints return afterwards. You do not call acquire/release tools. Your operations always apply to your participant. The philosopher string is passed to the original simulator. Both effects run once, then finish."
    } else if task == "1H" {
        "Hibana passes both module endpoints to your guest. Committing requires requests through both endpoints. You do not call acquire/release tools. This replaces the original sequential acquisition procedure. The feature and module pair must match your assignment. Module endpoints return before testing so that independent developers can continue. Read each actual test result. If it fails, reconsider your implementation, then implement, commit and test again. Finish after your tests pass."
    } else if coding {
        "Hibana passes both module endpoints to your guest. Committing requires the requests sent through both endpoints. You do not call acquire/release tools. This replaces the original sequential acquisition procedure. The feature and module pair must match your assignment. For this task the test outcome is terminal, including failure. Do not add retries. Module endpoints are returned before testing so that another developer can commit while your test runs."
    } else if api {
        "Hibana passes the database or test-environment Endpoint to the WASI participant. The request and actual result travel through that Endpoint before it returns. Migration completion precedes backend work, and backend completion precedes integration testing. You do not call acquire/release or notification tools. Use your assigned arguments. DB_ADMIN and BACKEND_DEV must also read the read-only /service/outcome object, which contains the original test result. That read may wait until the test finishes while other participants continue. A failed integration test is terminal in the task description and simulator. Report its actual result and finish, without adding a revision or retry."
    } else {
        "Hibana passes both section Endpoints to your WASI guest in parallel. Your actual write request travels through both Endpoints; the actual result comes back through both before they return. You do not call acquire/release tools. This deliberately replaces the original sequential acquisition procedure. The target must match your assigned topic, section or pair of resource names. The review result is terminal, so do not add retries to a failed review."
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
A script has at most 4096 UTF-8 bytes. Each participant has its own persistent
guest and private files.

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

fn trial_error(outcomes: &[Value]) -> Option<String> {
    outcomes
        .iter()
        .find_map(|outcome| match outcome["kind"].as_str() {
            Some("model_participant") => {
                let result = &outcome["result"];
                (result["status"] != "completed" || !result["error"].is_null()).then(|| {
                    format!(
                        "model participant {}: status={}, error={}",
                        result["participant"], result["status"], result["error"]
                    )
                })
            }
            Some("guest_runtime") => {
                let result = &outcome["result"];
                (!result["error"].is_null()
                    || result["runtime_exit_code"] != 0
                    || result["protocol"]["status"] != "complete"
                    || result["backend_errors"] != json!([]))
                .then(|| format!("guest runtime failed: {result}"))
            }
            _ => Some(format!("trial job failed: {outcome}")),
        })
}

#[hibana_runtime::scope]
async fn run(task: &str, depth: u8, output: &Path) -> Result<Value> {
    if ![
        "1E", "1M", "1H", "2E", "2M", "2H", "6E", "8E", "9E", "9M", "9H", "16E",
    ]
    .contains(&task)
        || depth > 2
    {
        return Err("expected a migrated task and depth 0..2".into());
    }
    fs::create_dir_all(output.parent().ok_or("output parent")?)?;
    fs::create_dir(output)?;
    let mut versions = snapshot::freeze(task, output)?;
    versions["model_connection"] = model::identity()?;
    save(&output.join("versions.json"), &versions)?;
    let config: Value = serde_json::from_slice(&fs::read("experiment.json")?)?;
    let seconds = config["run_timeout_seconds"]
        .as_u64()
        .ok_or("run timeout")?;
    let budget = config["participant_tool_call_limit"]
        .as_u64()
        .ok_or("tool budget")?;
    let trace = Trace::create(&output.join("events.jsonl"))?;
    let started = Instant::now();
    let (simulator, fixture) = process::simulator(task, depth, output, trace.clone()).await?;
    let (guest, guest_loop) = process::guest(
        &output.join("runtime").canonicalize()?,
        &output.join("wash.wasm").canonicalize()?,
        task,
        output,
        trace,
        simulator,
    )?;
    let mut people = Vec::new();
    for actor in fixture["fixtures"].as_array().ok_or("actor fixtures")? {
        let agent = actor["agent"].as_str().ok_or("actor name")?.to_owned();
        let instructions = prompt(Path::new("."), task, actor)?;
        fs::write(output.join(format!("{agent}-prompt.txt")), &instructions)?;
        people.push(model::Participant {
            output: output.join(&agent),
            agent,
            instructions,
        });
    }
    let finished: Result<()> = race(
        async {
            async {
                let result = match guest_loop.await {
                    Ok(value) => json!({"kind":"guest_runtime","result":value}),
                    Err(error) => json!({"status":"harness_error","error":error.to_string()}),
                };
                save(&output.join("guest-outcome.json"), &result)?;
            };
            async {
                let result = match model::run(
                    people,
                    model::Limits {
                        timeout: Duration::from_secs(seconds),
                        tool_calls: budget,
                    },
                    &guest,
                )
                .await
                {
                    Ok(results) => json!(
                        results
                            .into_iter()
                            .map(|result| json!({"kind":"model_participant","result":result}))
                            .collect::<Vec<_>>()
                    ),
                    Err(error) => json!([{"status":"harness_error","error":error.to_string()}]),
                };
                save(&output.join("model-outcomes.json"), &result)?;
            };
            Ok(())
        },
        deadline(Duration::from_secs(seconds + 20)),
    )
    .await;
    // These files are experiment reports, including partial reports after a
    // timeout. They never decide which participant or Endpoint may progress.
    let mut outcomes = Vec::new();
    match fs::read(output.join("guest-outcome.json")) {
        Ok(bytes) => outcomes.push(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    match fs::read(output.join("model-outcomes.json")) {
        Ok(bytes) => outcomes.extend(serde_json::from_slice::<Vec<Value>>(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    save(&output.join("outcomes.json"), &json!(outcomes))?;
    let error = match finished {
        Ok(()) => trial_error(&outcomes),
        Err(error) => Some(error.to_string()),
    };
    let result = json!({"task":task,"depth":depth,"kind":"unscored_direct_wasi_model_preflight",
        "scored_benchmark":false,"error":error,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "versions":versions,"outcomes":outcomes,"events_sha256":evidence::digest(&output.join("events.jsonl"))?});
    save(&output.join("result.json"), &result)?;
    Ok(result)
}

#[hibana_runtime::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, output] if mode == "--qualify-wasi" => println!(
            "{}",
            serde_json::to_string_pretty(&qualification::run(Path::new(output))?)?
        ),
        [mode, events] if mode == "--audit-philosophers" => println!("{}", serde_json::to_string_pretty(&philosophers_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-philosophers-mutants" => println!("{}", serde_json::to_string_pretty(&philosophers_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-api" => println!("{}", serde_json::to_string_pretty(&api_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-api-mutants" => println!("{}", serde_json::to_string_pretty(&api_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-review" => println!("{}", serde_json::to_string_pretty(&review_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-review-mutants" => println!("{}", serde_json::to_string_pretty(&review_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-writing" => println!("{}", serde_json::to_string_pretty(&writing_audit::audit(Path::new(events))?)?),
        [mode, events, output] if mode == "--check-writing-mutants" => println!("{}", serde_json::to_string_pretty(&writing_audit::check_mutants(Path::new(events), Path::new(output))?)?),
        [mode, events] if mode == "--audit-cicd" => println!(
            "{}",
            serde_json::to_string_pretty(&cicd_audit::audit(Path::new(events))?)?
        ),
        [mode, events, output] if mode == "--check-cicd-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&cicd_audit::check_mutants(Path::new(events), Path::new(output))?)?
        ),
        [mode, events] if mode == "--audit" => println!(
            "{}",
            serde_json::to_string_pretty(&codebase_audit::audit(Path::new(events))?)?
        ),
        [mode, events, output] if mode == "--check-protocol-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&codebase_audit::check_mutants(Path::new(events), Path::new(output))?)?
        ),
        [mode, input, output] if mode == "--check-run-mutants" => println!(
            "{}",
            serde_json::to_string_pretty(&audit::check_mutants(
                Path::new(input),
                Path::new(output)
            )?)?
        ),
        [mode, output] if mode == "--audit-run" => println!(
            "{}",
            serde_json::to_string_pretty(&audit::run(Path::new(output))?)?
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
            if let Some(error) = result["error"].as_str() {
                return Err(error.to_owned().into());
            }
        }
        _ => return Err(concat!(
            "usage: choreofix-experiments 1E|1M|1H|2E|2M|2H|6E|8E|9E|9M|9H|16E DEPTH OUTPUT\n",
            "       --audit-api|--audit-review|--audit-writing|--audit-cicd|--audit-philosophers|--audit EVENTS\n",
            "       --check-api-mutants|--check-review-mutants|--check-writing-mutants|--check-cicd-mutants|--check-philosophers-mutants|--check-protocol-mutants EVENTS NEW_OUTPUT\n",
            "       --audit-run RUN\n",
            "       --qualify-wasi NEW_OUTPUT\n",
            "       --check-run-mutants RUN NEW_OUTPUT"
        ).into()),
    }
    Ok(())
}
