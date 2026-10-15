use serde_json::{json,Value};
use sha2::{Sha256,Digest};
use std::{fs,path::{Path,PathBuf},process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn digest(path:&Path)->Result<String>{Ok(format!("{:x}",Sha256::digest(fs::read(path)?)))}
fn save(path:&Path,value:&Value)->Result<()> {fs::write(path,format!("{}\n",serde_json::to_string_pretty(value)?))?;Ok(())}
fn artifact(path:&Path)->Result<Value>{Ok(json!({"path":path.to_str().ok_or("path")?,"sha256":digest(path)?}))}
fn main()->Result<()> {
 let args:Vec<_>=std::env::args().skip(1).collect();let [rev,evidence,frozen]=args.as_slice() else{return Err("usage: record SOURCE_REV EVIDENCE_DIR FROZEN_BIN_DIR".into())};
 if rev.len()!=40||!rev.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid source commit".into())}
 let base=Path::new(evidence);let frozen=Path::new(frozen);
 let checks=["rust-tests.log.gz","clippy.log.gz","python-tests.log.gz","inventory.json","baseline-inventory.json","runner-tests.log.gz","runner-clippy.log.gz"];
 let mut qualification=json!({"format_version":1,"kind":"build_qualification_for_external_experiment_runner","source_revision":rev,"source_archive":artifact(&base.join("source.tar.gz"))?,"wash_sha256":digest(Path::new("vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"))?,"checks":checks.iter().map(|name|artifact(&base.join(name))).collect::<Result<Vec<_>>>()?,"cases":{}});
 for task in ["1E","1M","2E","2M","2H","9E","9M","9H"] {
  let audit_path=base.join(task).join("audit.json");let audit:Value=serde_json::from_slice(&fs::read(&audit_path)?)?;
  if audit["task"]!=task||audit["status"]!="passed"||audit["simulator_complete"]!=true||audit["model_calls"]!=0||audit["violations"]!=0{return Err("unqualified boundary".into())}
  let family=match task.as_bytes()[0]{b'1'=>"codebase",b'2'=>"writing",b'9'=>"philosophers",_=>unreachable!()};
  let runtime=frozen.join(format!("{family}-wasi"));let verifier=frozen.join(format!("{family}-boundary"));
  qualification["cases"][task]=json!({"audit":artifact(&audit_path)?,"runtime_path":runtime,"runtime_sha256":digest(&runtime)?,"protocol_verifier_path":verifier,"protocol_verifier_sha256":digest(&verifier)?});
  if !task.starts_with('1'){continue}
  let input=PathBuf::from(format!("vendor/tracefix/benchmark/environments/{task}/checklist.json"));let original:Value=serde_json::from_slice(&fs::read(input)?)?;
  let rules=original.as_array().ok_or("checklist")?.iter().map(|r|{
   let id=r["id"].as_str().unwrap();let (kind,proof)=if id.starts_with("RA"){("procedural_departure","Independent module requests precede either grant, replacing sequential acquisition; both grants still precede commit")}
    else if id.starts_with("DL"){("hazard_to_prevent","The single global orders shared module handoffs; disjoint commits remain in par")}
    else if id.starts_with('R'){("problem_constraint","The physical module Endpoint has two explicit affine visits; the audit rejects overlapping resource intervals")}
    else{("problem_constraint","Actual design and implementation complete before acquisition; returns finish before local tests")};
   json!({"id":id,"original_description":r["description"],"classification":kind,"evidence":proof})
  }).collect::<Vec<_>>();
  save(&PathBuf::from(format!("results/task-fidelity/{task}.json")),&json!({"task":task,"stage":"direct_wasi_boundary_qualified","accepted_for_scored_comparison":false,"source_revision":rev,"scored_benchmark":false,"model_calls":0,"original_rules":rules,"audit":audit,
   "interface_differences":["The owner fixes agent_id; feature strings must match the assigned feature","Module strings contain exactly the assigned pair; order and comma whitespace are unrestricted","Both independent acquisition and return requests are issued before either reply wait","The original 1E/1M terminal test semantics are retained; 1H is not included"],
   "limits":["Deterministic qualification uses artificial first-commit barriers and a delayed test; model trials use no artificial barriers","No scored comparison or success-rate estimate"],
   "remaining":["Qualify live Luna execution and complete the task acceptance and comparison interface review"]}))?;
 }
 save(Path::new("results/qualified-direct-wasi-build.json"),&qualification)?;
 let root=Command::new("git").args(["rev-parse","HEAD"]).output()?;
 println!("{}",json!({"qualification_written":true,"source_revision":rev,"recorded_at_parent":String::from_utf8(root.stdout)?.trim(),"cases":8}));
 Ok(())
}
