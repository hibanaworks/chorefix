use super::bus::{Bus, Terminal};
use crate::shell::{ShellError, ShellResult};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, FdSpec, ObjectId},
    exchange::FdBindingTable,
    protocol::{self, FdBinding, FdReadRow, FdReaddirRow, FdWriteRow},
};
use std::{cell::RefCell, rc::Rc};

const READ: u64 = 1 << 1;
const WRITE: u64 = 1 << 6;
const READDIR: u64 = 1 << 14;
const READ_WRITE: FdBinding = FdBinding {
    read: Some(FdReadRow::Base),
    write: Some(FdWriteRow::Base),
    readdir: None,
};
const WORK_PATHS: [&[u8]; 8] = [
    b"work/0", b"work/1", b"work/2", b"work/3", b"work/4", b"work/5", b"work/6", b"work/7",
];

const fn listing() -> [u8; 200] {
    let mut listing = [0; 200];
    let mut index = 0;
    while index < 8 {
        let base = index * 25;
        // d_next is the byte offset used by this fixed directory's cookie.
        listing[base] = ((index + 1) * 25) as u8;
        listing[base + 8] = (index + 6) as u8;
        listing[base + 16] = 1;
        listing[base + 20] = protocol::WASIP1_FILETYPE_REGULAR_FILE;
        listing[base + 24] = b'0' + index as u8;
        index += 1;
    }
    listing
}
const WORK_LISTING: [u8; 200] = listing();

// File contents, offsets and flags are filesystem data, never task progress or
// ownership authority. All file operations consume ChoreoFS operation results.
pub struct PrivateFs<'a> {
    input: Vec<u8>,
    reply: Rc<RefCell<Vec<u8>>>,
    work: [Vec<u8>; 8],
    offsets: [usize; 15],
    fd_flags: [u16; 15],
    output: Vec<u8>,
    pub terminal: Terminal,
    bus: &'a Bus,
}

impl<'a> PrivateFs<'a> {
    pub fn new(terminal: Terminal, bus: &'a Bus, reply: Rc<RefCell<Vec<u8>>>) -> Self {
        Self {
            input: Vec::new(),
            reply,
            work: std::array::from_fn(|_| Vec::new()),
            offsets: [0; 15],
            fd_flags: [0; 15],
            output: Vec::new(),
            terminal,
            bus,
        }
    }
}

