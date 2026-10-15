//! Each guest and both adjacent physical forks belong to this single global.
pub mod global;
pub(crate) mod localside;
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

async fn run_9e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use super::global::{ForkA, ForkB, ForkC};
    use global::{roles9e::*, *};
    let programs = project_9e();
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&acheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&bcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&ccheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(19);
    let mut a = rv.enter(session, &programs.a)?;
    let mut fork_a = rv.enter(session, &programs.fork_a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut fork_b = rv.enter(session, &programs.fork_b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut fork_c = rv.enter(session, &programs.fork_c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let [aterm, bterm, cterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    tokio::try_join!(
        localside::philosopher::run::<A, ForkA, ForkC, _>("PHILOSOPHER_A", &mut a, bus, &acheck),
        localside::philosopher::run::<B, ForkB, ForkA, _>("PHILOSOPHER_B", &mut b, bus, &bcheck),
        localside::philosopher::run::<C, ForkC, ForkB, _>("PHILOSOPHER_C", &mut c, bus, &ccheck),
        localside::fork::run::<FORK_A, ForkA>(&mut fork_a),
        localside::fork::run::<FORK_B, ForkB>(&mut fork_b),
        localside::fork::run::<FORK_C, ForkC>(&mut fork_c),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut a_wasi, abytes),
                private_io::run(&mut a_io, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut b_wasi, bbytes),
                private_io::run(&mut b_io, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut c_wasi, cbytes),
                private_io::run(&mut c_io, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

async fn run_9m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use super::global::{ForkA, ForkB, ForkC, ForkD, ForkE};
    use global::{roles9m::*, *};
    let programs = project_9m();
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let dcheck = Validation::new();
    let echeck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&acheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&bcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&ccheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<CHECK_D>::decision_state(&dcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<CHECK_E>::decision_state(&echeck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(19);
    let mut a = rv.enter(session, &programs.a)?;
    let mut fork_a = rv.enter(session, &programs.fork_a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut fork_b = rv.enter(session, &programs.fork_b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut fork_c = rv.enter(session, &programs.fork_c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut fork_d = rv.enter(session, &programs.fork_d)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut fork_e = rv.enter(session, &programs.fork_e)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let [aterm, bterm, cterm, dterm, eterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let dbytes = Rc::new(RefCell::new(Vec::new()));
    let mut dfs = PrivateFs::new(dterm, bus, dbytes.clone());
    let mut dmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let ebytes = Rc::new(RefCell::new(Vec::new()));
    let mut efs = PrivateFs::new(eterm, bus, ebytes.clone());
    let mut ememory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    tokio::try_join!(
        localside::philosopher::run::<A, ForkA, ForkE, _>("PHILOSOPHER_A", &mut a, bus, &acheck),
        localside::philosopher::run::<B, ForkB, ForkA, _>("PHILOSOPHER_B", &mut b, bus, &bcheck),
        localside::philosopher::run::<C, ForkC, ForkB, _>("PHILOSOPHER_C", &mut c, bus, &ccheck),
        localside::philosopher::run::<D, ForkD, ForkC, _>("PHILOSOPHER_D", &mut d, bus, &dcheck),
        localside::philosopher::run::<E, ForkE, ForkD, _>("PHILOSOPHER_E", &mut e, bus, &echeck),
        localside::fork::run::<FORK_A, ForkA>(&mut fork_a),
        localside::fork::run::<FORK_B, ForkB>(&mut fork_b),
        localside::fork::run::<FORK_C, ForkC>(&mut fork_c),
        localside::fork::run::<FORK_D, ForkD>(&mut fork_d),
        localside::fork::run::<FORK_E, ForkE>(&mut fork_e),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut a_wasi, abytes),
                private_io::run(&mut a_io, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut b_wasi, bbytes),
                private_io::run(&mut b_io, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut c_wasi, cbytes),
                private_io::run(&mut c_io, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(dguest, &mut d_wasi, dbytes),
                private_io::run(&mut d_io, &mut dfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            dfs.terminal
                .finish(bus, vm, std::mem::take(&mut dfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(eguest, &mut e_wasi, ebytes),
                private_io::run(&mut e_io, &mut efs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            efs.terminal
                .finish(bus, vm, std::mem::take(&mut efs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

async fn run_9h_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use super::global::{ForkA, ForkB, ForkC, ForkD, ForkE, ForkF, ForkG};
    use global::{roles9h::*, *};
    let programs = project_9h();
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let dcheck = Validation::new();
    let echeck = Validation::new();
    let fcheck = Validation::new();
    let gcheck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&acheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&bcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&ccheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<CHECK_D>::decision_state(&dcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<CHECK_E>::decision_state(&echeck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.f,
        ResolverRef::<CHECK_F>::decision_state(&fcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.g,
        ResolverRef::<CHECK_G>::decision_state(&gcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(19);
    let mut a = rv.enter(session, &programs.a)?;
    let mut fork_a = rv.enter(session, &programs.fork_a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut fork_b = rv.enter(session, &programs.fork_b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut fork_c = rv.enter(session, &programs.fork_c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut fork_d = rv.enter(session, &programs.fork_d)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut fork_e = rv.enter(session, &programs.fork_e)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let mut f = rv.enter(session, &programs.f)?;
    let mut fork_f = rv.enter(session, &programs.fork_f)?;
    let mut f_wasi = rv.enter(session, &programs.f_wasi)?;
    let mut f_io = rv.enter(session, &programs.f_io)?;
    let mut g = rv.enter(session, &programs.g)?;
    let mut fork_g = rv.enter(session, &programs.fork_g)?;
    let mut g_wasi = rv.enter(session, &programs.g_wasi)?;
    let mut g_io = rv.enter(session, &programs.g_io)?;
    let [aterm, bterm, cterm, dterm, eterm, fterm, gterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let dbytes = Rc::new(RefCell::new(Vec::new()));
    let mut dfs = PrivateFs::new(dterm, bus, dbytes.clone());
    let mut dmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let ebytes = Rc::new(RefCell::new(Vec::new()));
    let mut efs = PrivateFs::new(eterm, bus, ebytes.clone());
    let mut ememory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    let fbytes = Rc::new(RefCell::new(Vec::new()));
    let mut ffs = PrivateFs::new(fterm, bus, fbytes.clone());
    let mut fmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut fstore = HibanaWasiGuestStorage::uninit();
    let fguest = fstore.init(module, GuestMemory::new(&mut fmemory), bindings())?;
    let gbytes = Rc::new(RefCell::new(Vec::new()));
    let mut gfs = PrivateFs::new(gterm, bus, gbytes.clone());
    let mut gmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut gstore = HibanaWasiGuestStorage::uninit();
    let gguest = gstore.init(module, GuestMemory::new(&mut gmemory), bindings())?;
    tokio::try_join!(
        localside::philosopher::run::<A, ForkA, ForkG, _>("PHILOSOPHER_A", &mut a, bus, &acheck),
        localside::philosopher::run::<B, ForkB, ForkA, _>("PHILOSOPHER_B", &mut b, bus, &bcheck),
        localside::philosopher::run::<C, ForkC, ForkB, _>("PHILOSOPHER_C", &mut c, bus, &ccheck),
        localside::philosopher::run::<D, ForkD, ForkC, _>("PHILOSOPHER_D", &mut d, bus, &dcheck),
        localside::philosopher::run::<E, ForkE, ForkD, _>("PHILOSOPHER_E", &mut e, bus, &echeck),
        localside::philosopher::run::<F, ForkF, ForkE, _>("PHILOSOPHER_F", &mut f, bus, &fcheck),
        localside::philosopher::run::<G, ForkG, ForkF, _>("PHILOSOPHER_G", &mut g, bus, &gcheck),
        localside::fork::run::<FORK_A, ForkA>(&mut fork_a),
        localside::fork::run::<FORK_B, ForkB>(&mut fork_b),
        localside::fork::run::<FORK_C, ForkC>(&mut fork_c),
        localside::fork::run::<FORK_D, ForkD>(&mut fork_d),
        localside::fork::run::<FORK_E, ForkE>(&mut fork_e),
        localside::fork::run::<FORK_F, ForkF>(&mut fork_f),
        localside::fork::run::<FORK_G, ForkG>(&mut fork_g),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut a_wasi, abytes),
                private_io::run(&mut a_io, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut b_wasi, bbytes),
                private_io::run(&mut b_io, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut c_wasi, cbytes),
                private_io::run(&mut c_io, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(dguest, &mut d_wasi, dbytes),
                private_io::run(&mut d_io, &mut dfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            dfs.terminal
                .finish(bus, vm, std::mem::take(&mut dfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(eguest, &mut e_wasi, ebytes),
                private_io::run(&mut e_io, &mut efs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            efs.terminal
                .finish(bus, vm, std::mem::take(&mut efs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(fguest, &mut f_wasi, fbytes),
                private_io::run(&mut f_io, &mut ffs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            ffs.terminal
                .finish(bus, vm, std::mem::take(&mut ffs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(gguest, &mut g_wasi, gbytes),
                private_io::run(&mut g_io, &mut gfs)
            )?;
            if vm != io {
                return Err(ShellError::message("philosopher guest/IO status differs"));
            }
            gfs.terminal
                .finish(bus, vm, std::mem::take(&mut gfs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

pub async fn run_stdio(module: &[u8], level: &str) -> ShellResult<()> {
    match level {
        "9E" => {
            let (bus, mut reader, terminals) =
                Bus::open(["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"]);
            let result = tokio::select! {result=run_9e_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result {
Ok(())=>bus.emit(serde_json::json!({"type":"run_result","status":"complete"})).await?,
Err(error)=>bus.emit(serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
            result
        }
        "9M" => {
            let (bus, mut reader, terminals) = Bus::open([
                "PHILOSOPHER_A",
                "PHILOSOPHER_B",
                "PHILOSOPHER_C",
                "PHILOSOPHER_D",
                "PHILOSOPHER_E",
            ]);
            let result = tokio::select! {result=run_9m_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result {
Ok(())=>bus.emit(serde_json::json!({"type":"run_result","status":"complete"})).await?,
Err(error)=>bus.emit(serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
            result
        }
        "9H" => {
            let (bus, mut reader, terminals) = Bus::open([
                "PHILOSOPHER_A",
                "PHILOSOPHER_B",
                "PHILOSOPHER_C",
                "PHILOSOPHER_D",
                "PHILOSOPHER_E",
                "PHILOSOPHER_F",
                "PHILOSOPHER_G",
            ]);
            let result = tokio::select! {result=run_9h_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result {
Ok(())=>bus.emit(serde_json::json!({"type":"run_result","status":"complete"})).await?,
Err(error)=>bus.emit(serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
            result
        }
        _ => Err(ShellError::message("expected dining task 9E, 9M or 9H")),
    }
}
