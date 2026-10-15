//! Bash parsing and commands run inside the real WASI guest. ReadWriteFs is a
//! stateless std::fs adapter here: its calls are WASI imports, never host I/O.
#[cfg(not(target_os = "wasi"))]
compile_error!("This guest must run under WASI, not as a native host process.");

use rust_bash::{ReadWriteFs, RustBashBuilder};
use std::{
    io::{self, Read, Write},
    sync::Arc,
};

fn main() {
    match run() {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let mut shell = RustBashBuilder::new()
        .fs(Arc::new(ReadWriteFs::new()))
        .cwd("/")
        .build()?;
    let mut input = io::stdin().lock();
    loop {
        // A byte length frames each complete tool command, including multiline
        // scripts. Framing does not constrain its shell syntax or task effects.
        io::stdout().write_all(b"bash> ")?;
        io::stdout().flush()?;
        let mut size = [0; 4];
        input.read_exact(&mut size)?;
        let size = u32::from_le_bytes(size) as usize;
        let mut bytes = vec![0; size];
        input.read_exact(&mut bytes)?;
        let command = String::from_utf8(bytes)?;
        match shell.exec(&command) {
            Ok(result) => {
                if let Some(bytes) = result.stdout_bytes {
                    io::stdout().write_all(&bytes)?;
                } else {
                    io::stdout().write_all(result.stdout.as_bytes())?;
                }
                io::stderr().write_all(result.stderr.as_bytes())?;
                if shell.should_exit() {
                    return Ok(result.exit_code);
                }
            }
            Err(error) => writeln!(io::stderr(), "{error}")?,
        }
    }
}
