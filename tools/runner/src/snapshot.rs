use crate::{
    evidence::{Result, digest, save},
    model,
};
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeSet,
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

pub fn verify_upstream(source: &Path) -> Result<()> {
    let manifest: Value =
        serde_json::from_slice(&fs::read(source.join("benchmark/upstream-manifest.json"))?)?;
    let tasks = manifest["tasks"].as_array().ok_or("upstream tasks")?;
    if tasks.len() != 48 {
        return Err("upstream task set changed".into());
    }
    let mut expected = BTreeSet::new();
    for mapping in tasks
        .iter()
        .map(|v| &v["files"])
        .chain(std::iter::once(&manifest["simulator_files"]))
    {
        for (relative, hash) in mapping.as_object().ok_or("upstream file hashes")? {
            let path = source.join("vendor/tracefix").join(relative);
            if json!(digest(&path)?) != *hash {
                return Err(format!("upstream bytes differ: {relative}").into());
            }
            expected.insert(path);
        }
    }
    let mut actual = Vec::new();
    let mut ids = BTreeSet::new();
    for task in tasks {
        let id = task["id"].as_str().ok_or("upstream task id")?;
        ids.insert(id.to_owned());
        for directory in ["descriptions", "environments"] {
            files(
                &source
                    .join("vendor/tracefix/benchmark")
                    .join(directory)
                    .join(id),
                &mut actual,
            )?;
        }
    }
    let actual_ids: BTreeSet<_> =
        fs::read_dir(source.join("vendor/tracefix/benchmark/descriptions"))?
            .filter_map(|e| match e {
                Ok(e) if e.path().is_dir() => {
                    Some(Ok(e.file_name().to_string_lossy().into_owned()))
                }
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<std::io::Result<_>>()?;
    if ids != actual_ids {
        return Err("upstream task directories changed".into());
    }
    files(&source.join("vendor/tracefix/benchmark/tools"), &mut actual)?;
    // The upstream manifest inventories every description/environment input and
    // every Python simulator file; package documentation is outside that scope.
    let actual: BTreeSet<_> = actual
        .into_iter()
        .filter(|p| {
            !p.starts_with(source.join("vendor/tracefix/benchmark/tools"))
                || p.extension().is_some_and(|e| e == "py")
        })
        .collect();
    if actual != expected {
        return Err("upstream file set changed".into());
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

pub fn freeze(output: &Path) -> Result<Value> {
    verify_upstream(Path::new("."))?;
    let mut paths = Vec::new();
    for directory in [
        "src",
        "tools/runner",
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
            "tools/original_writing_simulator.py",
            "benchmark/upstream-manifest.json",
            "benchmark/experiment-contract.json",
            "vendor/hibana-wash/Cargo.toml",
            "vendor/hibana-wash/Cargo.lock",
        ]
        .map(PathBuf::from),
    );
    paths.sort();
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
        (
            "runtime",
            PathBuf::from("output/writing-direct-wasi-final-runtime"),
        ),
        (
            "wash.wasm",
            PathBuf::from("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
        ),
        ("runner", std::env::current_exe()?),
        (
            "protocol-audit",
            PathBuf::from("target/debug/writing-boundary"),
        ),
    ] {
        let saved = output.join(name);
        fs::copy(source, &saved)?;
        binaries.insert(name.into(), json!(digest(&saved)?));
    }
    let versions = json!({
        "consumer":command("git", &["rev-parse","HEAD"])?,
        "consumer_diff":command("git", &["diff","--stat"])?,
        "runtime_source_revision":"4096a2a6d3da146e73d7633d25b434a4859fe33d",
        "wash":command("git", &["-C","vendor/hibana-wash","rev-parse","HEAD"])?,
        "compiler":command("rustc", &["+1.95.0","-Vv"])?,
        "model_client":command(model::CLI, &["--version"])?,
        "model_client_sha256":digest(Path::new(model::CLI))?,
        "binaries":binaries,
        "source_manifest_sha256":digest(&output.join("source-manifest.json"))?,
        "model":model::MODEL,"effort":model::EFFORT,
    });
    fs::write(
        output.join("dependency-tree.txt"),
        command("cargo", &["+1.95.0", "tree", "--locked", "-i", "hibana"])?,
    )?;
    save(&output.join("versions.json"), &versions)?;
    Ok(versions)
}

#[cfg(test)]
mod tests {
    #[test]
    fn original_manifest_scope_matches_all_48_task_directories_and_simulators() {
        super::verify_upstream(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        )
        .unwrap();
    }
}
