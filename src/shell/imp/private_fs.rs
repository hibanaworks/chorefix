use super::{
    bus::{Bus, Terminal},
    file_wire::{ClosedFile, FILE_BYTES, RESULT_FD, ResultChunk, SERVICE_FD},
};
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::wire::ReceivedPayload;
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, ChoreoFsObjectSet, ChoreoFsRequest, FdSpec, ObjectId},
    exchange::FdBindingTable,
    protocol::{self, FdBinding, FdReadRow, FdReaddirRow, FdWriteRow},
};

const READ: u64 = 1 << 1;
const WRITE: u64 = 1 << 6;
const READDIR: u64 = 1 << 14;
const READ_WRITE: FdBinding = FdBinding {
    read: Some(FdReadRow::Base),
    write: Some(FdWriteRow::Base),
    readdir: None,
};
// Unbound slots have unrepresentable guest paths. They are not files or
// permissions. A successful private create binds a caller-chosen name.
const UNBOUND_PATHS: [&[u8]; 8] = [
    b"\0slot0", b"\0slot1", b"\0slot2", b"\0slot3", b"\0slot4", b"\0slot5", b"\0slot6", b"\0slot7",
];

// File contents, offsets and flags are filesystem data, never task progress or
// ownership authority. All file operations consume ChoreoFS operation results.
pub struct PrivateFs<'a> {
    input: Vec<u8>,
    reply: Vec<u8>,
    service_path: Vec<u8>,
    service_bytes: Vec<u8>,
    work: [Vec<u8>; 8],
    paths: [Vec<u8>; 8],
    listing: Vec<u8>,
    offsets: [usize; 16],
    fd_flags: [u16; 16],
    pub output: Vec<u8>,
    pub terminal: Terminal,
    bus: &'a Bus,
}

impl<'a> PrivateFs<'a> {
    pub fn new(terminal: Terminal, bus: &'a Bus) -> Self {
        Self {
            input: Vec::new(),
            reply: Vec::new(),
            service_path: b"\0arguments".to_vec(),
            service_bytes: Vec::new(),
            work: std::array::from_fn(|_| Vec::new()),
            paths: std::array::from_fn(|_| Vec::new()),
            listing: Vec::new(),
            offsets: [0; 16],
            fd_flags: [0; 16],
            output: Vec::new(),
            terminal,
            bus,
        }
    }
}

