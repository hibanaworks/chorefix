use crate::{
    evidence::{Result, digest, save},
    model,
};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn files(path: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if ["target", "__pycache__", ".git"].contains(&entry.file_name().to_string_lossy().as_ref())
        {
            continue;
        }
        if entry.file_type()?.is_dir() {
            files(&entry.path(), result)?;
        } else if entry.file_type()?.is_file() {
            result.push(entry.path());
        }
    }
    Ok(())
}

fn command(program: &str, arguments: &[&str]) -> Result<String> {
    let result = Command::new(program).args(arguments).output()?;
    if !result.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

pub const QUALIFICATION: &str = "results/qualified-direct-wasi-build.json";

/// Build evidence selects the measured executable before a run. This has no
/// access to a guest or any task transition.
pub fn qualification(source: &Path, task: &str) -> Result<Value> {
    let record: Value = serde_json::from_slice(&fs::read(source.join(QUALIFICATION))?)?;
    let case = &record["cases"][task];
    if record["format_version"] != 1 || case.is_null() {
        return Err("missing task build qualification".into());
    }
    let revision = record["source_revision"]
        .as_str()
        .ok_or("qualified source revision")?;
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid qualified source revision".into());
    }
    for artifact in [&record["source_archive"], &case["audit"]]
        .into_iter()
        .chain(
            record["checks"]
                .as_array()
                .ok_or("qualification checks")?
                .iter(),
        )
    {
        let relative = artifact["path"]
            .as_str()
            .ok_or("qualification artifact path")?;
        if Path::new(relative)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
            || json!(digest(&source.join(relative))?) != artifact["sha256"]
        {
            return Err("qualification artifact differs".into());
        }
    }
    let audit: Value = serde_json::from_slice(&fs::read(
        source.join(case["audit"]["path"].as_str().ok_or("audit path")?),
    )?)?;
    if audit["task"] != task
        || audit["status"] != "passed"
        || audit["model_calls"] != 0
        || audit["simulator_complete"] != true
        || audit["violations"] != 0
    {
        return Err("task boundary was not qualified".into());
    }
    Ok(record)
}

pub fn freeze(task: &str, output: &Path) -> Result<Value> {
    choreofix_inputs::verify_benchmark(Path::new("."))?;
    let qualification = qualification(Path::new("."), task)?;
    let case = &qualification["cases"][task];
    let runtime = case["runtime_path"].as_str().ok_or("runtime path")?;
    let verifier = case["protocol_verifier_path"]
        .as_str()
        .ok_or("verifier path")?;
    if json!(digest(Path::new(runtime))?) != case["runtime_sha256"]
        || json!(digest(Path::new(verifier))?) != case["protocol_verifier_sha256"]
    {
        return Err("current binaries differ from qualified build".into());
    }
    let mut paths = Vec::new();
    for directory in [
        "src",
        "tools/runner",
        "tools/inputs",
        "vendor/hibana-wash/src",
        "vendor/tracefix/benchmark",
    ] {
        files(Path::new(directory), &mut paths)?;
    }
    paths.extend(
        [
            "Cargo.toml",
            "Cargo.lock",
            ".cargo/config.toml",
            "experiment.json",
            "requirements.lock",
            "AGENTS.md",
            "plan.md",
            "benchmark/upstream-manifest.json",
            "benchmark/experiment-contract.json",
            "vendor/hibana-wash/Cargo.toml",
            "vendor/hibana-wash/Cargo.lock",
        ]
        .map(PathBuf::from),
    );
    paths.push(PathBuf::from(QUALIFICATION));
    for artifact in [&qualification["source_archive"], &case["audit"]]
        .into_iter()
        .chain(
            qualification["checks"]
                .as_array()
                .ok_or("qualification checks")?
                .iter(),
        )
    {
        paths.push(PathBuf::from(
            artifact["path"].as_str().ok_or("artifact path")?,
        ));
    }
    paths.sort();
    paths.dedup();
    let mut manifest = Map::new();
    for path in paths {
        let saved = output.join("source").join(&path);
        fs::create_dir_all(saved.parent().ok_or("source parent")?)?;
        fs::copy(&path, &saved)?;
        manifest.insert(path.to_string_lossy().into_owned(), json!(digest(&saved)?));
    }
    save(&output.join("source-manifest.json"), &json!(manifest))?;
    let mut binaries = Map::new();
    for (name, source) in [
        ("runtime", PathBuf::from(runtime)),
        (
            "wash.wasm",
            PathBuf::from("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
        ),
        ("runner", std::env::current_exe()?),
        ("protocol-audit", PathBuf::from(verifier)),
    ] {
        let saved = output.join(name);
        fs::copy(source, &saved)?;
        binaries.insert(name.into(), json!(digest(&saved)?));
    }
    let versions = json!({
        "consumer":command("git", &["rev-parse","HEAD"])?,
        "consumer_diff":command("git", &["diff","--stat"])?,
        "format_version":2,
        "runtime_source_revision":qualification["source_revision"],
        "qualification_sha256":digest(Path::new(QUALIFICATION))?,
        "wash":command("git", &["-C","vendor/hibana-wash","rev-parse","HEAD"])?,
        "host_toolchain":command("rustc", &["+stable","-Vv"])?,
        "runtime_build_toolchain":qualification["compiler"],
        "upstream_interpreter":crate::oracle::interpreter_identity()?,
        "model_client":command(model::CLI, &["--version"])?,
        "model_client_sha256":digest(Path::new(model::CLI))?,
        "binaries":binaries,
        "source_manifest_sha256":digest(&output.join("source-manifest.json"))?,
        "model":model::MODEL,"effort":model::EFFORT,
    });
    if versions["binaries"]["wash.wasm"] != qualification["wash_sha256"] {
        return Err("guest module differs from qualified build".into());
    }
    fs::write(
        output.join("dependency-tree.txt"),
        command("cargo", &["+stable", "tree", "--locked", "-i", "hibana"])?,
    )?;
    save(&output.join("versions.json"), &versions)?;
    Ok(versions)
}
