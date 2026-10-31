//! Independent Luna conversations over one App Server connection.
use crate::{
    evidence::{Result, Trace, digest, save},
    process::Guest,
    wire::Wire,
};
use hibana_runtime::{
    host::{Process, deadline},
    race,
};
use serde_json::{Value, json};
use std::{
    fs, io,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

mod driver;
#[cfg(test)]
mod tests;

pub const MODEL: &str = "gpt-6-luna";
pub const EFFORT: &str = "xhigh";
pub const CLI: &str = "/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex";

pub fn identity() -> Result<Value> {
    let version = std::process::Command::new(CLI).arg("--version").output()?;
    if !version.status.success() {
        return Err("model client version query failed".into());
    }
    let version = String::from_utf8(version.stdout)?.trim().to_owned();
    if version.is_empty() {
        return Err("model client returned an empty version".into());
    }
    Ok(json!({
        "model":MODEL,"effort":EFFORT,
        "client_version":version,
        "client_sha256":digest(std::path::Path::new(CLI))?,
    }))
}

pub struct Participant {
    pub agent: String,
    pub instructions: String,
    pub output: PathBuf,
}

pub struct Limits {
    pub timeout: Duration,
    pub tool_calls: u64,
}

#[derive(Default)]
struct Observed {
    thread_id: Value,
    returned_model: Value,
    returned_effort: Value,
    usage: Value,
    tool_calls: u64,
}

struct Conversation {
    participant: Participant,
    trace: Trace,
    started: Instant,
    observed: Observed,
}

impl Conversation {
    fn finish(&self, turn: &Value) -> Result<Value> {
        let result = json!({
            "participant":self.participant.agent,"status":turn["status"],"error":turn["error"],
            "thread_id":self.observed.thread_id,"requested_model":MODEL,
            "authentication":"chatgpt","model_connection":"codex_app_server",
            "returned_model":self.observed.returned_model,"requested_effort":EFFORT,
            "returned_effort":self.observed.returned_effort,"usage":self.observed.usage,
            "tool_calls":self.observed.tool_calls,"elapsed_seconds":self.started.elapsed().as_secs_f64(),
            "raw_events_sha256":digest(&self.participant.output.join("events.jsonl"))?
        });
        save(&self.participant.output.join("participant.json"), &result)?;
        Ok(result)
    }
}

pub async fn run(people: Vec<Participant>, limits: Limits, guest: &Guest) -> Result<Vec<Value>> {
    let parent = people
        .first()
        .ok_or("empty model group")?
        .output
        .parent()
        .ok_or("model output parent")?
        .to_owned();
    if people
        .iter()
        .any(|p| p.output.parent() != Some(parent.as_path()))
    {
        return Err("model outputs must share one trial directory".into());
    }
    let directory = parent.join("model-connection");
    fs::create_dir(&directory)?;
    let mut conversations = Vec::new();
    for participant in people {
        fs::create_dir(&participant.output)?;
        conversations.push(Conversation {
            trace: Trace::create(&participant.output.join("events.jsonl"))?,
            participant,
            started: Instant::now(),
            observed: Observed::default(),
        });
    }
    let mut child = Process::spawn(
        Command::new(CLI)
            .args([
                "-c",
                "forced_login_method=\"chatgpt\"",
                "-c",
                "model_provider=\"openai\"",
                "app-server",
                "--listen",
                "stdio://",
            ])
            .env_remove("OPENAI_API_KEY")
            .env_remove("CODEX_API_KEY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(fs::File::create_new(directory.join("stderr.log"))?),
    )?;
    let (input, output) = child.io()?;
    let mut wire = Wire::new(
        input,
        output,
        Trace::create(&directory.join("events.jsonl"))?,
    );
    let mut completed = Vec::new();
    let outcome: io::Result<Result<()>> = race(
        async {
            Ok(driver::run(
                &mut wire,
                &mut conversations,
                &mut completed,
                limits.tool_calls,
                guest,
            )
            .await)
        },
        deadline(limits.timeout),
    )
    .await;
    let outcome = match child.terminate() {
        Ok(_) => outcome,
        Err(error) => Ok(Err(error.into())),
    };
    finish_reports(&conversations, completed, outcome)
}

fn finish_reports(
    conversations: &[Conversation],
    mut completed: Vec<Value>,
    outcome: io::Result<Result<()>>,
) -> Result<Vec<Value>> {
    let error = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(("harness_error", error.to_string())),
        Err(_) => Some(("timeout", "participant group timeout".to_owned())),
    };
    if let Some((status, error)) = error {
        if conversations.iter().all(|conversation| {
            completed
                .iter()
                .any(|result| result["participant"] == conversation.participant.agent)
        }) {
            return Err(format!("model participant group {status}: {error}").into());
        }
        for conversation in conversations {
            if !completed
                .iter()
                .any(|result| result["participant"] == conversation.participant.agent)
            {
                completed.push(conversation.finish(&json!({"status":status,"error":error}))?);
            }
        }
    }
    Ok(completed)
}
