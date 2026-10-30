//! Read Wasm declarations without validating instructions or executing a guest.
use crate::Result;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let (value, rest) = self.0.split_at_checked(count).ok_or("truncated Wasm")?;
        self.0 = rest;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn unsigned(&mut self, bits: u32) -> Result<u64> {
        let mut value = 0;
        for shift in (0..bits).step_by(7) {
            let byte = self.byte()?;
            let digit = u64::from(byte & 127);
            if digit >= 1 << (bits - shift).min(7) {
                return Err("overflowing Wasm integer".into());
            }
            value |= digit << shift;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err("oversized Wasm integer".into())
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::try_from(self.unsigned(32)?)?)
    }

    fn name(&mut self) -> Result<&'a str> {
        let count = self.u32()? as usize;
        Ok(std::str::from_utf8(self.take(count)?)?)
    }

    fn limits(&mut self) -> Result<Value> {
        let flags = self.u32()?;
        if flags > 7 || flags & 2 != 0 && flags & 1 == 0 {
            return Err("invalid Wasm limit flags".into());
        }
        let bits = if flags & 4 == 0 { 32 } else { 64 };
        let minimum = self.unsigned(bits)?;
        let mut result = json!({"flags":flags,"minimum":minimum});
        if flags & 1 != 0 {
            let maximum = self.unsigned(bits)?;
            if maximum < minimum {
                return Err("Wasm maximum below minimum".into());
            }
            result["maximum"] = json!(maximum);
        }
        Ok(result)
    }
}

fn declarations(bytes: &[u8]) -> Result<Value> {
    let mut module = Reader(bytes);
    if module.take(8)? != b"\0asm\x01\0\0\0" {
        return Err("expected a Wasm 1 module".into());
    }
    let mut report =
        json!({"inspection_scope":"declarations only; not module validation or execution"});
    let mut imports = Vec::new();
    let mut memories = Vec::new();
    let mut tables = Vec::new();
    let mut seen = BTreeSet::new();
    let mut function_count = 0;
    let mut body_count = 0;
    while !module.0.is_empty() {
        let id = module.byte()?;
        let length = module.u32()? as usize;
        let mut section = Reader(module.take(length)?);
        if id > 13 || id != 0 && !seen.insert(id) {
            return Err("unknown or duplicate Wasm section".into());
        }
        match id {
            1 => {
                let count = section.u32()?;
                let (mut parameters, mut results) = (0, 0);
                for _ in 0..count {
                    if section.byte()? != 0x60 {
                        return Err("non-function Wasm type".into());
                    }
                    let n = section.u32()?;
                    section.take(n as usize)?;
                    parameters = parameters.max(n);
                    let n = section.u32()?;
                    section.take(n as usize)?;
                    results = results.max(n);
                }
                report["types"] = json!(count);
                report["max_function_parameters"] = json!(parameters);
                report["max_function_results"] = json!(results);
            }
            2 => {
                for _ in 0..section.u32()? {
                    let mut item = json!({"module":section.name()?,"name":section.name()?});
                    let kind = section.byte()?;
                    item["kind"] = json!(kind);
                    match kind {
                        0 => item["type"] = json!(section.u32()?),
                        1 => {
                            item["element_type"] = json!(section.byte()?);
                            item["limits"] = section.limits()?;
                        }
                        2 => item["limits"] = section.limits()?,
                        3 => item["global_type"] = json!(section.take(2)?),
                        4 => {
                            let attribute = section.byte()?;
                            if attribute != 0 {
                                return Err("invalid Wasm tag attribute".into());
                            }
                            item["attribute"] = json!(attribute);
                            item["type"] = json!(section.u32()?);
                        }
                        _ => return Err("unknown Wasm import kind".into()),
                    }
                    imports.push(item);
                }
            }
            3 => {
                function_count = section.u32()?;
                report["defined_functions"] = json!(function_count);
                for _ in 0..function_count {
                    section.u32()?;
                }
            }
            4 => {
                for _ in 0..section.u32()? {
                    let element = section.byte()?;
                    let mut limits = section.limits()?;
                    limits["element_type"] = json!(element);
                    tables.push(limits);
                }
            }
            5 => {
                for _ in 0..section.u32()? {
                    memories.push(section.limits()?);
                }
            }
            6 => report["globals"] = json!(section.u32()?),
            9 => report["element_segments"] = json!(section.u32()?),
            10 => {
                body_count = section.u32()?;
                let (mut largest, mut most_locals) = (0, 0u64);
                for _ in 0..body_count {
                    let size = section.u32()?;
                    let mut body = Reader(section.take(size as usize)?);
                    let mut locals = 0u64;
                    for _ in 0..body.u32()? {
                        locals = locals
                            .checked_add(u64::from(body.u32()?))
                            .ok_or("local count overflow")?;
                        body.byte()?;
                    }
                    largest = largest.max(size);
                    most_locals = most_locals.max(locals);
                }
                report["largest_function_body_bytes"] = json!(largest);
                report["max_declared_locals"] = json!(most_locals);
            }
            11 => report["data_segments"] = json!(section.u32()?),
            // Other sections do not contribute to these declaration metrics.
            _ => {}
        }
        if matches!(id, 1 | 2 | 3 | 4 | 5 | 10) && !section.0.is_empty() {
            return Err("trailing Wasm declaration bytes".into());
        }
    }
    if function_count != body_count {
        return Err("Wasm function/body counts differ".into());
    }
    report["non_wasi_imports"] = json!(
        imports
            .iter()
            .filter(|i| i["module"] != "wasi_snapshot_preview1")
            .collect::<Vec<_>>()
    );
    report["imports"] = json!(imports);
    report["memories"] = json!(memories);
    report["tables"] = json!(tables);
    Ok(report)
}

pub fn profile(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    let mut report = declarations(&bytes)?;
    report["module"] = json!(path.to_str().ok_or("module path UTF-8")?);
    report["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    report["bytes"] = json!(bytes.len());
    Ok(report)
}

fn check_wash(report: &Value) -> Result<()> {
    if report["non_wasi_imports"]
        .as_array()
        .ok_or("Wasm imports")?
        .is_empty()
        && report["imports"]
            .as_array()
            .ok_or("Wasm imports")?
            .iter()
            .all(|i| i["kind"] == 0)
        && report["memories"] == json!([{"flags":0,"minimum":1}])
    {
        Ok(())
    } else {
        Err("wash requires only WASIp1 imports and one initially 64-KiB memory".into())
    }
}

pub fn wash(path: &Path) -> Result<Value> {
    let report = profile(path)?;
    check_wash(&report)?;
    Ok(report)
}

#[cfg(test)]
mod tests;
