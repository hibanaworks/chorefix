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
}
