use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, WireEncode, WirePayload};
use serde_json::Value;

/// Canonical JSON with the original task's required string argument.
#[derive(Debug, PartialEq, Eq)]
pub struct WriteArguments(Vec<u8>);

impl WriteArguments {
    pub fn from_value(value: Value) -> ShellResult<Self> {
        if !valid_object(&value) {
            return Err(ShellError::message("expected a string argument: sections"));
        }
        serde_json::to_vec(&value)
            .map(Self)
            .map_err(|error| ShellError::message(error.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        let value = serde_json::from_slice(bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        Self::from_value(value)
    }

    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated canonical sections arguments")
    }
}

fn valid_object(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.get("sections").is_some_and(Value::is_string))
}

impl WireEncode for WriteArguments {
    fn encode_into(&self, output: &mut [u8]) -> Result<usize, CodecError> {
        let bytes = output
            .get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?;
        bytes.copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}

impl WirePayload for WriteArguments {
    const SCHEMA_ID: u32 = 0x5752_4901;
    type Decoded<'a> = Self;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        if !valid_object(&value)
            || serde_json::to_vec(&value).map_err(|_| CodecError::Malformed)? != input.as_bytes()
        {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(
        input: hibana::runtime::wire::ReceivedPayload<'a, Self>,
    ) -> Self::Decoded<'a> {
        Self(input.as_bytes().to_vec())
    }
}
