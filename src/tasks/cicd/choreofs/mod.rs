//! Projected request/reply connects each WASI role to its service owner.
pub mod global;
pub(crate) mod localside;

use crate::{
    imp::transport::InMemoryTransport,
    shell::{ShellError, ShellResult, imp::service::Validation},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};

async fn run_guests(
    module: &[u8],
    bus: &crate::shell::imp::bus::Bus,
    terminals: [crate::shell::imp::bus::Terminal; 3],
) -> ShellResult<()> {
    use crate::shell::{
        imp::private_fs::{PrivateFs, bindings},
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{DEFAULT_GUEST_MEMORY_BYTES, GuestMemory, HibanaWasiGuestStorage};
    use std::{cell::RefCell, rc::Rc};
    let programs = programs();
    let compile = Validation::new();
    let test = Validation::new();
    let deploy = Validation::new();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.developer,
        ResolverRef::<COMPILE_CLOSE>::decision_state(&compile, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.tester,
        ResolverRef::<TEST_CLOSE>::decision_state(&test, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.deployer,
        ResolverRef::<DEPLOY_CLOSE>::decision_state(&deploy, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(16);
    let mut avm = rv.enter(session, &programs.dev_wasi)?;
    let mut bvm = rv.enter(session, &programs.test_wasi)?;
    let mut cvm = rv.enter(session, &programs.deploy_wasi)?;
    let mut developer = rv.enter(session, &programs.developer)?;
    let mut tester = rv.enter(session, &programs.tester)?;
    let mut deployer = rv.enter(session, &programs.deployer)?;
    let mut build = rv.enter(session, &programs.build_server)?;
    let mut env = rv.enter(session, &programs.deploy_env)?;
    let mut aio = rv.enter(session, &programs.dev_io)?;
    let mut bio = rv.enter(session, &programs.test_io)?;
    let mut cio = rv.enter(session, &programs.deploy_io)?;
    let [aterm, bterm, cterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
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
        localside::developer::run(&mut developer, bus, &compile),
        localside::tester::run(&mut tester, bus, &test),
        localside::deployer::run(&mut deployer, bus, &deploy),
        localside::infrastructure::build_server(&mut build),
        localside::infrastructure::deploy_env(&mut env),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut avm, abytes),
                private_io::run(&mut aio, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut bvm, bbytes),
                private_io::run(&mut bio, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("tester guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut cvm, cbytes),
                private_io::run(&mut cio, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("deployer guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let (bus, mut reader, terminals) =
        crate::shell::imp::bus::Bus::open(["DEVELOPER", "TESTER", "DEPLOYER"]);
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

#[cfg(test)]
mod tests;
