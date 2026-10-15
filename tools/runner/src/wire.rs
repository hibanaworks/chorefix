//! JSON-line process transport. No task progress or permission decisions.
use crate::evidence::{Result, Trace};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines};

pub async fn send(output: &mut (impl AsyncWrite + Unpin), value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    output.write_all(&bytes).await?;
    output.flush().await?;
    Ok(())
}

pub struct Wire<W, R> {
    pub input: W,
    pub output: Lines<BufReader<R>>,
    pub trace: Trace,
}
impl<W: AsyncWrite + Unpin, R: AsyncRead + Unpin> Wire<W, R> {
    pub fn new(input: W, output: R, trace: Trace) -> Self {
        Self {
            input,
            output: BufReader::new(output).lines(),
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
