//! External build evidence only. Never used for task admission.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn digest(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn artifact(path: &Path) -> Result<Value> {
    Ok(json!({"path":path.to_str().ok_or("path")?,"sha256":digest(path)?}))
}
fn save(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [rev, base, frozen] = args.as_slice() else {
        return Err("usage: record SOURCE_REV EVIDENCE_DIR FROZEN_BIN_DIR".into());
    };
    if rev.len() != 40 || !rev.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("source commit".into());
    }
    let base = Path::new(base);
    let frozen = Path::new(frozen);
    let checks = [
        "rust-tests.log.gz",
        "clippy.log.gz",
        "python-tests.log.gz",
        "inventory.json",
        "baseline-inventory.json",
        "runner-tests.log.gz",
        "runner-clippy.log.gz",
        "native-stress.log.gz",
        "runner-process.log.gz",
        "8E-depth1/audit.json",
        "8E-depth2/audit.json",
        "8E/mutants.json",
        "8E-depth1/mutants.json",
        "8E-depth2/mutants.json",
    ];
    let mut record = json!({"format_version":1,"kind":"build_qualification_for_external_experiment_runner","source_revision":rev,"source_archive":artifact(&base.join("source.tar.gz"))?,"wash_sha256":digest(Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"))?,"checks":checks.iter().map(|p|artifact(&base.join(p))).collect::<Result<Vec<_>>>()?,"cases":{}});
    for task in ["1E", "1M", "2E", "2M", "2H", "8E", "9E", "9M", "9H"] {
        let path = base.join(task).join("audit.json");
        let audit: Value = serde_json::from_slice(&fs::read(&path)?)?;
        if audit["task"] != task
            || audit["status"] != "passed"
            || audit["model_calls"] != 0
            || audit["simulator_complete"] != true
            || audit["violations"] != 0
        {
            return Err("boundary failed".into());
        }
        let family = match task.as_bytes()[0] {
            b'1' => "codebase",
            b'2' => "writing",
            b'8' => "api",
            b'9' => "philosophers",
            _ => unreachable!(),
        };
        let runtime = frozen.join(format!("{family}-wasi"));
        let verifier = frozen.join(format!("{family}-boundary"));
        record["cases"][task] = json!({"audit":artifact(&path)?,"runtime_path":runtime,"runtime_sha256":digest(&runtime)?,"protocol_verifier_path":verifier,"protocol_verifier_sha256":digest(&verifier)?});
    }
    for depth in [1, 2] {
        let audit: Value =
            serde_json::from_slice(&fs::read(base.join(format!("8E-depth{depth}/audit.json")))?)?;
        if audit["task"] != "8E"
            || audit["depth"] != depth
            || audit["status"] != "passed"
            || audit["simulator_complete"] != false
            || audit["expected_terminal_test_failure"] != true
        {
            return Err("terminal failure changed".into());
        }
    }
    for name in ["8E", "8E-depth1", "8E-depth2"] {
        let m: Value = serde_json::from_slice(&fs::read(base.join(name).join("mutants.json"))?)?;
        let rows = m["mutants"].as_array().ok_or("mutant records")?;
        if rows.len() != 8 || rows.iter().any(|m| m["rejected"] != true) {
            return Err("mutations not rejected".into());
        }
    }
    let path = Path::new("results/task-fidelity/8E.json");
    let mut fidelity: Value = serde_json::from_slice(&fs::read(path)?)?;
    fidelity["direct_wasi"] = json!({"stage":"boundary_qualified","source_revision":rev,"scored_benchmark":false,"accepted_for_scored_comparison":false,"audit":artifact(&base.join("8E/audit.json"))?,"failure_audits":[artifact(&base.join("8E-depth1/audit.json"))?,artifact(&base.join("8E-depth2/audit.json"))?],"single_global_session":28,"read_boundary":"The developer's result-file request and the tester's actual notification are inputs in par; both must be received before ChoreoFS can answer","model_qualification":"pending","remaining":["Live Luna qualification","Resolve task/checklist conflict before claiming full-fidelity acceptance","Scored comparison"]});
    save(path, &fidelity)?;
    save(
        Path::new("results/qualified-direct-wasi-build.json"),
        &record,
    )?;
    println!(
        "{}",
        json!({"source_revision":rev,"cases":9,"terminal_negative_cases":2,"scored_benchmark":false})
    );
    Ok(())
}
