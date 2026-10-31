//! One I/O thread owns the original environment and starts its asynchronous calls.
use crate::evidence::{Result, Trace};
use pyo3::{
    prelude::*,
    types::{PyCFunction, PyDict, PyModule, PyType},
};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Instant};
use tokio::sync::mpsc;

type Request = (Value, mpsc::Sender<Result<Value>>);

pub fn interpreter_identity() -> Result<Value> {
    if std::env::var("PYTHONHASHSEED").as_deref() != Ok("0") {
        return Err("PYTHONHASHSEED=0 is required before interpreter startup".into());
    }
    Python::initialize();
    Python::attach(|py| -> Result<Value> {
        let sys = PyModule::import(py, "sys")?;
        Ok(
            json!({"version":sys.getattr("version")?.extract::<String>()?,
            "implementation":sys.getattr("implementation")?.getattr("name")?.extract::<String>()?,
            "base_prefix":sys.getattr("base_prefix")?.extract::<String>()?,
            "hash_seed":0,"adapter":"Rust PyO3 API calls, no evaluated snippets"}),
        )
    })
}

struct EventLoop {
    object: Py<PyAny>,
    worker: Option<std::thread::JoinHandle<std::result::Result<(), String>>>,
}

impl EventLoop {
    fn new() -> Result<Self> {
        let object = Python::attach(|py| -> PyResult<_> {
            Ok(PyModule::import(py, "asyncio")?
                .call_method0("new_event_loop")?
                .unbind())
        })?;
        let worker_loop = Python::attach(|py| object.clone_ref(py));
        let worker = std::thread::Builder::new()
            .name("upstream-environment".into())
            .spawn(move || {
                Python::attach(|py| -> PyResult<()> {
                    PyModule::import(py, "asyncio")?
                        .call_method1("set_event_loop", (worker_loop.bind(py),))?;
                    worker_loop.bind(py).call_method0("run_forever")?;
                    worker_loop.bind(py).call_method0("close")?;
                    Ok(())
                })
                .map_err(|e| e.to_string())
            })?;
        Ok(Self {
            object,
            worker: Some(worker),
        })
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        let stopped = Python::attach(|py| -> PyResult<()> {
            let event_loop = self.object.bind(py);
            event_loop.call_method1("call_soon_threadsafe", (event_loop.getattr("stop")?,))?;
            Ok(())
        });
        if let Err(error) = stopped {
            eprintln!("upstream loop shutdown: {error}");
        }
        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => eprintln!("upstream loop: {error}"),
                Err(_) => eprintln!("upstream loop panicked"),
            }
        }
    }
}

fn json_value(value: &Bound<'_, PyAny>) -> Result<Value> {
    let encoded: String = PyModule::import(value.py(), "json")?
        .call_method1("dumps", (value,))?
        .extract()?;
    Ok(serde_json::from_str(&encoded)?)
}

struct Environment {
    simulator: Py<PyAny>,
    registry: Py<PyAny>,
    event_loop: EventLoop,
    trace: Trace,
    started: Instant,
}

#[derive(Clone)]
pub struct Simulator(mpsc::UnboundedSender<(Value, mpsc::Sender<Result<Value>>)>);

