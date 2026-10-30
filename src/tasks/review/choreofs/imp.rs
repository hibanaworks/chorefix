//! Repository request data and the actual selected review target.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{
    resolver::{DecisionArm, ResolverError},
    wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload},
};
use serde_json::Value;
use std::cell::Cell;
pub struct RepositoryArguments<const OPERATION: u8>(Vec<u8>);
impl<const OPERATION: u8> RepositoryArguments<OPERATION> {
    const KEY: &'static str = match OPERATION {
        1 => "feature_name",
        2 => "branch",
        3 => "pr_id",
        _ => panic!("unknown repository operation"),
    };
    pub fn from_bytes(bytes: &[u8]) -> ShellResult<Self> {
        Self::validate_payload(Payload::new(bytes))
            .map_err(|e| ShellError::message(format!("{e:?}")))?;
        Ok(Self(bytes.to_vec()))
    }
    pub fn into_value(self) -> Value {
        serde_json::from_slice(&self.0).expect("validated repository arguments")
    }
}
impl<const OPERATION: u8> WireEncode for RepositoryArguments<OPERATION> {
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        out.get_mut(..self.0.len())
            .ok_or(CodecError::Truncated)?
            .copy_from_slice(&self.0);
        Ok(self.0.len())
    }
}
impl<const OPERATION: u8> WirePayload for RepositoryArguments<OPERATION> {
    const SCHEMA_ID: u32 = 0xac06_0000 | OPERATION as u32;
    type Decoded<'a> = Self;
    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let value: Value =
            serde_json::from_slice(input.as_bytes()).map_err(|_| CodecError::Malformed)?;
        let object = value.as_object().ok_or(CodecError::Malformed)?;
        if object.len() != 1 || !object.get(Self::KEY).is_some_and(Value::is_string) {
            return Err(CodecError::Malformed);
        }
        Ok(())
    }
    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self {
        Self(input.as_bytes().to_vec())
    }
}
#[derive(Default)]
pub struct ReviewTarget(Cell<Option<&'static str>>);
impl ReviewTarget {
    pub fn record(&self, feature: &'static str) {
        self.0.set(Some(feature));
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        match self.0.get() {
            Some(super::global::FEATURE_A) => Ok(DecisionArm::Left),
            Some(super::global::FEATURE_B) => Ok(DecisionArm::Right),
            _ => Err(ResolverError::reject()),
        }
    }
}
