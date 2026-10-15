use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, FdSpec, ObjectId},
    protocol,
};
use serde_json::Value;
use std::cell::RefCell;
use tokio::sync::{mpsc, oneshot};

/// Observed WASI requests and their completion channels, not protocol stages.
pub enum Import {
    Open(
        protocol::PathOpenReq,
        oneshot::Sender<protocol::PathOpenedRet>,
    ),
    Write(
        protocol::FdWriteReq,
        oneshot::Sender<protocol::FdWriteDoneRet>,
    ),
    Close(
        protocol::FdRequest,
        oneshot::Sender<(protocol::FdClosedRet, Vec<u8>)>,
    ),
}
pub type Imports = mpsc::UnboundedReceiver<Import>;

pub trait Domain {
    fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> impl Future<Output = Result<Value, String>>;
}

/// The resolver reads only the actual validation result for this close.
pub struct Validation(RefCell<Option<ShellResult<Value>>>);
impl Validation {
    pub fn new() -> Self {
        Self(RefCell::new(None))
    }
    pub fn record(&self, result: ShellResult<Value>) {
        self.0.replace(Some(result));
    }
    pub fn take(&self) -> ShellResult<Value> {
        self.0
            .borrow_mut()
            .take()
            .ok_or_else(|| ShellError::message("missing validation result"))?
    }
    pub fn rejection(&self) -> ShellResult<String> {
        match self.0.borrow().as_ref() {
            Some(Err(error)) => Ok(error.to_string()),
            _ => Err(ShellError::message("missing argument rejection")),
        }
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        match self.0.borrow().as_ref() {
            Some(Ok(_)) => Ok(DecisionArm::Left),
            Some(Err(_)) => Ok(DecisionArm::Right),
            None => Err(ResolverError::reject()),
        }
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
    pub bytes: Vec<u8>,
}
impl File {
    pub fn new(path: &'static [u8]) -> Self {
        Self {
            path,
            bytes: Vec::new(),
        }
    }
    pub fn objects(&self) -> ChoreoFsObjectSet<'static, 1> {
        ChoreoFsObjectSet::new([ChoreoFsObject::writable(
            self.path,
            ObjectId(4),
            FdSpec::new(4, 1 << 6, 1),
            protocol::FdBinding::write(protocol::FdWriteRow::Object),
        )])
    }
    pub fn open(&mut self, request: protocol::PathOpenReq) -> protocol::PathOpenedRet {
        let operation = self.objects().choreofs().path_open(request.0);
        if operation.is_open() && request.0.open_flags() & 8 != 0 {
            self.bytes.clear();
        }
        operation.opened_ret()
    }
    pub fn write(&mut self, request: protocol::FdWriteReq) -> protocol::FdWriteDoneRet {
        let operation = self.objects().choreofs().fd_write(request.0);
        if operation.is_writable() {
            if self.bytes.len() + operation.bytes().len() > 8192 {
                return protocol::FdWriteDoneRet(protocol::FdWriteDone::new_with_errno(
                    request.0.fd(),
                    0,
                    22,
                ));
            }
            self.bytes.extend_from_slice(operation.bytes());
        }
        operation.written()
    }
    pub fn arguments(&self, request: protocol::FdRequest, key: &str) -> ShellResult<Value> {
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
