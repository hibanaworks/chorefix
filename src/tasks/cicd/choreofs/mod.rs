//! Direct ChoreoFS service boundary under construction. This is not yet wired
//! to the experiment's guest runner and is not qualified experiment evidence.
pub mod global;
pub mod imp;
mod localside;

use crate::{
    imp::transport::InMemoryTransport,
    shell::{ShellError, ShellResult},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use imp::{Domain, Imports, Validation};

/// Run only the projected service roles. The guest integration supplies actual
/// ABI requests; synthetic request tests do not qualify the guest runner.
pub async fn run<D: Domain>(
    developer: Imports,
    tester: Imports,
    deployer: Imports,
    domain: &D,
) -> ShellResult<()> {
    use global::*;
    let programs = programs();
    let compile = Validation::new();
    let test = Validation::new();
    let deploy = Validation::new();
    let mut slab = vec![0; 256 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit.rendezvous(&mut slab, InMemoryTransport::new())?;
    rendezvous
        .set_resolver(
            &programs.build_server,
            ResolverRef::<COMPILE_CLOSE>::decision_state(&compile, Validation::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rendezvous
        .set_resolver(
            &programs.build_server,
            ResolverRef::<TEST_CLOSE>::decision_state(&test, Validation::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rendezvous
        .set_resolver(
            &programs.deploy_env,
            ResolverRef::<DEPLOY_CLOSE>::decision_state(&deploy, Validation::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let session = SessionId::new(16);
    let mut a = rendezvous.enter(session, &programs.developer)?;
    let mut b = rendezvous.enter(session, &programs.tester)?;
    let mut c = rendezvous.enter(session, &programs.deployer)?;
    let mut build = rendezvous.enter(session, &programs.build_server)?;
    let mut env = rendezvous.enter(session, &programs.deploy_env)?;
    tokio::try_join!(
        localside::developer::run(&mut a, developer),
        localside::tester::run(&mut b, tester),
        localside::deployer::run(&mut c, deployer),
        localside::infrastructure::build_server(&mut build, domain, &compile, &test),
        localside::infrastructure::deploy_env(&mut env, domain, &deploy),
    )?;
    Ok(())
}

/// Execute three real WASI guests and all task/private-I/O roles projected from
/// the same composed global. The original simulator remains outside this runtime.
pub async fn run_guests(
    module: &[u8],
    bus: &crate::shell::imp::bus::Bus,
    terminals: [crate::shell::imp::bus::Terminal; 3],
) -> ShellResult<()> {
    use crate::shell::{
        imp::private_fs::{PrivateFs, bindings},
        localside::{direct_guest, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{DEFAULT_GUEST_MEMORY_BYTES, GuestMemory, HibanaWasiGuestStorage};
    use std::{cell::RefCell, rc::Rc};
    use tokio::sync::mpsc;
    let programs = programs();
    let compile = Validation::new();
    let test = Validation::new();
    let deploy = Validation::new();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit.rendezvous(&mut slab, InMemoryTransport::new())?;
    rendezvous
        .set_resolver(
            &programs.build_server,
            ResolverRef::<COMPILE_CLOSE>::decision_state(&compile, Validation::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rendezvous
        .set_resolver(
            &programs.build_server,
            ResolverRef::<TEST_CLOSE>::decision_state(&test, Validation::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    rendezvous
        .set_resolver(
            &programs.deploy_env,
            ResolverRef::<DEPLOY_CLOSE>::decision_state(&deploy, Validation::decide),
        )
        .map_err(|error| ShellError::message(format!("{error:?}")))?;
    let session = SessionId::new(16);
    let mut a = rendezvous.enter(session, &programs.developer)?;
    let mut b = rendezvous.enter(session, &programs.tester)?;
    let mut c = rendezvous.enter(session, &programs.deployer)?;
    let mut build = rendezvous.enter(session, &programs.build_server)?;
    let mut env = rendezvous.enter(session, &programs.deploy_env)?;
    let mut avm = rendezvous.enter(session, &programs.dev_vm)?;
    let mut aio = rendezvous.enter(session, &programs.dev_io)?;
    let mut bvm = rendezvous.enter(session, &programs.test_vm)?;
    let mut bio = rendezvous.enter(session, &programs.test_io)?;
    let mut cvm = rendezvous.enter(session, &programs.deploy_vm)?;
    let mut cio = rendezvous.enter(session, &programs.deploy_io)?;
    let (asend, arecv) = mpsc::unbounded_channel();
    let (bsend, brecv) = mpsc::unbounded_channel();
    let (csend, crecv) = mpsc::unbounded_channel();
    let [aterm, bterm, cterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    // Backing capacity is supplied here; committing additional pages still
    // requires the projected MemoryGrow request/reply in each I/O localside.
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    tokio::try_join!(
        localside::developer::run(&mut a, arecv),
        localside::tester::run(&mut b, brecv),
        localside::deployer::run(&mut c, crecv),
        localside::infrastructure::build_server(&mut build, bus, &compile, &test),
        localside::infrastructure::deploy_env(&mut env, bus, &deploy),
        async {
            let (guest, io) = tokio::try_join!(
                direct_guest::run(aguest, &mut avm, asend, abytes),
                private_io::run(&mut aio, &mut afs)
            )?;
            if guest != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            afs.terminal.finish(bus, guest).await?;
            if guest != 0 {
                return Err(ShellError::message("developer guest failed"));
            }
            Ok(())
        },
        async {
            let (guest, io) = tokio::try_join!(
                direct_guest::run(bguest, &mut bvm, bsend, bbytes),
                private_io::run(&mut bio, &mut bfs)
            )?;
            if guest != io {
                return Err(ShellError::message("tester guest/IO status differs"));
            }
            bfs.terminal.finish(bus, guest).await?;
            if guest != 0 {
                return Err(ShellError::message("tester guest failed"));
            }
            Ok(())
        },
        async {
            let (guest, io) = tokio::try_join!(
                direct_guest::run(cguest, &mut cvm, csend, cbytes),
                private_io::run(&mut cio, &mut cfs)
            )?;
            if guest != io {
                return Err(ShellError::message("deployer guest/IO status differs"));
            }
            cfs.terminal.finish(bus, guest).await?;
            if guest != 0 {
                return Err(ShellError::message("deployer guest failed"));
            }
            Ok(())
        },
    )?;
    Ok(())
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let (bus, mut reader, terminals) = crate::shell::imp::bus::Bus::open();
    let result = tokio::select! {
        result = run_guests(module, &bus, terminals) => result,
        result = reader.run(&bus) => result,
    };
    match &result {
        Ok(()) => bus.emit(serde_json::json!({"type":"run_result", "status":"complete"})).await?,
        Err(error) => bus.emit(serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
    }
    result
}
