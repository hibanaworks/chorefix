//! Owned JSON-line I/O and external response delivery. No task permissions.
use crate::evidence::{Result, Trace};
use hibana_runtime::host::{Io, Notification};
use serde_json::Value;
use std::{io, sync::mpsc};

pub type Reply = (mpsc::Sender<(Value, Result<Value>)>, Notification);

pub fn respond(reply: &Reply, id: Value, result: Result<Value>) -> io::Result<()> {
    if reply.0.send((id, result)).is_err() {
        return Ok(()); // The caller cancelled; accepted effects are not reversed.
    }
    match reply.1.notify() {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub async fn receive<T>(input: &mpsc::Receiver<T>, ready: &mut Io) -> Result<Option<T>> {
    let mut bytes = [0; 64];
    loop {
        match input.try_recv() {
            Ok(value) => return Ok(Some(value)),
            Err(mpsc::TryRecvError::Disconnected) => return Ok(None),
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if ready.read(&mut bytes).await? == 0 {
            return match input.try_recv() {
                Ok(value) => Ok(Some(value)),
                Err(mpsc::TryRecvError::Disconnected) => Ok(None),
                Err(mpsc::TryRecvError::Empty) => {
                    Err("notification closed with a live producer".into())
                }
            };
        }
    }
}

pub async fn send(output: &mut Io, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    output.write_all(&bytes).await?;
    Ok(())
}

pub struct Lines {
    input: Io,
    bytes: Vec<u8>,
}

impl Lines {
    pub fn new(input: Io) -> Self {
        Self {
            input,
            bytes: Vec::new(),
        }
    }

    // Partial input belongs to the I/O owner, so cancelling a pending read does
    // not lose the prefix when another direction is selected by race().
    pub async fn next_line(&mut self) -> Result<Option<String>> {
        loop {
            if let Some(end) = self.bytes.iter().position(|byte| *byte == b'\n') {
                let mut line: Vec<u8> = self.bytes.drain(..=end).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some(String::from_utf8(line)?));
            }
            let mut bytes = [0; 4096];
            let read = self.input.read(&mut bytes).await?;
            if read == 0 {
                return if self.bytes.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(String::from_utf8(std::mem::take(&mut self.bytes))?))
                };
            }
            self.bytes.extend_from_slice(&bytes[..read]);
        }
    }
}

pub struct Wire {
    pub input: Io,
    pub output: Lines,
    pub trace: Trace,
}
impl Wire {
    pub fn new(input: Io, output: Io, trace: Trace) -> Self {
        Self {
            input,
            output: Lines::new(output),
            trace,
        }
    }
    pub async fn send(&mut self, message: &Value) -> Result<()> {
        self.trace.record("send", message)?;
        send(&mut self.input, message).await
    }
    pub async fn next(&mut self) -> Result<Value> {
        let line = self
            .output
            .next_line()
            .await?
            .ok_or("model process closed before completion")?;
        let value = serde_json::from_str(&line)?;
        self.trace.record("receive", &value)?;
        Ok(value)
    }
    pub async fn request(&mut self, id: u64, method: &str, params: Value) -> Result<Value> {
        self.send(&serde_json::json!({"id":id, "method":method, "params":params}))
            .await?;
        loop {
            let message = self.next().await?;
            if message.get("id") == Some(&Value::from(id)) && message.get("method").is_none() {
                if let Some(error) = message.get("error") {
                    return Err(format!("RPC {method}: {error}").into());
                }
                return message
                    .get("result")
                    .cloned()
                    .ok_or_else(|| "RPC result missing".into());
            }
            if message.get("id").is_some() && message.get("method").is_some() {
                self.send(&serde_json::json!({"id":message["id"], "error":{"code":-32601,"message":"Only supplied experiment tools are available"}})).await?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hibana_runtime::host::{deadline, sleep};
    use std::{io::Write, os::unix::net::UnixStream, time::Duration};

    #[hibana_runtime::main]
    #[test]
    async fn cancelling_a_partial_line_read_preserves_its_prefix_and_following_lines() -> Result<()>
    {
        let (mut writer, reader) = UnixStream::pair()?;
        writer.write_all(b"{\"id\":")?;
        let mut input = Lines::new(Io::new(reader.into())?);
        let mut writer = Io::new(writer.into())?;
        async {
            let error = hibana_runtime::race(input.next_line(), deadline(Duration::from_millis(1)))
                .await
                .unwrap_err();
            assert_eq!(
                error.downcast_ref::<io::Error>().unwrap().kind(),
                io::ErrorKind::TimedOut
            );
            assert_eq!(input.next_line().await?.as_deref(), Some("{\"id\":7}"));
            assert_eq!(input.next_line().await?.as_deref(), Some("second"));
            assert_eq!(input.next_line().await?.as_deref(), Some("unterminated"));
            assert_eq!(input.next_line().await?, None);
        };
        async move {
            sleep(Duration::from_millis(10)).await?;
            writer.write_all(b"7}\nsecond\r\nunterminated").await?;
        };
        Ok(())
    }
}
