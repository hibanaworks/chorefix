//! Offline frame-header checks against observations from the selected executable.
use crate::evidence::{Result, digest};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};

// The canonical TapEvent identifiers and packed fields are documented by
// hibana::runtime::tap. The auditor does not link or execute the protocol kernel.
const LOCAL: u64 = 0x0300;
const SEND: u64 = 0x0301;
const RECV: u64 = 0x0302;

fn contracts(compiled: &Value) -> Result<BTreeMap<u16, [u8; 6]>> {
    if compiled["type"] != "compiled_projection"
        || compiled["format_version"] != 1
        || compiled["task"] != "6E"
    {
        return Err("unknown compiled projection format or task".into());
    }
    let mut roles = BTreeSet::new();
    let mut headers = BTreeMap::new();
    let mut directions = BTreeMap::<u16, BTreeSet<u64>>::new();
    for program in compiled["programs"].as_array().ok_or("compiled programs")? {
        let role = program["role"]
            .as_u64()
            .filter(|role| *role <= 255)
            .ok_or("compiled role")?;
        if !roles.insert(role) {
            return Err("duplicate compiled program".into());
        }
        let actor = if role == 13 { 7 } else { role };
        let mut seen = BTreeSet::new();
        for (ordinal, row) in program["rows"]
            .as_array()
            .ok_or("compiled rows")?
            .iter()
            .enumerate()
        {
            let row = row
                .as_array()
                .filter(|row| row.len() == 5)
                .ok_or("compiled row layout")?;
            let values: Vec<_> = row
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .filter(|value| *value <= u64::from(u32::MAX))
                        .ok_or("compiled row word")
                })
                .collect::<std::result::Result<_, _>>()?;
            let [at, kind, actors, packed, peers] = values[..] else {
                unreachable!()
            };
            if at != ordinal as u64
                || actors != actor << 8 | role
                || !matches!(kind, LOCAL | SEND | RECV)
            {
                return Err("invalid compiled row identity or direction".into());
            }
            let wire = (packed >> 16) as u16;
            let header = [
                (packed >> 8) as u8,
                (peers >> 24) as u8,
                (peers >> 16) as u8,
                packed as u8,
                (peers >> 8) as u8,
                peers as u8,
            ];
            if !seen.insert((wire, kind))
                || header[4] > 1
                || (header[4] == 0 && header[5] != 0)
                || ((kind == LOCAL) != (header[1] == header[2]))
            {
                return Err("duplicate or malformed compiled operation".into());
            }
            if headers
                .insert(wire, header)
                .is_some_and(|previous| previous != header)
            {
                return Err("compiled views disagree about a wire operation".into());
            }
            directions.entry(wire).or_default().insert(kind);
        }
    }
    if roles != BTreeSet::from([0, 1, 2, 5, 6, 7, 8, 9, 10, 13]) || headers.is_empty() {
        return Err("incomplete compiled program inventory".into());
    }
    for (wire, directions) in directions {
        if directions == BTreeSet::from([LOCAL]) {
            headers.remove(&wire);
        } else if directions != BTreeSet::from([SEND, RECV]) {
            return Err("compiled wire operation lacks its send or receive".into());
        }
    }
    Ok(headers)
}

/// Freeze static metadata before starting the selected runtime's guest.
pub fn archive(runtime: &Path, output: &Path) -> Result<Value> {
    let before = digest(runtime)?;
    let result = Command::new(runtime).arg("--projection").output()?;
    if !result.status.success() || !result.stderr.is_empty() || before != digest(runtime)? {
        return Err("compiled projection export failed or executable changed".into());
    }
    let compiled: Value = serde_json::from_slice(&result.stdout)?;
    contracts(&compiled)?;
    let artifact = json!({"format_version":1,"runtime_binary_sha256":before,"compiled":compiled});
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(output.join("projection.json"))?;
    writeln!(file, "{}", serde_json::to_string_pretty(&artifact)?)?;
    Ok(artifact)
}

pub fn load(events: &Path) -> Result<Value> {
    let directory = events.parent().ok_or("event directory")?;
    let path = directory.join("projection.json");
    let artifact: Value = serde_json::from_slice(&fs::read(&path)?)?;
    if artifact["format_version"] != 1
        || artifact["runtime_binary_sha256"]
            .as_str()
            .is_none_or(|hash| {
                hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
    {
        return Err("invalid compiled projection receipt".into());
    }
    // Mechanism cases and model runs have different existing receipts. Both
    // bind the frozen projection to the executable selected before execution.
    let case = directory.join("case.json");
    let versions = directory.join("versions.json");
    let (receipt, binary) = match (case.exists(), versions.exists()) {
        (true, false) => {
            let receipt: Value = serde_json::from_slice(&fs::read(case)?)?;
            let binary = receipt["runtime_binary_sha256"].clone();
            (receipt, binary)
        }
        (false, true) => {
            let receipt: Value = serde_json::from_slice(&fs::read(versions)?)?;
            let binary = receipt["binaries"]["runtime"].clone();
            (receipt, binary)
        }
        _ => return Err("expected one mechanism-case or model-run receipt".into()),
    };
    if binary != artifact["runtime_binary_sha256"]
        || receipt["compiled_projection_sha256"] != digest(&path)?
    {
        return Err("compiled projection differs from the selected executable receipt".into());
    }
    contracts(&artifact["compiled"])?;
    Ok(artifact)
}

/// Return each frame's logical message label after checking every observation.
/// This checks static membership, not dynamic branch or iteration eligibility.
pub fn frames(artifact: &Value, rows: &[Value]) -> Result<BTreeMap<u64, u8>> {
    let headers = contracts(&artifact["compiled"])?;
    let mut labels = BTreeMap::new();
    let mut observed = false;
    for row in rows {
        if row["direction"] != "runtime" {
            continue;
        }
        let message = &row["message"];
        match message["type"].as_str() {
            Some("compiled_projection") => {
                if observed || !labels.is_empty() || message != &artifact["compiled"] {
                    return Err("runtime projection differs from the frozen artifact".into());
                }
                observed = true;
            }
            Some("execute") if !observed => {
                return Err("effect before compiled projection observation".into());
            }
            Some("protocol_frame") => {
                if !observed {
                    return Err("frame before compiled projection observation".into());
                }
                let wire = message["frame_label"]
                    .as_u64()
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or("wire frame label")?;
                let header = headers
                    .get(&wire)
                    .ok_or("frame label absent from compiled projection")?;
                for (field, expected) in ["lane", "source", "target"].into_iter().zip(header) {
                    if message[field] != *expected {
                        return Err(
                            format!("frame {field} differs from compiled projection").into()
                        );
                    }
                }
                if header[4] == 1 {
                    let [high, low] = wire.to_be_bytes();
                    if message["payload"] != json!([header[5], high, low]) {
                        return Err(
                            "delegation payload differs from compiled resource and occurrence"
                                .into(),
                        );
                    }
                }
                let id = message["frame_id"].as_u64().ok_or("frame ID")?;
                if labels
                    .insert(id, header[3])
                    .is_some_and(|previous| previous != header[3])
                {
                    return Err("frame changed its compiled message contract".into());
                }
            }
            _ => {}
        }
    }
    if !observed {
        return Err("missing runtime projection observation".into());
    }
    Ok(labels)
}

#[cfg(test)]
mod tests;
