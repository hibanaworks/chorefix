use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::mpsc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
/// Append requests to the thread that owns the trace file.
pub struct Trace(mpsc::Sender<(String, Value, mpsc::SyncSender<Result<()>>)>);
impl Trace {
    pub fn create(path: &Path) -> Result<Self> {
        Self::from_file(File::create_new(path)?)
    }

    fn from_file(mut file: File) -> Result<Self> {
        let (send, receive) = mpsc::channel::<(String, Value, mpsc::SyncSender<Result<()>>)>();
        let started = Instant::now();
        std::thread::Builder::new()
            .name("experiment-trace".into())
            .spawn(move || {
                while let Ok((direction, message, reply)) = receive.recv() {
                    let result = (|| -> Result<()> {
                        serde_json::to_writer(
                            &mut file,
                            &json!({"elapsed_seconds":started.elapsed().as_secs_f64(), "unix_ns":SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos().to_string(), "direction":direction, "message":message}),
                        )?;
                        writeln!(file)?;
                        file.flush()?;
                        Ok(())
                    })();
                    match result {
                        Ok(()) => {
                            if reply.send(Ok(())).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = reply.send(Err(error));
                            break;
                        }
                    }
                }
            })?;
        Ok(Self(send))
    }
    pub fn record(&self, direction: &str, message: &Value) -> Result<()> {
        let (send, receive) = mpsc::sync_channel(0);
        self.0
            .send((direction.to_owned(), message.clone(), send))
            .map_err(|_| "trace writer closed")?;
        receive
            .recv()
            .map_err(|_| "trace writer stopped before replying")?
    }
}

