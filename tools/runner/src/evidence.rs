use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct Trace {
    file: Mutex<File>,
    started: Instant,
}
impl Trace {
    pub fn create(path: &Path) -> Result<Self> {
        Ok(Self {
            file: Mutex::new(File::create_new(path)?),
            started: Instant::now(),
        })
    }
    pub fn record(&self, direction: &str, message: &Value) -> Result<()> {
        let mut file = self.file.lock().map_err(|_| "trace lock poisoned")?;
        serde_json::to_writer(
            &mut *file,
            &json!({"elapsed_seconds":self.started.elapsed().as_secs_f64(), "unix_ns":SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos().to_string(), "direction":direction, "message":message}),
        )?;
        writeln!(file)?;
        file.flush()?;
        Ok(())
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
