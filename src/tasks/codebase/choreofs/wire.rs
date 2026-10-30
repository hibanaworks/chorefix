//! Canonical service requests. Validation reads borrowed bytes without allocating.
use super::{global::SERVICE_FD, input::assignment};
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::{CodecError, Payload, ReceivedPayload, WireEncode, WirePayload};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsRequest, FdSpec, ObjectId},
    protocol,
};
use serde_json::Value;

pub const DESIGN: u8 = 1;
pub const IMPLEMENT: u8 = 2;
pub const COMMIT: u8 = 3;
pub const TEST: u8 = 4;

/// Actual fd and normalized input bytes, never an expected step or permission.
pub struct Close<const TASK: u8, const ROLE: u8, const OPERATION: u8> {
    fd: u8,
    arguments: Vec<u8>,
}

impl<const TASK: u8, const ROLE: u8, const OPERATION: u8> Close<TASK, ROLE, OPERATION> {
    pub const PATH: &'static [u8] = match OPERATION {
        DESIGN => b"service/design_feature",
        IMPLEMENT => b"service/implement_code",
        COMMIT => b"service/commit_changes",
        TEST => b"service/run_local_tests",
        _ => panic!("unknown service request kind"),
    };
    pub const KEY: &'static str = if OPERATION == COMMIT {
        "modules"
    } else {
        "feature_name"
    };

    pub fn objects() -> ChoreoFsObjectSet<'static, 1> {
        ChoreoFsObjectSet::new([ChoreoFsObject::writable(
            Self::PATH,
            ObjectId(SERVICE_FD as u32),
            FdSpec::new(SERVICE_FD, 1 << 6, 1),
            protocol::FdBinding::write(protocol::FdWriteRow::Object),
        )])
    }

    /// ABI data conversion happens before sending a projected request. Invalid
    /// command data returns to the shell without faulting or advancing Endpoint.
    pub fn new(request: protocol::FdCloseReq, mut arguments: Value) -> ShellResult<Self> {
        let expected = assignment::<TASK, ROLE>();
        if request.0.fd() != SERVICE_FD {
            return Err(ShellError::message("invalid service fd"));
        }
        let object = arguments
            .as_object_mut()
            .ok_or_else(|| ShellError::message("expected an argument object"))?;
        if object.len() != 1 {
            return Err(ShellError::message("expected one argument"));
        }
        let value = object
            .get_mut(Self::KEY)
            .ok_or_else(|| ShellError::message("incorrect argument name"))?;
        let text = value
            .as_str()
            .ok_or_else(|| ShellError::message("expected a string argument"))?;
        if OPERATION == COMMIT {
            let modules: Vec<_> = text.split(',').map(str::trim).collect();
            if modules.len() != 2
                || !(modules == expected.modules
                    || modules == [expected.modules[1], expected.modules[0]])
            {
                return Err(ShellError::message("arguments name different task modules"));
            }
            *value = Value::String(modules.join(","));
        } else if text != expected.feature {
            return Err(ShellError::message(
                "arguments name a different task feature",
            ));
        }
        let arguments = serde_json::to_vec(&arguments)
            .map_err(|error| ShellError::message(error.to_string()))?;
        Ok(Self {
            fd: request.0.fd(),
            arguments,
        })
    }
}

impl<const TASK: u8, const ROLE: u8, const OPERATION: u8> WireEncode
    for Close<TASK, ROLE, OPERATION>
{
    fn encode_into(&self, out: &mut [u8]) -> Result<usize, CodecError> {
        let len = self.arguments.len() + 1;
        if out.len() < len {
            return Err(CodecError::Truncated);
        }
        out[0] = self.fd;
        out[1..len].copy_from_slice(&self.arguments);
        Ok(len)
    }
}

impl<const TASK: u8, const ROLE: u8, const OPERATION: u8> WirePayload
    for Close<TASK, ROLE, OPERATION>
{
    const SCHEMA_ID: u32 = {
        let _ = assignment::<TASK, ROLE>();
        assert!(
            OPERATION >= DESIGN && OPERATION <= TEST,
            "unknown request kind"
        );
        // Design, implementation and testing share the same feature-data
        // format. Their projected message identities enforce their order.
        let format = if OPERATION == COMMIT { 2 } else { 1 };
        0xce00_0000 | ((TASK as u32) << 12) | ((ROLE as u32) << 4) | format
    };
    type Decoded<'a> = ReceivedPayload<'a, Self>;

    fn validate_payload(input: Payload<'_>) -> Result<(), CodecError> {
        let expected = assignment::<TASK, ROLE>();
        let (fd, arguments) = input
            .as_bytes()
            .split_first()
            .ok_or(CodecError::Truncated)?;
        if *fd != SERVICE_FD {
            return Err(CodecError::Malformed);
        }
        if OPERATION == COMMIT {
            let modules = arguments
                .strip_prefix(b"{\"modules\":\"")
                .and_then(|value| value.strip_suffix(b"\"}"))
                .ok_or(CodecError::Malformed)?;
            let comma = modules
                .iter()
                .position(|byte| *byte == b',')
                .ok_or(CodecError::Malformed)?;
            let first = &modules[..comma];
            let second = &modules[comma + 1..];
            if !((first == expected.modules[0].as_bytes()
                && second == expected.modules[1].as_bytes())
                || (first == expected.modules[1].as_bytes()
                    && second == expected.modules[0].as_bytes()))
            {
                return Err(CodecError::Malformed);
            }
        } else {
            let feature = arguments
                .strip_prefix(b"{\"feature_name\":\"")
                .and_then(|value| value.strip_suffix(b"\"}"))
                .ok_or(CodecError::Malformed)?;
            if feature != expected.feature.as_bytes() {
                return Err(CodecError::Malformed);
            }
        }
        Ok(())
    }

    fn decode_validated_payload<'a>(input: ReceivedPayload<'a, Self>) -> Self::Decoded<'a> {
        input
    }
}

impl<const TASK: u8, const ROLE: u8, const OPERATION: u8> ChoreoFsRequest
    for Close<TASK, ROLE, OPERATION>
{
    type Request = protocol::FdCloseReq;
    fn request(input: &ReceivedPayload<'_, Self>) -> Self::Request {
        protocol::FdCloseReq(protocol::FdRequest::new(input.as_bytes()[0]))
    }
}
