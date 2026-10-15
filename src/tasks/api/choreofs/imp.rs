//! ChoreoFS object bytes and offsets, without task progress or permission state.
use super::global::{OUTCOME_FD, OUTCOME_PATH};
use crate::shell::ShellResult;
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, FdSpec, ObjectId},
    protocol,
};
use std::{cell::RefCell, rc::Rc};
pub(crate) fn outcome_objects(bytes: &[u8]) -> ChoreoFsObjectSet<'_, 1> {
    ChoreoFsObjectSet::new([ChoreoFsObject::readable(
        OUTCOME_PATH,
        ObjectId(u32::from(OUTCOME_FD)),
        FdSpec::new(OUTCOME_FD, 1 << 1, 1),
        bytes,
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )])
}
pub(crate) struct OutcomeFile {
    pub bytes: Rc<RefCell<Vec<u8>>>,
    offset: usize,
}
impl OutcomeFile {
    pub fn new(bytes: Rc<RefCell<Vec<u8>>>) -> Self {
        Self { bytes, offset: 0 }
    }
    pub fn open(&mut self, request: protocol::PathOpenReq) -> ShellResult<protocol::PathOpenedRet> {
        let r = request.0;
        let path = r.path().strip_prefix(b"/").unwrap_or(r.path());
        let normalized = protocol::PathOpen::new(r.preopen_fd(), r.rights_base(), path)?
            .with_flags(r.open_flags() as u32, r.fd_flags() as u32)?;
        let bytes = self.bytes.borrow();
        let objects = outcome_objects(&bytes);
        let op = objects.choreofs().path_open(normalized);
        if op.is_open() {
            self.offset = 0
        }
        Ok(op.opened_ret())
    }
    pub fn read(&mut self, request: protocol::FdReadReq) -> ShellResult<protocol::FdReadDoneRet> {
        let (response, next) = outcome_objects(&self.bytes.borrow())
            .choreofs()
            .fd_read(request.0)
            .read_from(self.offset)?;
        self.offset = next;
        Ok(response)
    }
    pub fn stat(&self, request: protocol::FdFdstatGetReq) -> protocol::FdStatRet {
        outcome_objects(&self.bytes.borrow())
            .choreofs()
            .fd_fdstat_get(request.0)
    }
    pub fn file_stat(&self, request: protocol::FdFilestatGetReq) -> protocol::FdFilestatRet {
        outcome_objects(&self.bytes.borrow())
            .choreofs()
            .fd_filestat_get(request.0)
    }
    pub fn close(&self, request: protocol::FdCloseReq) -> protocol::FdClosedRet {
        outcome_objects(&self.bytes.borrow())
            .choreofs()
            .fd_close(request.0)
    }
}