impl Simulator {
    pub fn new(
        task: &str,
        depth: u8,
        delay: f64,
        trace: Trace,
        failures: &[(&str, u32)],
    ) -> Result<(Self, Value)> {
        if depth > 2
            || !delay.is_finite()
            || delay < 0.0
            || std::env::var("PYTHONHASHSEED").as_deref() != Ok("0")
        {
            return Err("expected depth 0..2, a finite delay and PYTHONHASHSEED=0 before interpreter startup".into());
        }
        choreofix_inputs::verify_benchmark(Path::new("."))?;
        let interpreter = interpreter_identity()?;
        let source = Path::new("vendor/tracefix").canonicalize()?;
        let metadata: Value = serde_json::from_slice(&fs::read(
            source
                .join("benchmark/descriptions")
                .join(task)
                .join("metadata.json"),
        )?)?;
        let (simulator, registry, inputs) = Python::attach(|py| -> Result<_> {
            let sys = PyModule::import(py, "sys")?;
            let version = sys.getattr("version_info")?;
            let major: u8 = version.get_item(0)?.extract()?;
            let minor: u8 = version.get_item(1)?.extract()?;
            let packages = Path::new(".venv/lib")
                .join(format!("python{major}.{minor}/site-packages"))
                .canonicalize()?;
            PyModule::import(py, "site")?
                .call_method1("addsitedir", (packages.to_str().ok_or("site path UTF-8")?,))?;
            sys.getattr("path")?
                .call_method1("insert", (0, source.to_str().ok_or("upstream path UTF-8")?))?;
            let name = format!("benchmark.environments.{task}.sim");
            let module = PyModule::import(py, &name)?;
            let actual = std::path::PathBuf::from(module.getattr("__file__")?.extract::<String>()?)
                .canonicalize()?;
            if actual
                != source
                    .join("benchmark/environments")
                    .join(task)
                    .join("sim.py")
            {
                return Err("a different upstream environment was imported".into());
            }
            let base = PyModule::import(py, "benchmark.tools.sim_base")?
                .getattr("SimContext")?
                .cast_into::<PyType>()
                .map_err(PyErr::from)?;
            let mut classes = Vec::new();
            for (_, value) in module.dict().iter() {
                if let Ok(class) = value.cast::<PyType>()
                    && class.getattr("__module__")?.extract::<String>()? == name
                    && class.is_subclass(&base)?
                {
                    classes.push(class.clone());
                }
            }
            if classes.len() != 1 {
                return Err("expected one original environment class".into());
            }
            let sim = classes[0].call0()?;
            sim.setattr("_seed", 0)?;
            sim.setattr("_delay_multiplier", delay)?;
            sim.call_method1("set_scenario_depth", (depth,))?;
            // Unscored recovery trials inject failures through the upstream API.
            // Their evidence records every override separately from scored runs.
            for (tool, call_index) in failures {
                let arguments = PyDict::new(py);
                arguments.set_item("tool_name", tool)?;
                arguments.set_item("call_index", call_index)?;
                let failure = PyModule::import(py, "benchmark.tools.sim_base")?
                    .getattr("FailureSpec")?
                    .call((), Some(&arguments))?;
                sim.getattr("_failure_specs")?
                    .call_method1("append", (failure,))?;
            }
            let kwargs = PyDict::new(py);
            kwargs.set_item("sim", &sim)?;
            let registry = PyModule::import(py, "benchmark.tools")?.call_method(
                "load_tools",
                (task,),
                Some(&kwargs),
            )?;
            let dataclasses = PyModule::import(py, "dataclasses")?;
            let mut inputs = serde_json::Map::new();
            for (key, value) in module.dict().iter() {
                let key: String = key.extract()?;
                if key.ends_with("_STEPS") {
                    let entries: Result<Vec<_>> = value
                        .try_iter()?
                        .map(|entry| json_value(&dataclasses.call_method1("asdict", (entry?,))?))
                        .collect();
                    inputs.insert(key, json!(entries?));
                } else if key == "_FORKS" {
                    inputs.insert(key, json_value(&value)?);
                }
            }
            Ok((sim.unbind(), registry.unbind(), inputs))
        })?;
        let mut fixture = fixture(task, &metadata, &json!(inputs))?;
        fixture["delay_multiplier"] = json!(delay);
        fixture["depth"] = json!(depth);
        fixture["seed"] = json!(0);
        fixture["interpreter"] = interpreter;
        fixture["failure_overrides"] = json!(failures);
        fixture["terminal_test_failure"] =
            json!(task != "1H" && failures.contains(&("run_local_tests", 0)));
        fixture["terminal_review_failure"] =
            json!(task.starts_with('2') && failures.contains(&("review_own_work", 0)));
        trace.record("simulator", &fixture)?;
        let event_loop = EventLoop::new()?;
        let environment = Environment {
            simulator,
            registry,
            event_loop,
            trace,
            started: Instant::now(),
        };
        let (send, mut receive) = mpsc::unbounded_channel::<Request>();
        std::thread::Builder::new()
            .name("original-environment".into())
            .spawn(move || {
                // Test-only I/O withholding: requests have already crossed the
                // real guest and Endpoint path. The gate cannot grant a step.
                #[cfg(test)]
                let mut commit_gate: Option<(
                    std::collections::BTreeSet<String>,
                    Vec<Request>,
                )> = None;
                while let Some((message, reply)) = receive.blocking_recv() {
                    #[cfg(test)]
                    {
                        if message["type"] == "hold_disjoint_commits" {
                            let agents: Option<std::collections::BTreeSet<String>> =
                                message["agents"].as_array().and_then(|values| {
                                    values
                                        .iter()
                                        .map(|value| value.as_str().map(str::to_owned))
                                        .collect()
                                });
                            let configured = match agents {
                                Some(agents)
                                    if commit_gate.is_none()
                                        && agents.len() > 1
                                        && message["agents"].as_array().map(Vec::len)
                                            == Some(agents.len()) =>
                                {
                                    environment
                                        .trace
                                        .record("test_control", &message)
                                        .map(|()| {
                                            commit_gate = Some((agents, Vec::new()));
                                            json!({"status":"configured"})
                                        })
                                }
                                _ => Err("invalid or duplicate test commit gate".into()),
                            };
                            let _ = reply.try_send(configured);
                            continue;
                        }
                        if message["type"] == "execute"
                            && message["tool"] == "commit_changes"
                            && let Some((agents, held)) = &mut commit_gate
                            && message["agent"]
                                .as_str()
                                .is_some_and(|a| agents.contains(a))
                        {
                            if held
                                .iter()
                                .any(|(prior, _)| prior["agent"] == message["agent"])
                            {
                                let _ = reply.try_send(Err("duplicate held commit".into()));
                                continue;
                            }
                            if let Err(error) = environment.trace.record(
                                "test_control",
                                &json!({"type":"commit_held","request":message}),
                            ) {
                                let _ = reply.try_send(Err(error));
                                continue;
                            }
                            held.push((message, reply));
                            if held.len() == agents.len() {
                                let (agents, held) = commit_gate.take().expect("live test gate");
                                if let Err(error) = environment.trace.record(
                                    "test_control",
                                    &json!({"type":"commits_released","agents":agents}),
                                ) {
                                    for (_, reply) in held {
                                        let _ = reply.try_send(Err(error.to_string().into()));
                                    }
                                    continue;
                                }
                                for (message, reply) in held {
                                    environment.serve(message, reply);
                                }
                            }
                            continue;
                        }
                    }
                    environment.serve(message, reply);
                }
            })?;
        Ok((Self(send), fixture))
    }

