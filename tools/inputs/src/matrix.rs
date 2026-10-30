use crate::{LATEST, PAPER, Result, read, task_ids};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const CONDITIONS: [&str; 5] = [
    "topology_monitored",
    "mediator_enforced",
    "prompt_only",
    "chat_only",
    "hibana",
];

/// Preserve all original conditions, all 48 tasks and the latest comparison.
pub fn experiment_matrix(config: &Value) -> Result<BTreeMap<&'static str, String>> {
    if config["model"] != "gpt-6-luna"
        || config["reasoning_effort"] != "xhigh"
        || config["hibana_version"] != "0.10.0"
        || config["benchmark_revision"] != LATEST
        || config["paper_baseline_revision"] != PAPER
        || config["latest_baseline_revision"] != LATEST
    {
        return Err("participant model, effort or measured source revision changed".into());
    }
    let conditions = config["conditions"].as_array().ok_or("conditions")?;
    if conditions.len() != 5
        || conditions
            .iter()
            .map(|v| v.as_str().ok_or("condition name"))
            .collect::<std::result::Result<BTreeSet<_>, _>>()?
            != CONDITIONS.into_iter().collect()
        || config["scenario_depths"] != json!([0, 1, 2])
        || config["repetitions"] != 3
        || config["expected_runs"] != 2160
        || config["supplementary_conditions"] != json!(["latest_tracefix_monitored"])
        || config["expected_supplementary_runs"] != 432
    {
        return Err("the complete experiment conditions, depths or repetitions changed".into());
    }
    let mut original = String::new();
    let mut latest = String::new();
    let mut identities = BTreeSet::new();
    for task in task_ids() {
        for depth in 0..3 {
            for seed in 0..3 {
                for condition in conditions {
                    let condition = condition.as_str().ok_or("condition")?;
                    let mut run = json!({"task":task,"scenario_depth":depth,"repetition":seed+1,"seed":seed,
                        "condition":condition,"run_id":format!("{task}-S{depth}-R{}-{condition}",seed+1)});
                    if !identities.insert(run["run_id"].as_str().ok_or("run id")?.to_owned()) {
                        return Err("duplicate run identity".into());
                    }
                    line(&mut original, &run)?;
                    if condition == "hibana" {
                        run["condition"] = json!("latest_tracefix_monitored");
                        run["run_id"] = json!(format!(
                            "{task}-S{depth}-R{}-latest_tracefix_monitored",
                            seed + 1
                        ));
                        run["upstream_revision"] = json!(LATEST);
                        line(&mut latest, &run)?;
                    }
                }
            }
        }
    }
    Ok(BTreeMap::from([
        ("matrix.jsonl", original),
        ("latest-matrix.jsonl", latest),
    ]))
}

fn line(output: &mut String, value: &Value) -> Result<()> {
    // Flat scalar records, sorted keys and conventional spaces retain the
    // existing matrix bytes and identities without invoking another language.
    let fields = value
        .as_object()
        .ok_or("matrix record")?
        .iter()
        .map(|(key, value)| {
            Ok(format!(
                "{}: {}",
                serde_json::to_string(key)?,
                serde_json::to_string(value)?
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    output.push('{');
    output.push_str(&fields.join(", "));
    output.push_str("}\n");
    Ok(())
}

pub fn check(root: &Path) -> Result<Value> {
    for (name, bytes) in experiment_matrix(&read(&root.join("experiment.json"))?)? {
        if fs::read(root.join("benchmark").join(name))? != bytes.as_bytes() {
            return Err(format!("recorded experiment matrix differs: {name}").into());
        }
    }
    Ok(json!({"runs":2160,"supplementary_runs":432,"total_planned_runs":2592,"unchanged":true}))
}

pub fn write(root: &Path) -> Result<Value> {
    let matrix = experiment_matrix(&read(&root.join("experiment.json"))?)?;
    for (name, bytes) in matrix {
        fs::write(root.join("benchmark").join(name), bytes)?;
    }
    check(root)
}