impl PrivateFs<'_> {
    fn objects<'b>(&'b self, reply: &'b [u8]) -> ChoreoFsObjectSet<'b, 14> {
        ChoreoFsObjectSet::new([
            ChoreoFsObject::readable(
                b"llm/stdin",
                ObjectId(0),
                FdSpec::new(0, READ, 1),
                &self.input,
                FdBinding::read(FdReadRow::Base),
            ),
            ChoreoFsObject::writable(
                b"llm/stdout",
                ObjectId(1),
                FdSpec::new(1, WRITE, 1),
                FdBinding::write(FdWriteRow::Base),
            ),
            ChoreoFsObject::writable(
                b"llm/stderr",
                ObjectId(2),
                FdSpec::new(2, WRITE, 1),
                FdBinding::write(FdWriteRow::Base),
            ),
            ChoreoFsObject::readdir(
                b"",
                ObjectId(3),
                FdSpec::new(3, READDIR, 1),
                b"",
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            ChoreoFsObject::readable(
                b"service/result",
                ObjectId(5),
                FdSpec::new(5, READ, 1),
                reply,
                FdBinding::read(FdReadRow::Base),
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[0],
                ObjectId(6),
                FdSpec::new(6, READ | WRITE, 1),
                &self.work[0],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[1],
                ObjectId(7),
                FdSpec::new(7, READ | WRITE, 1),
                &self.work[1],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[2],
                ObjectId(8),
                FdSpec::new(8, READ | WRITE, 1),
                &self.work[2],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[3],
                ObjectId(9),
                FdSpec::new(9, READ | WRITE, 1),
                &self.work[3],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[4],
                ObjectId(10),
                FdSpec::new(10, READ | WRITE, 1),
                &self.work[4],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[5],
                ObjectId(11),
                FdSpec::new(11, READ | WRITE, 1),
                &self.work[5],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[6],
                ObjectId(12),
                FdSpec::new(12, READ | WRITE, 1),
                &self.work[6],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                WORK_PATHS[7],
                ObjectId(13),
                FdSpec::new(13, READ | WRITE, 1),
                &self.work[7],
                READ_WRITE,
            ),
            ChoreoFsObject::readdir(
                b"work",
                ObjectId(14),
                FdSpec::new(14, READDIR, 1),
                &WORK_LISTING,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
        ])
    }

    pub async fn read(&mut self, read: protocol::FdRead) -> ShellResult<protocol::FdReadDoneRet> {
        if read.fd() == 0 && self.offsets[0] == self.input.len() {
            let command = self
                .terminal
                .read_command(self.bus, std::mem::take(&mut self.output))
                .await?;
            if command.len() > 4095 || command.as_bytes().contains(&0) {
                return Err(ShellError::message(
                    "shell command exceeds guest input bound",
                ));
            }
            self.input = command.as_bytes().to_vec();
            self.input.push(b'\n');
            self.offsets[0] = 0;
        }
        let index = read.fd() as usize;
        let offset = self.offsets.get(index).copied().unwrap_or(0);
        let (response, next) = self
            .objects(&self.reply.borrow())
            .choreofs()
            .fd_read(read)
            .read_from(offset)?;
        if let Some(offset) = self.offsets.get_mut(index) {
            *offset = next;
        }
        Ok(response)
    }
    pub async fn write(
        &mut self,
        write: protocol::FdWrite,
    ) -> ShellResult<protocol::FdWriteDoneRet> {
        let operation = self
            .objects(&self.reply.borrow())
            .choreofs()
            .fd_write(write);
        if !operation.is_writable() {
            return Ok(operation.written());
        }
        match operation.object() {
            Some(ObjectId(1 | 2)) => self.output.extend_from_slice(operation.bytes()),
            Some(ObjectId(fd @ 6..=13)) => {
                let fd = fd as usize;
                let file = &mut self.work[fd - 6];
                let start = if self.fd_flags[fd] & 1 != 0 {
                    file.len()
                } else {
                    self.offsets[fd]
                };
                let Some(end) = start
                    .checked_add(operation.bytes().len())
                    .filter(|end| *end <= 8192)
                else {
                    return Ok(protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(write.fd(), 0, 22),
                    ));
                };
                if end > file.len() {
                    file.resize(end, 0);
                }
                file[start..end].copy_from_slice(operation.bytes());
                self.offsets[fd] = end;
            }
            _ => {
                return Err(ShellError::message(
                    "writable object has no answering owner",
                ));
            }
        }
        Ok(operation.written())
    }
    pub fn open(&mut self, request: protocol::PathOpen) -> ShellResult<protocol::PathOpenedRet> {
        if request.preopen_fd() != 3 {
            return Err(ShellError::message("non-root path_open"));
        }
        let path = normalize(request.path());
        let request = protocol::PathOpen::new(3, request.rights_base(), path)?
            .with_flags(request.open_flags() as u32, request.fd_flags() as u32)?;
        let open = self
            .objects(&self.reply.borrow())
            .choreofs()
            .path_open(request);
        if let Some(fd) = open.fd() {
            let fd = fd as usize;
            if request.open_flags() & 8 != 0 {
                match open.object() {
                    Some(ObjectId(6..=13)) => self.work[fd - 6].clear(),
                    _ => {
                        return Err(ShellError::message(
                            "truncate is only supported on work files",
                        ));
                    }
                }
            }
            self.offsets[fd] = 0;
            self.fd_flags[fd] = request.fd_flags();
        }
        Ok(open.opened_ret())
    }
    pub fn stat(&self, request: protocol::FdRequest) -> protocol::FdStatRet {
        self.objects(&self.reply.borrow())
            .choreofs()
            .fd_fdstat_get(request)
    }
    pub fn file_stat(&self, request: protocol::FdRequest) -> protocol::FdFilestatRet {
        self.objects(&self.reply.borrow())
            .choreofs()
            .fd_filestat_get(request)
    }
    pub fn path_stat(
        &self,
        request: protocol::PathFilestatGet,
    ) -> ShellResult<protocol::PathFilestatRet> {
        if request.preopen_fd() != 3 {
            return Err(ShellError::message("non-root path_filestat_get"));
        }
        let request =
            protocol::PathFilestatGet::new(3, request.flags(), normalize(request.path()))?;
        Ok(self
            .objects(&self.reply.borrow())
            .choreofs()
            .path_filestat_get(request))
    }
    pub fn read_dir(
        &self,
        request: protocol::FdReaddir,
    ) -> ShellResult<protocol::FdReaddirDoneRet> {
        Ok(self
            .objects(&self.reply.borrow())
            .choreofs()
            .fd_readdir(request)
            .read_dir()?)
    }
    pub fn close(&self, request: protocol::FdRequest) -> protocol::FdClosedRet {
        self.objects(&self.reply.borrow())
            .choreofs()
            .fd_close(request)
    }
}
fn normalize(path: &[u8]) -> &[u8] {
    match path.strip_prefix(b"/") {
        Some(path) => path,
        None => path,
    }
}
pub fn bindings() -> FdBindingTable {
    let mut bindings = FdBindingTable::empty();
    for (fd, binding) in [
        (0, FdBinding::read(FdReadRow::Base)),
        (1, FdBinding::write(FdWriteRow::Base)),
        (2, FdBinding::write(FdWriteRow::Base)),
        (3, FdBinding::readdir(FdReaddirRow::Base)),
    ] {
        bindings
            .bind_fd(fd, binding)
            .expect("four fixed initial bindings fit");
    }
    bindings
}