    pub async fn call(&self, message: Value) -> Result<Value> {
        let (send, mut receive) = mpsc::channel(1);
        self.0
            .send((message, send))
            .map_err(|_| "upstream environment closed")?;
        receive
            .recv()
            .await
            .ok_or("upstream reply direction closed")?
    }

    #[cfg(test)]
    pub async fn observed_events(&self) -> Result<Vec<Value>> {
        let result = self.call(json!({"type":"observed_events"})).await?;
        Ok(result.as_array().ok_or("upstream event list")?.clone())
    }
}

impl Environment {
    fn serve(&self, message: Value, reply: mpsc::Sender<Result<Value>>) {
        let result = match message["type"].as_str() {
            Some("execute") => match self.execute(message, reply.clone()) {
                Ok(()) => return,
                Err(error) => Err(error),
            },
            Some("score") => self.score(),
            #[cfg(test)]
            Some("observed_events") => self.observed_events().map(Value::Array),
            _ => Err("unknown upstream request".into()),
        };
        let _ = reply.try_send(result);
    }

    fn execute(&self, message: Value, reply: mpsc::Sender<Result<Value>>) -> Result<()> {
        let tool = message["tool"].as_str().ok_or("upstream tool name")?;
        let agent = message["agent"].as_str().ok_or("upstream agent")?;
        let arguments = serde_json::to_string(&message["arguments"])?;
        let started = u64::try_from(self.started.elapsed().as_nanos())?;
        self.trace.record("domain_start", &message)?;
        let trace = self.trace.clone();
        let clock = self.started;
        let id = message["id"].clone();
        let tool = tool.to_owned();
        let agent = agent.to_owned();
        Python::attach(|py| -> Result<()> {
            let arguments = PyModule::import(py, "json")?
                .call_method1("loads", (arguments,))?
                .cast_into::<PyDict>()
                .map_err(PyErr::from)?;
            arguments.set_item("agent_id", &agent)?;
            let coroutine =
                self.registry
                    .bind(py)
                    .call_method("call", (&tool,), Some(&arguments))?;
            let future = PyModule::import(py, "asyncio")?.call_method1(
                "run_coroutine_threadsafe",
                (coroutine, self.event_loop.object.bind(py)),
            )?;
            let callback = PyCFunction::new_closure(
                py,
                None,
                None,
                move |args, _kwargs| -> PyResult<()> {
                    let result = (|| -> Result<Value> {
                        let result = args
                            .get_item(0)?
                            .call_method0("result")?
                            .call_method0("to_dict")?;
                        let result = json_value(&result)?;
                        trace.record(
                            "domain_end",
                            &json!({"id":id,"agent":agent,"tool":tool,"result":result}),
                        )?;
                        let response = json!({"type":"simulator_result", "id":id,"result":result,
                            "started_ns":started,"finished_ns":u64::try_from(clock.elapsed().as_nanos())?,
                            "clock":"runner monotonic elapsed"});
                        trace.record("simulator", &response)?;
                        Ok(response)
                    })();
                    match reply.try_send(result) {
                        Ok(()) | Err(mpsc::error::TrySendError::Closed(_)) => Ok(()),
                        Err(mpsc::error::TrySendError::Full(_)) => Err(
                            pyo3::exceptions::PyRuntimeError::new_err("duplicate upstream reply"),
                        ),
                    }
                },
            )?;
            future.call_method1("add_done_callback", (callback,))?;
            Ok(())
        })?;
        Ok(())
    }