pub fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        hash.update(&buffer[..length]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn save(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

pub fn archive_build_receipt(runtime: &Path, input: &Path, directory: &Path) -> Result<Value> {
    let original = fs::read(input)?;
    let mut receipt: Value = serde_json::from_slice(&original)?;
    if receipt["binary_sha256"] != digest(runtime)? || receipt["source_unchanged"] != true {
        return Err("runtime binary differs from its build receipt".into());
    }
    let manifest = input.parent().ok_or("receipt directory")?.join(
        receipt["source_manifest"]
            .as_str()
            .ok_or("source manifest path")?,
    );
    let manifest = fs::read(manifest)?;
    if receipt["source_manifest_sha256"] != format!("{:x}", Sha256::digest(&manifest)) {
        return Err("source manifest differs from its build receipt".into());
    }
    receipt["input_receipt_sha256"] = format!("{:x}", Sha256::digest(&original)).into();
    receipt["source_manifest"] = "build-source.sha256".into();
    let mut normalized = serde_json::to_vec_pretty(&receipt)?;
    normalized.push(b'\n');
    for (name, bytes) in [
        ("build-source.sha256", manifest.as_slice()),
        ("input-build-receipt.json", original.as_slice()),
        ("build-receipt.json", normalized.as_slice()),
    ] {
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(directory.join(name))?;
        file.write_all(bytes)?;
        file.flush()?;
    }
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "choreofix-trace-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn concurrent_recorders_return_only_after_their_complete_record_is_written() {
        let path = path();
        let trace = Trace::create(&path).unwrap();
        let writers: Vec<_> = (0..3)
            .map(|actor| {
                let trace = trace.clone();
                std::thread::spawn(move || {
                    for sequence in 0..20 {
                        trace
                            .record("test", &json!({"actor":actor,"sequence":sequence}))
                            .unwrap();
                    }
                })
            })
            .collect();
        for writer in writers {
            writer.join().unwrap();
        }
        let records: Vec<Value> = fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(records.len(), 60);
        for actor in 0..3 {
            assert_eq!(
                records
                    .iter()
                    .filter(|event| event["message"]["actor"] == actor)
                    .map(|event| event["message"]["sequence"].as_u64().unwrap())
                    .collect::<Vec<_>>(),
                (0..20).collect::<Vec<_>>()
            );
        }
        assert!(records.windows(2).all(|pair| {
            pair[0]["elapsed_seconds"].as_f64().unwrap()
                <= pair[1]["elapsed_seconds"].as_f64().unwrap()
        }));
        drop(trace);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn a_write_failure_is_returned_and_the_writer_cannot_accept_later_records() {
        let path = path();
        File::create_new(&path).unwrap();
        let trace = Trace::from_file(File::open(&path).unwrap()).unwrap();
        assert!(trace.record("test", &json!(1)).is_err());
        assert!(trace.record("test", &json!(2)).is_err());
        assert!(fs::read(&path).unwrap().is_empty());
        drop(trace);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn archived_build_receipt_survives_moving_the_evidence_bundle() {
        let root = std::env::temp_dir().join(format!(
            "choreofix-build-receipt-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let input = root.join("input");
        let bundle = root.join("bundle");
        fs::create_dir_all(&input).unwrap();
        fs::create_dir(&bundle).unwrap();
        let binary = input.join("runtime");
        let manifest = input.join("original-source.sha256");
        let receipt = input.join("receipt.json");
        fs::write(&binary, b"qualified binary").unwrap();
        fs::write(&manifest, b"source hashes\n").unwrap();
        save(
            &receipt,
            &json!({"binary_sha256":digest(&binary).unwrap(),
            "source_unchanged":true,"source_manifest":"original-source.sha256",
            "source_manifest_sha256":digest(&manifest).unwrap()}),
        )
        .unwrap();
        let original_receipt = digest(&receipt).unwrap();
        let saved = archive_build_receipt(&binary, &receipt, &bundle).unwrap();
        let relocated = root.join("relocated");
        fs::rename(&bundle, &relocated).unwrap();
        fs::remove_dir_all(&input).unwrap();
        let recorded: Value =
            serde_json::from_slice(&fs::read(relocated.join("build-receipt.json")).unwrap())
                .unwrap();
        assert_eq!(recorded, saved);
        assert_eq!(recorded["source_manifest"], "build-source.sha256");
        assert_eq!(
            recorded["source_manifest_sha256"],
            digest(&relocated.join(recorded["source_manifest"].as_str().unwrap())).unwrap()
        );
        assert_eq!(recorded["input_receipt_sha256"], original_receipt);
        assert_eq!(
            digest(&relocated.join("input-build-receipt.json")).unwrap(),
            original_receipt
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mismatched_build_evidence_cannot_publish_an_archived_receipt() {
        let root = std::env::temp_dir().join(format!(
            "choreofix-bad-build-receipt-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let binary = root.join("runtime");
        let manifest = root.join("source.sha256");
        let receipt = root.join("input.json");
        let output = root.join("bundle");
        fs::create_dir(&output).unwrap();
        fs::write(&binary, b"qualified binary").unwrap();
        fs::write(&manifest, b"source hashes\n").unwrap();
        let valid = json!({"binary_sha256":digest(&binary).unwrap(),"source_unchanged":true,
            "source_manifest":"source.sha256","source_manifest_sha256":digest(&manifest).unwrap()});
        for invalid in [
            "binary_sha256",
            "source_manifest_sha256",
            "source_unchanged",
            "source_manifest",
        ] {
            let mut value = valid.clone();
            value[invalid] = if invalid == "source_unchanged" {
                false.into()
            } else {
                "different".into()
            };
            save(&receipt, &value).unwrap();
            assert!(archive_build_receipt(&binary, &receipt, &output).is_err());
            assert!(!output.join("build-receipt.json").exists());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_existing_build_archive_cannot_be_replaced_by_another_valid_receipt() {
        let root = std::env::temp_dir().join(format!(
            "choreofix-existing-receipt-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let binary = root.join("runtime");
        let manifest = root.join("source.sha256");
        let input = root.join("input.json");
        let output = root.join("bundle");
        fs::create_dir(&output).unwrap();
        fs::write(&binary, b"qualified binary").unwrap();
        fs::write(&manifest, b"original source hashes\n").unwrap();
        let mut receipt = json!({"binary_sha256":digest(&binary).unwrap(),
            "source_unchanged":true,"source_manifest":"source.sha256",
            "source_manifest_sha256":digest(&manifest).unwrap()});
        save(&input, &receipt).unwrap();
        archive_build_receipt(&binary, &input, &output).unwrap();
        let names = [
            "build-receipt.json",
            "input-build-receipt.json",
            "build-source.sha256",
        ];
        let original = names.map(|name| digest(&output.join(name)).unwrap());
        fs::write(&manifest, b"replacement source hashes\n").unwrap();
        receipt["source_manifest_sha256"] = digest(&manifest).unwrap().into();
        save(&input, &receipt).unwrap();
        assert!(archive_build_receipt(&binary, &input, &output).is_err());
        assert_eq!(
            original,
            names.map(|name| digest(&output.join(name)).unwrap())
        );
        fs::remove_dir_all(root).unwrap();
    }
}
