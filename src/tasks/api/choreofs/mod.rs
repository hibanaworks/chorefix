//! The three actual guests, services, resources and result objects share one global.
pub mod global;
mod imp;
mod localside;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            private_fs::{PrivateFs, bindings},
            service::Validation,
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use hibana_wasip1_runtime::{DEFAULT_GUEST_MEMORY_BYTES, GuestMemory, HibanaWasiGuestStorage};
use std::{cell::RefCell, rc::Rc};
async fn run_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::*;
    let p = programs();
    let ac = Validation::new();
    let bc = Validation::new();
    let tc = Validation::new();
    let result = super::imp::TestInput::default();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &p.admin,
        ResolverRef::<CHECK_ADMIN>::decision_state(&ac, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.backend,
        ResolverRef::<CHECK_BACKEND>::decision_state(&bc, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.tester,
        ResolverRef::<CHECK_TESTER>::decision_state(&tc, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.tester,
        ResolverRef::<TEST_RESULT>::decision_state(&result, super::imp::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(28);
    let mut admin = rv.enter(session, &p.admin)?;
    let mut backend = rv.enter(session, &p.backend)?;
    let mut tester = rv.enter(session, &p.tester)?;
    let mut database = rv.enter(session, &p.database)?;
    let mut test_env = rv.enter(session, &p.test_env)?;
    let mut admin_vm = rv.enter(session, &p.admin_vm)?;
    let mut backend_vm = rv.enter(session, &p.backend_vm)?;
    let mut tester_vm = rv.enter(session, &p.tester_vm)?;
    let mut admin_io = rv.enter(session, &p.admin_io)?;
    let mut backend_io = rv.enter(session, &p.backend_io)?;
    let mut tester_io = rv.enter(session, &p.tester_io)?;
    let mut admin_outcome = rv.enter(session, &p.admin_outcome)?;
    let mut backend_outcome = rv.enter(session, &p.backend_outcome)?;
    let mut admin_outcome_vm = rv.enter(session, &p.admin_outcome_vm)?;
    let mut backend_outcome_vm = rv.enter(session, &p.backend_outcome_vm)?;
    let [bterm, aterm, tterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let tbytes = Rc::new(RefCell::new(Vec::new()));
    let ar = Rc::new(RefCell::new(Vec::new()));
    let br = Rc::new(RefCell::new(Vec::new()));
    let mut afile = imp::OutcomeFile::new(ar.clone());
    let mut bfile = imp::OutcomeFile::new(br.clone());
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut tfs = PrivateFs::new(tterm, bus, tbytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut tmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let mut tstore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let tguest = tstore.init(module, GuestMemory::new(&mut tmemory), bindings())?;
    tokio::try_join!(
        localside::admin::run(&mut admin, bus, &ac),
        localside::backend::run(&mut backend, bus, &bc),
        localside::tester::run(&mut tester, bus, &tc, &result),
        localside::resources::run(&mut database),
        localside::resources::run(&mut test_env),
        localside::outcome::admin::admin(&mut admin_outcome, &mut afile),
        localside::outcome::backend::backend(&mut backend_outcome, &mut bfile),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::recipient(aguest, &mut admin_vm, &mut admin_outcome_vm, abytes),
                localside::private_io::run(&mut admin_io, &mut afs, &ar)
            )?;
            if vm != io {
                return Err(ShellError::message("admin guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::recipient(
                    bguest,
                    &mut backend_vm,
                    &mut backend_outcome_vm,
                    bbytes
                ),
                localside::private_io::run(&mut backend_io, &mut bfs, &br)
            )?;
            if vm != io {
                return Err(ShellError::message("backend guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::tester(tguest, &mut tester_vm, tbytes),
                private_io::run(&mut tester_io, &mut tfs)
            )?;
            if vm != io {
                return Err(ShellError::message("tester guest/IO status differs"));
            }
            tfs.terminal
                .finish(bus, vm, std::mem::take(&mut tfs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let (bus, mut reader, terminals) = Bus::open(["BACKEND_DEV", "DB_ADMIN", "TESTER"]);
    let result = tokio::select! {r=run_guests(module,&bus,terminals)=>r,r=reader.run(&bus)=>r};
    match &result {
        Ok(()) => {
            bus.emit(serde_json::json!({"type":"run_result","status":"complete"}))
                .await?
        }
        Err(e) => {
            bus.emit(
                serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()}),
            )
            .await?
        }
    }
    result
}