    fn score(&self) -> Result<Value> {
        let result = Python::attach(|py| -> Result<Value> {
            let sim = self.simulator.bind(py);
            let dataclasses = PyModule::import(py, "dataclasses")?;
            let events: Result<Vec<_>> = sim
                .getattr("events")?
                .try_iter()?
                .map(|event| json_value(&dataclasses.call_method1("asdict", (event?,))?))
                .collect();
            let violations: Result<Vec<_>> = sim
                .getattr("violations")?
                .try_iter()?
                .map(|event| json_value(&dataclasses.call_method1("asdict", (event?,))?))
                .collect();
            Ok(
                json!({"type":"score", "complete":sim.call_method0("is_complete")?.extract::<bool>()?,
                "events":events?, "violations":violations?, "progress":json_value(&sim.getattr("progress")?)?}),
            )
        })?;
        self.trace.record("simulator", &result)?;
        Ok(result)
    }

    #[cfg(test)]
    pub fn observed_events(&self) -> Result<Vec<Value>> {
        Python::attach(|py| {
            let dataclasses = PyModule::import(py, "dataclasses")?;
            self.simulator
                .bind(py)
                .getattr("events")?
                .try_iter()?
                .map(|event| json_value(&dataclasses.call_method1("asdict", (event?,))?))
                .collect()
        })
    }
}