impl PrivateFs<'_> {
    fn work_path(&self, index: usize) -> &[u8] {
        if self.paths[index].is_empty() {
            UNBOUND_PATHS[index]
        } else {
            &self.paths[index]
        }
    }
    fn refresh_listing(&mut self) {
        self.listing.clear();
        for (index, path) in self.paths.iter().enumerate() {
            let Some(name) = path.strip_prefix(b"work/") else {
                continue;
            };
            let next = (self.listing.len() + 24 + name.len()) as u64;
            self.listing.extend_from_slice(&next.to_le_bytes());
            self.listing
                .extend_from_slice(&(index as u64 + 6).to_le_bytes());
            self.listing
                .extend_from_slice(&(name.len() as u32).to_le_bytes());
            self.listing
                .extend_from_slice(&[protocol::WASIP1_FILETYPE_REGULAR_FILE, 0, 0, 0]);
            self.listing.extend_from_slice(name);
        }
    }

    fn objects(&self) -> ChoreoFsObjectSet<'_, 16> {
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
                b"\0preopen",
                ObjectId(3),
                FdSpec::new(3, READDIR, 1),
                b"",
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            ChoreoFsObject::readable(
                b"service/result",
                ObjectId(5),
                FdSpec::new(5, READ, 1),
                &self.reply,
                FdBinding::read(FdReadRow::Base),
            ),
            ChoreoFsObject::read_write(
                self.work_path(0),
                ObjectId(6),
                FdSpec::new(6, READ | WRITE, 1),
                &self.work[0],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(1),
                ObjectId(7),
                FdSpec::new(7, READ | WRITE, 1),
                &self.work[1],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(2),
                ObjectId(8),
                FdSpec::new(8, READ | WRITE, 1),
                &self.work[2],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(3),
                ObjectId(9),
                FdSpec::new(9, READ | WRITE, 1),
                &self.work[3],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(4),
                ObjectId(10),
                FdSpec::new(10, READ | WRITE, 1),
                &self.work[4],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(5),
                ObjectId(11),
                FdSpec::new(11, READ | WRITE, 1),
                &self.work[5],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(6),
                ObjectId(12),
                FdSpec::new(12, READ | WRITE, 1),
                &self.work[6],
                READ_WRITE,
            ),
            ChoreoFsObject::read_write(
                self.work_path(7),
                ObjectId(13),
                FdSpec::new(13, READ | WRITE, 1),
                &self.work[7],
                READ_WRITE,
            ),
            ChoreoFsObject::readdir(
                b"work",
                ObjectId(14),
                FdSpec::new(14, READDIR, 1),
                &self.listing,
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            // An opened directory is guest-owned. Its close must not replace
            // or remove the preopen used to resolve subsequent paths.
            ChoreoFsObject::readdir(
                b"",
                ObjectId(15),
                FdSpec::new(15, READDIR, 1),
                b"",
                FdBinding::readdir(FdReaddirRow::Base),
            ),
            ChoreoFsObject::writable(
                &self.service_path,
                ObjectId(SERVICE_FD as u32),
                FdSpec::new(SERVICE_FD, WRITE, 1),
                FdBinding::write(FdWriteRow::Object),
            ),
        ])
    }

    pub async fn read(
        &mut self,
        received: ReceivedPayload<'_, protocol::FdReadReq>,
    ) -> ShellResult<protocol::FdReadDoneRet> {
        let read = protocol::FdReadReq::request(&received).0;
        if read.fd() == 0 && self.offsets[0] == self.input.len() {
            let command = self
                .terminal
                .read_command(self.bus, std::mem::take(&mut self.output))
                .await?;
            if command.len() > 4096 || command.as_bytes().contains(&0) {
                return Err(ShellError::message(
                    "shell command exceeds guest input bound",
                ));
            }
            self.input = (command.len() as u32).to_le_bytes().to_vec();
            self.input.extend_from_slice(command.as_bytes());
            self.offsets[0] = 0;
        }
        let index = read.fd() as usize;
        let offset = self.offsets.get(index).copied().unwrap_or(0);
        let (response, next) = self
            .objects()
            .choreofs()
            .fd_read(received)
            .read_from(offset)?;
        if let Some(offset) = self.offsets.get_mut(index) {
            *offset = next;
        }
        Ok(response)
    }
    pub async fn write<P>(
        &mut self,
        received: ReceivedPayload<'_, P>,
    ) -> ShellResult<protocol::FdWriteDoneRet>
    where
        P: ChoreoFsRequest<Request = (protocol::FdWrite, FdWriteRow)>,
    {
        let (write, _) = P::request(&received);
        let operation = self.objects().choreofs().fd_write(received);
        if !operation.is_writable() {
            return Ok(operation.written());
        }
        match operation.object() {
            Some(ObjectId(1 | 2)) => self.output.extend_from_slice(operation.bytes()),
            Some(ObjectId(4)) => {
                let start = if self.fd_flags[4] & 1 != 0 {
                    self.service_bytes.len()
                } else {
                    self.offsets[4]
                };
                let Some(end) = start
                    .checked_add(operation.bytes().len())
                    .filter(|end| *end <= FILE_BYTES)
                else {
                    return Ok(protocol::FdWriteDoneRet(
                        protocol::FdWriteDone::new_with_errno(SERVICE_FD, 0, 22),
                    ));
                };
                self.service_bytes
                    .resize(self.service_bytes.len().max(end), 0);
                self.service_bytes[start..end].copy_from_slice(operation.bytes());
                self.offsets[4] = end;
            }
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
    pub fn open(
        &mut self,
        received: ReceivedPayload<'_, protocol::PathOpenReq>,
    ) -> ShellResult<protocol::PathOpenedRet> {
        let request = protocol::PathOpenReq::request(&received).0;
        if request.preopen_fd() != 3 {
            return Err(ShellError::message("non-root path_open"));
        }
        let path = normalize(request.path());
        if path.starts_with(b"service/") && path != b"service/result" {
            let objects = ChoreoFsObjectSet::new([ChoreoFsObject::writable(
                path,
                ObjectId(SERVICE_FD as u32),
                FdSpec::new(SERVICE_FD, WRITE, 1),
                FdBinding::write(FdWriteRow::Object),
            )]);
            let open = objects.choreofs().path_open(received);
            if open.is_open() {
                self.service_path.clear();
                self.service_path.extend_from_slice(path);
                if request.open_flags() & 8 != 0 {
                    self.service_bytes.clear();
                }
                self.offsets[4] = 0;
                self.fd_flags[4] = request.fd_flags();
            }
            return Ok(open.opened_ret());
        }
        let created = if !self.paths.iter().any(|existing| existing == path)
            && request.open_flags() & 1 != 0
            && request.rights_base() & WRITE != 0
            && let Some(name) = path.strip_prefix(b"work/")
        {
            if name.is_empty() || name.len() > 64 || name.contains(&b'/') || name.contains(&0) {
                return Ok(protocol::PathOpenedRet(protocol::PathOpened::new(0, 28)));
            }
            let Some(index) = self.paths.iter().position(Vec::is_empty) else {
                return Ok(protocol::PathOpenedRet(protocol::PathOpened::new(0, 22)));
            };
            self.paths[index].extend_from_slice(path);
            Some(index)
        } else {
            None
        };
        let open = self.objects().choreofs().path_open(received);
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
        if let Some(index) = created {
            if !open.is_open() {
                self.paths[index].clear();
            }
            self.refresh_listing();
        }
        Ok(open.opened_ret())
    }
    pub fn stat(
        &self,
        request: ReceivedPayload<'_, protocol::FdFdstatGetReq>,
    ) -> protocol::FdStatRet {
        self.objects().choreofs().fd_fdstat_get(request)
    }
    pub fn file_stat(
        &self,
        request: ReceivedPayload<'_, protocol::FdFilestatGetReq>,
    ) -> protocol::FdFilestatRet {
        self.objects().choreofs().fd_filestat_get(request)
    }
    pub fn path_stat(
        &self,
        request: ReceivedPayload<'_, protocol::PathFilestatGetReq>,
    ) -> ShellResult<protocol::PathFilestatRet> {
        if protocol::PathFilestatGetReq::request(&request)
            .0
            .preopen_fd()
            != 3
        {
            return Err(ShellError::message("non-root path_filestat_get"));
        }
        Ok(self.objects().choreofs().path_filestat_get(request))
    }
    pub fn read_dir(
        &self,
        request: ReceivedPayload<'_, protocol::FdReaddirReq>,
    ) -> ShellResult<protocol::FdReaddirDoneRet> {
        Ok(self.objects().choreofs().fd_readdir(request).read_dir()?)
    }
    pub fn close(&self, request: ReceivedPayload<'_, protocol::FdCloseReq>) -> ClosedFile {
        let completion = self.objects().choreofs().fd_close(request);
        let (path, bytes) = if completion.0.fd() == SERVICE_FD && completion.0.errno() == 0 {
            (self.service_path.clone(), self.service_bytes.clone())
        } else {
            (Vec::new(), Vec::new())
        };
        ClosedFile {
            completion,
            path,
            bytes,
        }
    }
    pub fn result_chunk(&mut self, request: ReceivedPayload<'_, ResultChunk>) -> ShellResult<()> {
        let offset = ResultChunk::offset(&request);
        let objects = ChoreoFsObjectSet::new([ChoreoFsObject::writable(
            b"service/result",
            ObjectId(RESULT_FD as u32),
            FdSpec::new(RESULT_FD, WRITE, 1),
            FdBinding::write(FdWriteRow::Base),
        )]);
        let operation = objects.choreofs().fd_write(request);
        if !operation.is_writable() {
            return Err(ShellError::message("result file is not writable"));
        }
        if offset == 0 {
            self.reply.clear();
        }
        let end = offset + operation.bytes().len();
        self.reply.resize(self.reply.len().max(end), 0);
        self.reply[offset..end].copy_from_slice(operation.bytes());
        let _completion = operation.written();
        Ok(())
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
