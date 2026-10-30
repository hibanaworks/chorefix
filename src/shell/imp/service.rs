use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{
    resolver::{DecisionArm, ResolverError},
    wire::ReceivedPayload,
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsRequest, FdSpec, ObjectId},
    protocol,
};
use serde_json::Value;
use std::cell::Cell;

pub trait Domain {
    fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> impl Future<Output = Result<Value, String>>;
}

/// The resolver reads only the actual validation result for this close.
pub struct Validation(Cell<Option<ShellResult<Value>>>);
impl Validation {
    pub fn new() -> Self {
        Self(Cell::new(None))
    }
    pub fn record(&self, result: ShellResult<Value>) {
        self.0.replace(Some(result));
    }
    pub fn take(&self) -> ShellResult<Value> {
        self.0
            .take()
            .ok_or_else(|| ShellError::message("missing validation result"))?
    }
    pub fn rejection(&self) -> ShellResult<String> {
        let result = self.0.take();
        let rejection = match result.as_ref() {
            Some(Err(error)) => Ok(error.to_string()),
            _ => Err(ShellError::message("missing argument rejection")),
        };
        self.0.set(result);
        rejection
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        let result = self.0.take();
        let decision = match result.as_ref() {
            Some(Ok(_)) => Ok(DecisionArm::Left),
            Some(Err(_)) => Ok(DecisionArm::Right),
            None => Err(ResolverError::reject()),
        };
        self.0.set(result);
        decision
    }
}
impl Default for Validation {
    fn default() -> Self {
        Self::new()
    }
}

/// Object bytes belong to the service owner. This has no expected-step or
/// resource-owner state. Only the owner's projected localside admits effects.
pub struct File {
    path: &'static [u8],
    fd: u8,
    pub bytes: Vec<u8>,
}
impl File {
    pub fn new(path: &'static [u8], fd: u8) -> Self {
        Self {
            path,
            fd,
            bytes: Vec::new(),
        }
    }
    pub fn objects(&self) -> ChoreoFsObjectSet<'static, 1> {
        ChoreoFsObjectSet::new([ChoreoFsObject::writable(
            self.path,
            ObjectId(self.fd as u32),
            FdSpec::new(self.fd, 1 << 6, 1),
            protocol::FdBinding::write(protocol::FdWriteRow::Object),
        )])
    }
    pub fn open(
        &mut self,
        request: ReceivedPayload<'_, protocol::PathOpenReq>,
    ) -> protocol::PathOpenedRet {
        let truncate = protocol::PathOpenReq::request(&request).0.open_flags() & 8 != 0;
        let operation = self.objects().choreofs().path_open(request);
        if operation.is_open() && truncate {
            self.bytes.clear();
        }
        operation.opened_ret()
    }
    pub fn write<P>(&mut self, request: ReceivedPayload<'_, P>) -> protocol::FdWriteDoneRet
    where
        P: ChoreoFsRequest<Request = (protocol::FdWrite, protocol::FdWriteRow)>,
    {
        let operation = self.objects().choreofs().fd_write(request);
        if operation.is_writable() {
            if self.bytes.len() + operation.bytes().len() > 8192 {
                return protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(
                    operation.request().fd(),
                    0,
                    22,
                ));
            }
            self.bytes.extend_from_slice(operation.bytes());
        }
        operation.written()
    }
    pub fn arguments(
        &self,
        request: ReceivedPayload<'_, protocol::FdCloseReq>,
        key: &str,
    ) -> ShellResult<Value> {
        let response = self.objects().choreofs().fd_close(request);
        if response.0.errno() != 0 {
            return Err(ShellError::message("invalid service fd"));
        }
        let value: Value = serde_json::from_slice(&self.bytes)
            .map_err(|error| ShellError::message(error.to_string()))?;
        let Some(object) = value.as_object() else {
            return Err(ShellError::message("expected an argument object"));
        };
        if object.len() != 1 || !object.get(key).is_some_and(Value::is_string) {
            return Err(ShellError::message(format!(
                "expected one string argument: {key}"
            )));
        }
        Ok(value)
    }
}