fn fixture(task: &str, metadata: &Value, inputs: &Value) -> Result<Value> {
    let family = task.trim_end_matches(['E', 'M', 'H']).parse::<u8>()?;
    let mut actors = Vec::new();
    for agent in metadata["agents"].as_array().ok_or("original agent list")? {
        let step = |table: &str, key: &str| -> Result<Value> {
            inputs[table]
                .as_array()
                .ok_or_else(|| format!("missing original {table}"))?
                .iter()
                .find(|step| step["agent_id"] == *agent)
                .map(|step| step[key].clone())
                .filter(|v| !v.is_null())
                .ok_or_else(|| format!("missing original input {table} {key}").into())
        };
        let calls = if family == 9 {
            json!([{"tool":"think","arguments":{"philosopher":agent}}, {"tool":"eat","arguments":{"philosopher":agent}}])
        } else if family == 1 {
            let feature = step("_DESIGN_STEPS", "feature")?;
            let resources = metadata["agent_resources"][agent.as_str().ok_or("agent name")?]
                .as_array()
                .ok_or("agent modules")?
                .iter()
                .map(|v| v.as_str().ok_or("module name"))
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join(", ");
            json!([{"tool":"design_feature","arguments":{"feature_name":feature}},
                {"tool":"implement_code","arguments":{"feature_name":feature}},
                {"tool":"commit_changes","arguments":{"modules":resources}},
                {"tool":"run_local_tests","arguments":{"feature_name":feature}}])
        } else if family == 2 {
            let sections = metadata["agent_resources"][agent.as_str().ok_or("agent name")?]
                .as_array()
                .ok_or("agent sections")?
                .iter()
                .map(|v| v.as_str().ok_or("section name"))
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join(", ");
            let section = step("_DRAFT_STEPS", "section")?;
            json!([{"tool":"read_literature","arguments":{"topic":step("_READ_STEPS","topic")?}},
                {"tool":"draft_content","arguments":{"section_name":section}},
                {"tool":"write_to_document","arguments":{"sections":sections}},
                {"tool":"review_own_work","arguments":{"section_name":section}}])
        } else if task == "6E" {
            if agent == "REVIEWER" {
                Value::Array(inputs["_CODE_REVIEW_STEPS"].as_array().ok_or("original review inputs")?.iter().filter(|step|step["agent_id"]==*agent).map(|step|json!({"tool":"code_review","arguments":{"pr_id":step["target"]}})).collect())
            } else {
                json!([
                    {"tool":"implement_feature","arguments":{"feature_name":step("_IMPLEMENT_STEPS","feature")?}},
                    {"tool":"merge_code","arguments":{"branch":step("_MERGE_STEPS","branch")?}}
                ])
            }
        } else if task == "16E" {
            let (tool, table, owner, key) = match agent.as_str() {
                Some("DEVELOPER") => (
                    "compile_and_test",
                    "_BUILD_STEPS",
                    "developer_id",
                    "component",
                ),
                Some("TESTER") => ("run_integration_tests", "_TEST_STEPS", "agent_id", "suite"),
                Some("DEPLOYER") => ("deploy_application", "_DEPLOY_STEPS", "agent_id", "target"),
                _ => return Err("unknown original CI/CD agent".into()),
            };
            let value = inputs[table]
                .as_array()
                .ok_or("original CI/CD inputs")?
                .iter()
                .find(|step| step[owner] == *agent)
                .ok_or("original CI/CD assignment")?[key]
                .clone();
            if !value.is_string() {
                return Err("original CI/CD argument".into());
            }
            json!([{"tool":tool,"arguments":{key:value}}])
        } else if task == "8E" {
            let (tool, table, field, argument) = match agent.as_str() {
                Some("DB_ADMIN") => (
                    "migrate_schema",
                    "_MIGRATE_SCHEMA_STEPS",
                    "migration_name",
                    "migration_name",
                ),
                Some("BACKEND_DEV") => (
                    "implement_backend",
                    "_IMPLEMENT_BACKEND_STEPS",
                    "component",
                    "component",
                ),
                Some("TESTER") => (
                    "run_integration_tests",
                    "_INTEGRATION_TEST_STEPS",
                    "suite",
                    "suite",
                ),
                _ => return Err("unknown original API agent".into()),
            };
            json!([{"tool":tool,"arguments":{argument:step(table,field)?}}])
        } else {
            json!([])
        };
        actors.push(json!({"agent":agent,"calls":calls,
            "resources":metadata["tool_resource_map"][calls[0]["tool"].as_str().unwrap_or("")],
            "modules":metadata["agent_resources"][agent.as_str().ok_or("agent name")?],
            "sections":metadata["agent_resources"][agent.as_str().ok_or("agent name")?],
            "forks":inputs["_FORKS"][agent.as_str().ok_or("agent name")?]}));
    }
    Ok(
        // These describe the original simulator's normal delay/failure profile.
        // They are never task authority. Actual model calls are verified from
        // participant events, independently of this environment metadata.
        json!({"type":"fixtures","task":task,"fixtures":actors,"inputs":inputs,
        "adapter":"rust_upstream_api","test_delays":{},
        "terminal_review_failure":false,"terminal_test_failure":false}),
    )
}

#[cfg(test)]
mod tests;
