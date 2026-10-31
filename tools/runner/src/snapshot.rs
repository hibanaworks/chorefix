use crate::evidence::{Result, archive_build_receipt, digest, save};
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
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

/// Verify the actual runtime sources, including every source file and both manifests.
/// A binary hash alone cannot identify which source a build receipt describes.
pub fn runtime_sources(source: &Path, manifest: &Path) -> Result<BTreeMap<String, String>> {
    let mut entries = BTreeMap::new();
    for row in fs::read_to_string(manifest)?.lines() {
        let (hash, name) = row.split_once("  ").ok_or("invalid build-source row")?;
        if hash.len() != 64
            || !hash.bytes().all(|c| c.is_ascii_hexdigit())
            || name.is_empty()
            || !Path::new(name)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            || entries.insert(name.to_owned(), hash.to_owned()).is_some()
        {
            return Err("invalid or duplicate build-source entry".into());
        }
        if digest(&source.join(name))? != hash {
            return Err(format!("runtime source differs from its build: {name}").into());
        }
    }
    let mut required = Vec::new();
    files(&source.join("src"), &mut required)?;
    required.extend([source.join("Cargo.toml"), source.join("Cargo.lock")]);
    for name in [".cargo/config", ".cargo/config.toml", "build.rs"] {
        let path = source.join(name);
        if path.exists() {
            required.push(path);
        }
    }
    if required.iter().any(|path| {
        path.strip_prefix(source)
            .ok()
            .and_then(Path::to_str)
            .is_none_or(|name| !entries.contains_key(name))
    }) {
        return Err("build-source manifest omits runtime source files".into());
    }
    Ok(entries)
}

/// Preserve the selected executable, its checked build receipt and experiment inputs.
/// This selects artifacts before a run; it cannot authorize a guest operation.
pub fn freeze(task: &str, output: &Path) -> Result<Value> {
    choreofix_inputs::verify_benchmark(Path::new("."))?;
    let family = match task {
        "1E" | "1M" | "1H" => "CODEBASE",
        "2E" | "2M" | "2H" => "WRITING",
        "6E" => "REVIEW",
        "8E" => "API",
        "9E" | "9M" | "9H" => "PHILOSOPHERS",
        "16E" => "CICD",
        _ => return Err("task has no delegated WASI implementation".into()),
    };
    let runtime_key = format!("CHOREOFIX_{family}_WASI_BINARY");
    let receipt_key = format!("CHOREOFIX_{family}_BUILD_RECEIPT");
    let runtime = PathBuf::from(std::env::var_os(&runtime_key).ok_or(runtime_key)?);
    let input_receipt = PathBuf::from(std::env::var_os(&receipt_key).ok_or(receipt_key)?);
    let receipt = archive_build_receipt(&runtime, &input_receipt, output)?;
    let build_sources = runtime_sources(Path::new("."), &output.join("build-source.sha256"))?;
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
    paths.extend(build_sources.keys().map(PathBuf::from));
    paths.sort();
    paths.dedup();
    let mut manifest = Map::new();
    for path in paths {
        let saved = output.join("source").join(&path);
        fs::create_dir_all(saved.parent().ok_or("source parent")?)?;
        fs::copy(&path, &saved)?;
        manifest.insert(path.to_string_lossy().into_owned(), json!(digest(&saved)?));
    }
    runtime_sources(&output.join("source"), &output.join("build-source.sha256"))?;
    save(&output.join("source-manifest.json"), &json!(manifest))?;
    let artifacts = [
        ("runtime", runtime),
        (
            "wash.wasm",
            PathBuf::from("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
        ),
        ("runner", std::env::current_exe()?),
    ];
    let mut binaries = Map::new();
    for (name, source) in artifacts {
        let saved = output.join(name);
        fs::copy(source, &saved)?;
        binaries.insert(name.into(), json!(digest(&saved)?));
    }
    if binaries["runtime"] != receipt["binary_sha256"] {
        return Err("runtime changed while archiving its build".into());
    }
    let projection = if matches!(task, "6E" | "16E") {
        let projection = crate::projection::archive(&output.join("runtime"), output, task)?;
        if projection["runtime_binary_sha256"] != binaries["runtime"] {
            return Err("compiled projection executable differs from the snapshot".into());
        }
        json!(digest(&output.join("projection.json"))?)
    } else {
        Value::Null
    };
    let versions = json!({
        "format_version":4,
        "consumer":command("git", &["rev-parse", "HEAD"])?,
        "host_toolchain":command("rustc", &["+stable", "-Vv"])?,
        "runtime_build_receipt":receipt,
        "build_receipt_sha256":digest(&output.join("build-receipt.json"))?,
        "input_build_receipt_sha256":digest(&output.join("input-build-receipt.json"))?,
        "runtime_source_manifest_sha256":digest(&output.join("build-source.sha256"))?,
        "upstream_interpreter":crate::oracle::interpreter_identity()?,
        "binaries":binaries,
        "source_manifest_sha256":digest(&output.join("source-manifest.json"))?,
        "compiled_projection_sha256":projection,
    });
    fs::write(
        output.join("dependency-tree.txt"),
        command("cargo", &["+stable", "tree", "--locked", "-i", "hibana"])?,
    )?;
    save(&output.join("versions.json"), &versions)?;
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn a_build_manifest_must_cover_the_actual_runtime_and_reject_changed_or_escaping_sources() {
        let root = std::env::temp_dir().join(format!(
            "choreofix-runtime-source-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        for (name, bytes) in [
            ("src/lib.rs", "pub fn execute() {}\n"),
            ("Cargo.toml", "[package]\n"),
            ("Cargo.lock", "version = 4\n"),
        ] {
            fs::write(root.join(name), bytes).unwrap();
        }
        let entries: Vec<_> = ["src/lib.rs", "Cargo.toml", "Cargo.lock"]
            .map(|name| format!("{}  {name}\n", digest(&root.join(name)).unwrap()))
            .into();
        let manifest = root.join("build-source.sha256");
        let original = entries.concat();
        fs::write(&manifest, &original).unwrap();
        assert_eq!(runtime_sources(&root, &manifest).unwrap().len(), 3);
        for bad in [
            String::new(),
            entries[1..].concat(),
            format!("{original}{}", entries[0]),
            format!("{}  ../Cargo.toml\n", "0".repeat(64)),
            format!("{}  /tmp/source.rs\n", "0".repeat(64)),
        ] {
            fs::write(&manifest, bad).unwrap();
            assert!(runtime_sources(&root, &manifest).is_err());
        }
        fs::write(&manifest, &original).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn bypass() {}\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::write(root.join("src/lib.rs"), "pub fn execute() {}\n").unwrap();
        fs::write(root.join("src/receiver.rs"), "pub fn effect() {}\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::remove_file(root.join("src/receiver.rs")).unwrap();
        fs::create_dir(root.join(".cargo")).unwrap();
        let config = root.join(".cargo/config.toml");
        fs::write(&config, "[build]\njobs = 1\n").unwrap();
        assert!(runtime_sources(&root, &manifest).is_err());
        fs::write(
            &manifest,
            format!(
                "{original}{}  .cargo/config.toml\n",
                digest(&config).unwrap()
            ),
        )
        .unwrap();
        assert_eq!(runtime_sources(&root, &manifest).unwrap().len(), 4);
        fs::remove_dir_all(root).unwrap();
    }
}
