//! Offline measurement only. Never read by a task, guest or resolver.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader},
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn peak(intervals: &[(u64, u64)]) -> Result<usize> {
    let mut edges = Vec::new();
    for &(start, end) in intervals {
        if start >= end {
            return Err("invalid domain interval".into());
        }
        edges.push((start, 1i32));
        edges.push((end, -1));
    }
    edges.sort();
    let (mut active, mut maximum) = (0i32, 0);
    for (_, delta) in edges {
        active += delta;
        maximum = maximum.max(active)
    }
    if active != 0 {
        return Err("unbalanced domain intervals".into());
    }
    Ok(maximum as usize)
}
fn measure(path: &Path) -> Result<Value> {
    let mut calls = BTreeMap::new();
    let mut rows = Vec::new();
    let mut used = BTreeSet::new();
    let mut fixture = None;
    for line in BufReader::new(fs::File::open(path)?).lines() {
        let r: Value = serde_json::from_str(&line?)?;
        let m = &r["message"];
        match (r["direction"].as_str(), m["type"].as_str()) {
            (Some("simulator"), Some("fixtures")) => {
                if fixture.replace(m.clone()).is_some() {
                    return Err("duplicate fixture".into());
                }
            }
            (Some("runtime"), Some("execute")) => {
                let id = m["id"].as_u64().ok_or("execute ID")?;
                if !used.insert(id) || calls.insert(id, m.clone()).is_some() {
                    return Err("duplicate call".into());
                }
            }
            (Some("simulator"), Some("simulator_result")) => {
                let id = m["id"].as_u64().ok_or("result ID")?;
                let c = calls.remove(&id).ok_or("unmatched domain result")?;
                let start = m["started_ns"].as_u64().ok_or("start time")?;
                let end = m["finished_ns"].as_u64().ok_or("end time")?;
                rows.push(json!({"id":id,"agent":c["agent"],"tool":c["tool"],"started_ns":start,"finished_ns":end}));
            }
            _ => {}
        }
    }
    let fixture = fixture.ok_or("missing fixture")?;
    if fixture["model_run"] != true
        || fixture["test_delays"] != json!({})
        || fixture["delay_multiplier"] != 1.0
        || !calls.is_empty()
        || rows.is_empty()
    {
        return Err("not a complete unmodified-delay model trace".into());
    }
    rows.sort_by_key(|r| r["started_ns"].as_u64().unwrap());
    let first = rows[0]["started_ns"].as_u64().unwrap();
    let all: Vec<_> = rows
        .iter()
        .map(|r| {
            (
                r["started_ns"].as_u64().unwrap(),
                r["finished_ns"].as_u64().unwrap(),
            )
        })
        .collect();
    let commits: Vec<_> = rows
        .iter()
        .filter(|r| r["tool"] == "commit_changes")
        .map(|r| {
            (
                r["started_ns"].as_u64().unwrap(),
                r["finished_ns"].as_u64().unwrap(),
            )
        })
        .collect();
    for r in &mut rows {
        r["start_seconds"] = json!((r["started_ns"].as_u64().unwrap() - first) as f64 / 1e9);
        r["end_seconds"] = json!((r["finished_ns"].as_u64().unwrap() - first) as f64 / 1e9)
    }
    Ok(
        json!({"task":fixture["task"],"scored_benchmark":false,"measurement":"Simulator RPC entry immediately before registry.call through return; includes the unchanged simulator's asynchronous tool delay","artificial_delays":false,"max_admitted_domain_calls":peak(&all)?,"max_shared_commits":peak(&commits)?,"intervals":rows}),
    )
}
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: domain-intervals EVENTS.jsonl")?;
    println!(
        "{}",
        serde_json::to_string_pretty(&measure(Path::new(&path))?)?
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn disjoint_intervals_do_not_overlap_at_a_boundary() {
        assert_eq!(super::peak(&[(1, 3), (3, 5)]).unwrap(), 1)
    }
    #[test]
    fn independent_nested_work_is_counted() {
        assert_eq!(super::peak(&[(1, 10), (2, 4), (3, 5), (4, 6)]).unwrap(), 3)
    }
    #[test]
    fn invalid_or_zero_intervals_are_rejected() {
        assert!(super::peak(&[(2, 1)]).is_err());
        assert!(super::peak(&[(1, 1)]).is_err())
    }
}
