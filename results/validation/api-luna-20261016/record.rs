//! Aggregate immutable model trials after the external Rust audits have passed.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn digest(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn save(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
fn command(c: &mut Command) -> Result<()> {
    let result = c.status()?;
    if result.success() {
        Ok(())
    } else {
        Err(format!("archive failed: {result}").into())
    }
}
fn main() -> Result<()> {
    let base = Path::new("results/validation/api-luna-20261016");
    let runs = Path::new("output/direct-wasi-preflight/api-luna-20261016-001");
    let verification = fs::read_to_string(base.join("verification-source-revision.txt"))?
        .trim()
        .to_owned();
    let mut trials = Vec::new();
    let mut complete_archives = String::new();
    let mut tokens = 0u64;
    let mut shell = 0u64;
    for depth in [0, 1, 2] {
        let name = format!("depth{depth}");
        let run = runs.join(&name);
        let saved = base.join(&name);
        fs::create_dir(&saved)?;
        let versions = load(&run.join("versions.json"))?;
        let result = load(&run.join("result.json"))?;
        let audit = load(&run.join("audit-v2.json"))?;
        let timing = load(&run.join("timing-audit.json"))?;
        let mutants = load(&run.join("mutants-v2.json"))?;
        if versions["consumer"] != "50ce492fab130c6c8fdaeb4ad0b58e08cf372148"
            || versions["consumer_diff"] != ""
            || versions["runtime_source_revision"] != "29344a8a4c6800ff583a3d24143fa592de6f6a17"
            || audit["status"] != "passed"
            || audit["max_simultaneous_model_turns"] != 3
            || audit["protocol"]["domain_calls"] != 3
            || audit["protocol"]["simulator_complete"] != (depth == 0)
            || timing["early_result_opens"] != 1
            || timing["result_opens"] != 2
        {
            return Err("unexpected trial evidence".into());
        }
        let checks = mutants["checked"].as_array().ok_or("mutation checks")?;
        if checks.len() != 8
            || checks
                .iter()
                .any(|m| m["status"] != "rejected" || m["outer_hashes_recomputed"] != true)
        {
            return Err("mutation check failed".into());
        }
        let subjects = result["outcomes"]
            .as_array()
            .ok_or("outcomes")?
            .iter()
            .filter(|r| r["kind"] == "model_participant")
            .collect::<Vec<_>>();
        if subjects.len() != 3
            || subjects.iter().any(|r| {
                r["result"]["returned_model"] != "gpt-6-luna"
                    || r["result"]["returned_effort"] != "xhigh"
                    || r["result"]["status"] != "completed"
            })
        {
            return Err("model substitution or failed participant".into());
        }
        tokens += audit["inclusive_tokens"].as_u64().ok_or("tokens")?;
        let calls = subjects
            .iter()
            .map(|r| r["result"]["tool_calls"].as_u64().ok_or("shell calls"))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .iter()
            .sum::<u64>();
        shell += calls;
        for (from, to) in [
            ("audit-v2.json", "audit.json"),
            ("timing-audit.json", "timing-audit.json"),
            ("mutants-v2.json", "mutants.json"),
            ("versions.json", "versions.json"),
            ("source-manifest.json", "source-manifest.json"),
        ] {
            fs::copy(run.join(from), saved.join(to))?;
        }
        command(
            Command::new("tar")
                .arg("-czf")
                .arg(saved.join("execution-source.tar.gz"))
                .arg("-C")
                .arg(run.join("source"))
                .arg("."),
        )?;
        command(
            Command::new("tar")
                .arg("-czf")
                .arg(saved.join("raw.tar.gz"))
                .args([
                    "--exclude=source",
                    "--exclude=runtime",
                    "--exclude=runner",
                    "--exclude=protocol-audit",
                    "--exclude=wash.wasm",
                ])
                .arg("-C")
                .arg(&run)
                .arg("."),
        )?;
        let complete = runs.join(format!("{name}-complete.tar.gz"));
        command(
            Command::new("tar")
                .arg("-czf")
                .arg(&complete)
                .arg("-C")
                .arg(&run)
                .arg("."),
        )?;
        complete_archives.push_str(&format!("{}  {}\n", digest(&complete)?, complete.display()));
        let log = fs::File::create(saved.join("run.log.gz"))?;
        let status = Command::new("gzip")
            .args(["-c", &format!("/tmp/choreofix-api-luna-depth{depth}.log")])
            .stdout(log)
            .status()?;
        if !status.success() {
            return Err("log compression failed".into());
        }
        trials.push(json!({"task":"8E","depth":depth,"elapsed_seconds":result["elapsed_seconds"],"shell_calls":calls,"qualified_model_and_protocol_audit":audit,"corrected_result_read_timing_audit":timing,"mutation_checks":8,"raw_archive_sha256":digest(&saved.join("raw.tar.gz"))?}));
    }
    let status = json!({"kind":"unscored_direct_wasi_model_preflight","scored_benchmark":false,"execution_consumer":"50ce492fab130c6c8fdaeb4ad0b58e08cf372148","runtime_source_revision":"29344a8a4c6800ff583a3d24143fa592de6f6a17","verification_source_revision":verification,"model":"gpt-6-luna","reasoning_effort":"xhigh","seed":0,"domain_delay_multiplier":1.0,"artificial_barriers":false,"participants":9,"domain_calls":9,"shell_calls":shell,"inclusive_tokens":tokens,"protocol_completed":3,"domain_complete":1,"terminal_domain_failures":2,"violations":0,"trials":trials,"qualified_checker_metric_correction":"The qualified execution checker called all result opens early_result_opens. The separate timing audit counts only opens preceding the actual test result; use that corrected metric. Original raw data and binaries are unchanged.","limits":["Unscored qualification, not a success-rate or speed comparison","Task/checklist O2 conflict remains recorded","No model rerun after the external mutation-generator correction"]});
    save(&base.join("status.json"), &status)?;
    save(Path::new("results/api-direct-wasi-preflight.json"), &status)?;
    fs::write(base.join("complete-archives.sha256"), complete_archives)?;
    let path = PathBuf::from("results/task-fidelity/8E.json");
    let mut fidelity = load(&path)?;
    fidelity["direct_wasi"]["stage"] = json!("model_path_qualified");
    fidelity["direct_wasi"]["model_qualification"] = json!({"ledger":"results/api-direct-wasi-preflight.json","depths":[0,1,2],"participants":9,"protocol_completed":3,"domain_complete":1,"terminal_domain_failures":2,"inclusive_tokens":tokens,"scored_benchmark":false});
    fidelity["direct_wasi"]["remaining"] = json!([
        "Resolve task/checklist conflict before claiming full-fidelity acceptance",
        "Scored comparison"
    ]);
    save(&path, &fidelity)?;
    println!(
        "{}",
        json!({"participants":9,"domain_calls":9,"shell_calls":shell,"inclusive_tokens":tokens,"mutations_rejected":24,"domain_complete":1,"terminal_domain_failures":2,"scored_benchmark":false})
    );
    Ok(())
}
