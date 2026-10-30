//! File contents carried by the private I/O protocol.
use crate::shell::{ShellError, ShellResult};
use hibana::{
    g,
    runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
};
use hibana_wasip1_runtime::{choreofs::ChoreoFsRequest, protocol};
use serde_json::Value;

pub const FILE_BYTES: usize = 8192;
pub const SERVICE_FD: u8 = 4;
pub const RESULT_FD: u8 = 5;
pub const RESULT_CHUNK_LABEL: u8 = 241;
pub type ClosedFileMsg = g::Msg<{ protocol::LABEL_WASI_FD_CLOSE_RET }, ClosedFile>;
pub type ResultChunkMsg = g::Msg<RESULT_CHUNK_LABEL, ResultChunk>;

/// The close result and, for the argument file, its actual pathname and bytes.
pub struct ClosedFile {
    pub completion: protocol::FdClosedRet,
    pub path: Vec<u8>,
    pub bytes: Vec<u8>,
}

impl ClosedFile {
    pub fn arguments(&self, expected_path: &[u8], key: &str) -> ShellResult<Value> {
        if self.completion.0.errno() != 0 || self.path != expected_path {
            return Err(ShellError::message(
                "service file does not match the request",
            ));
        }
        let value: Value = serde_json::from_slice(&self.bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        let object = value
            .as_object()
            .ok_or_else(|| ShellError::message("expected an argument object"))?;
        if object.len() != 1 || !object.get(key).is_some_and(Value::is_string) {
            return Err(ShellError::message(format!(
                "expected one string argument: {key}"
            )));
        }
        Ok(value)
    }
}

impl WireEncode for ClosedFile {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        if self.path.len() > protocol::WASIP1_PATH_CHUNK_CAPACITY || self.bytes.len() > FILE_BYTES {
            return Err(CodecError::Malformed);
        }
        if self.completion.0.fd() != SERVICE_FD && (!self.path.is_empty() || !self.bytes.is_empty())
        {
            return Err(CodecError::Malformed);
        }
        let len = 4 + self.path.len() + self.bytes.len();
        let out = out.get_mut(..len).ok_or(CodecError::Truncated)?;
        out[0] = self.completion.0.fd();
        out[1..3].copy_from_slice(&self.completion.0.errno().to_be_bytes());
        out[3] = self.path.len() as u8;
        out[4..4 + self.path.len()].copy_from_slice(&self.path);
        out[4 + self.path.len()..].copy_from_slice(&self.bytes);
        Ok(len)
    }
}
impl WirePayload for ClosedFile {
    const SCHEMA_ID: u32 = 0xcf00_0001;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() < 4 {
            return Err(CodecError::Truncated);
        }
        let path_len = bytes[3] as usize;
        if path_len > protocol::WASIP1_PATH_CHUNK_CAPACITY
            || bytes.len() < 4 + path_len
            || bytes.len() - 4 - path_len > FILE_BYTES
        {
            return Err(CodecError::Malformed);
        }
        if bytes[0] != SERVICE_FD && bytes.len() != 4 {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        let bytes = input.as_bytes();
        let path_end = 4 + bytes[3] as usize;
        Self {
            completion: protocol::FdClosedRet(protocol::FdClosed::new_with_errno(
                bytes[0],
                u16::from_be_bytes([bytes[1], bytes[2]]),
            )),
            path: bytes[4..path_end].to_vec(),
            bytes: bytes[path_end..].to_vec(),
        }
    }
}

/// One write to the guest's result file, at an explicit byte offset.
pub struct ResultChunk {
    offset: usize,
    write: protocol::FdWrite,
}
impl ResultChunk {
    pub fn new(offset: usize, bytes: &[u8]) -> ShellResult<Self> {
        if offset
            .checked_add(bytes.len())
            .is_none_or(|end| end > FILE_BYTES)
        {
            return Err(ShellError::message("result file exceeds its byte bound"));
        }
        Ok(Self {
            offset,
            write: protocol::FdWrite::new(RESULT_FD, bytes)?,
        })
    }
    pub fn offset(input: &ReceivedPayload<'_, Self>) -> usize {
        let bytes = input.as_bytes();
        u16::from_be_bytes([bytes[0], bytes[1]]) as usize
    }
}
impl WireEncode for ResultChunk {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let len = 2 + self.write.as_bytes().len();
        let out = out.get_mut(..len).ok_or(CodecError::Truncated)?;
        out[..2].copy_from_slice(&(self.offset as u16).to_be_bytes());
        out[2..].copy_from_slice(self.write.as_bytes());
        Ok(len)
    }
}
impl WirePayload for ResultChunk {
    const SCHEMA_ID: u32 = 0xcf00_0002;
    type Decoded<'a> = ReceivedPayload<'a, Self>;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let bytes = input.as_bytes();
        if bytes.len() < 2 {
            return Err(CodecError::Truncated);
        }
        let offset = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
        if bytes.len() - 2 > protocol::WASIP1_IO_CHUNK_CAPACITY
            || offset + bytes.len() - 2 > FILE_BYTES
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}
impl ChoreoFsRequest for ResultChunk {
    type Request = (protocol::FdWrite, protocol::FdWriteRow);
    fn request(input: &ReceivedPayload<'_, Self>) -> Self::Request {
        (
            protocol::FdWrite::new(RESULT_FD, &input.as_bytes()[2..])
                .expect("validated result chunk"),
            protocol::FdWriteRow::Base,
        )
    }
}
