//! Each guest and both adjacent physical forks belong to this single global.
pub mod global;
pub(crate) mod identity;
mod imp;
pub(crate) mod localside;
#[cfg(test)]
mod tests;
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            private_fs::{PrivateFs, bindings},
        },
        localside::private_io,
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};

async fn run_9e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::*;
    use identity::roles9e::*;
    let programs = project_9e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;

    let session = SessionId::new(identity::SESSION);
    let mut a = rv.enter(session, &programs.a)?;
    let mut fork_a = rv.enter(session, &programs.fork_a)?;
    let resource_a = rv.enter(session, &programs.resource_a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut fork_b = rv.enter(session, &programs.fork_b)?;
    let resource_b = rv.enter(session, &programs.resource_b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut fork_c = rv.enter(session, &programs.fork_c)?;
    let resource_c = rv.enter(session, &programs.resource_c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let [aterm, bterm, cterm] = terminals;
    let mut afs = PrivateFs::new(aterm, bus);
    let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let mut bfs = PrivateFs::new(bterm, bus);
    let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let mut cfs = PrivateFs::new(cterm, bus);
    let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    tokio::try_join!(
        localside::philosopher::run::<A, FORK_A_RESOURCE, FORK_C_RESOURCE>(
            "PHILOSOPHER_A",
            &mut a,
            bus
        ),
        localside::philosopher::run::<B, FORK_B_RESOURCE, FORK_A_RESOURCE>(
            "PHILOSOPHER_B",
            &mut b,
            bus
        ),
        localside::philosopher::run::<C, FORK_C_RESOURCE, FORK_B_RESOURCE>(
            "PHILOSOPHER_C",
            &mut c,
            bus
        ),
        localside::fork::run::<FORK_A, FORK_A_RESOURCE>(&mut fork_a, resource_a),
        localside::fork::run::<FORK_B, FORK_B_RESOURCE>(&mut fork_b, resource_b),
        localside::fork::run::<FORK_C, FORK_C_RESOURCE>(&mut fork_c, resource_c),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run::<A_WASI, FORK_A_RESOURCE, FORK_C_RESOURCE>(
                    aguest,
                    &mut a_wasi,
                ),
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
                localside::wasi::run::<B_WASI, FORK_B_RESOURCE, FORK_A_RESOURCE>(
                    bguest,
                    &mut b_wasi,
                ),
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
                localside::wasi::run::<C_WASI, FORK_C_RESOURCE, FORK_B_RESOURCE>(
                    cguest,
                    &mut c_wasi,
                ),
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
    use global::*;
    use identity::roles9m::*;
    let programs = project_9m();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;

    let session = SessionId::new(identity::SESSION);
    let mut a = rv.enter(session, &programs.a)?;
    let mut fork_a = rv.enter(session, &programs.fork_a)?;
    let resource_a = rv.enter(session, &programs.resource_a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut fork_b = rv.enter(session, &programs.fork_b)?;
    let resource_b = rv.enter(session, &programs.resource_b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut fork_c = rv.enter(session, &programs.fork_c)?;
    let resource_c = rv.enter(session, &programs.resource_c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut fork_d = rv.enter(session, &programs.fork_d)?;
    let resource_d = rv.enter(session, &programs.resource_d)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut fork_e = rv.enter(session, &programs.fork_e)?;
    let resource_e = rv.enter(session, &programs.resource_e)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let [aterm, bterm, cterm, dterm, eterm] = terminals;
    let mut afs = PrivateFs::new(aterm, bus);
    let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let mut bfs = PrivateFs::new(bterm, bus);
    let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let mut cfs = PrivateFs::new(cterm, bus);
    let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let mut dfs = PrivateFs::new(dterm, bus);
    let mut dmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let mut efs = PrivateFs::new(eterm, bus);
    let mut ememory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    tokio::try_join!(
        localside::philosopher::run::<A, FORK_A_RESOURCE, FORK_E_RESOURCE>(
            "PHILOSOPHER_A",
            &mut a,
            bus
        ),
        localside::philosopher::run::<B, FORK_B_RESOURCE, FORK_A_RESOURCE>(
            "PHILOSOPHER_B",
            &mut b,
            bus
        ),
        localside::philosopher::run::<C, FORK_C_RESOURCE, FORK_B_RESOURCE>(
            "PHILOSOPHER_C",
            &mut c,
            bus
        ),
        localside::philosopher::run::<D, FORK_D_RESOURCE, FORK_C_RESOURCE>(
            "PHILOSOPHER_D",
            &mut d,
            bus
        ),
        localside::philosopher::run::<E, FORK_E_RESOURCE, FORK_D_RESOURCE>(
            "PHILOSOPHER_E",
            &mut e,
            bus
        ),
        localside::fork::run::<FORK_A, FORK_A_RESOURCE>(&mut fork_a, resource_a),
        localside::fork::run::<FORK_B, FORK_B_RESOURCE>(&mut fork_b, resource_b),
        localside::fork::run::<FORK_C, FORK_C_RESOURCE>(&mut fork_c, resource_c),
        localside::fork::run::<FORK_D, FORK_D_RESOURCE>(&mut fork_d, resource_d),
        localside::fork::run::<FORK_E, FORK_E_RESOURCE>(&mut fork_e, resource_e),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run::<A_WASI, FORK_A_RESOURCE, FORK_E_RESOURCE>(
                    aguest,
                    &mut a_wasi,
                ),
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
                localside::wasi::run::<B_WASI, FORK_B_RESOURCE, FORK_A_RESOURCE>(
                    bguest,
                    &mut b_wasi,
                ),
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
                localside::wasi::run::<C_WASI, FORK_C_RESOURCE, FORK_B_RESOURCE>(
                    cguest,
                    &mut c_wasi,
                ),
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
                localside::wasi::run::<D_WASI, FORK_D_RESOURCE, FORK_C_RESOURCE>(
                    dguest,
                    &mut d_wasi,
                ),
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
                localside::wasi::run::<E_WASI, FORK_E_RESOURCE, FORK_D_RESOURCE>(
                    eguest,
                    &mut e_wasi,
                ),
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
    use global::*;
    use identity::roles9h::*;
    let programs = project_9h();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;

    let session = SessionId::new(identity::SESSION);
    let mut a = rv.enter(session, &programs.a)?;
    let mut fork_a = rv.enter(session, &programs.fork_a)?;
    let resource_a = rv.enter(session, &programs.resource_a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut fork_b = rv.enter(session, &programs.fork_b)?;
    let resource_b = rv.enter(session, &programs.resource_b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut fork_c = rv.enter(session, &programs.fork_c)?;
    let resource_c = rv.enter(session, &programs.resource_c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut fork_d = rv.enter(session, &programs.fork_d)?;
    let resource_d = rv.enter(session, &programs.resource_d)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut fork_e = rv.enter(session, &programs.fork_e)?;
    let resource_e = rv.enter(session, &programs.resource_e)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let mut f = rv.enter(session, &programs.f)?;
    let mut fork_f = rv.enter(session, &programs.fork_f)?;
    let resource_f = rv.enter(session, &programs.resource_f)?;
    let mut f_wasi = rv.enter(session, &programs.f_wasi)?;
    let mut f_io = rv.enter(session, &programs.f_io)?;
    let mut g = rv.enter(session, &programs.g)?;
    let mut fork_g = rv.enter(session, &programs.fork_g)?;
    let resource_g = rv.enter(session, &programs.resource_g)?;
    let mut g_wasi = rv.enter(session, &programs.g_wasi)?;
    let mut g_io = rv.enter(session, &programs.g_io)?;
    let [aterm, bterm, cterm, dterm, eterm, fterm, gterm] = terminals;
    let mut afs = PrivateFs::new(aterm, bus);
    let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let mut bfs = PrivateFs::new(bterm, bus);
    let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let mut cfs = PrivateFs::new(cterm, bus);
    let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let mut dfs = PrivateFs::new(dterm, bus);
    let mut dmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let mut efs = PrivateFs::new(eterm, bus);
    let mut ememory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    let mut ffs = PrivateFs::new(fterm, bus);
    let mut fmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut fstore = HibanaWasiGuestStorage::uninit();
    let fguest = fstore.init(module, GuestMemory::new(&mut fmemory), bindings())?;
    let mut gfs = PrivateFs::new(gterm, bus);
    let mut gmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut gstore = HibanaWasiGuestStorage::uninit();
    let gguest = gstore.init(module, GuestMemory::new(&mut gmemory), bindings())?;
    tokio::try_join!(
        localside::philosopher::run::<A, FORK_A_RESOURCE, FORK_G_RESOURCE>(
            "PHILOSOPHER_A",
            &mut a,
            bus
        ),
        localside::philosopher::run::<B, FORK_B_RESOURCE, FORK_A_RESOURCE>(
            "PHILOSOPHER_B",
            &mut b,
            bus
        ),
        localside::philosopher::run::<C, FORK_C_RESOURCE, FORK_B_RESOURCE>(
            "PHILOSOPHER_C",
            &mut c,
            bus
        ),
        localside::philosopher::run::<D, FORK_D_RESOURCE, FORK_C_RESOURCE>(
            "PHILOSOPHER_D",
            &mut d,
            bus
        ),
        localside::philosopher::run::<E, FORK_E_RESOURCE, FORK_D_RESOURCE>(
            "PHILOSOPHER_E",
            &mut e,
            bus
        ),
        localside::philosopher::run::<F, FORK_F_RESOURCE, FORK_E_RESOURCE>(
            "PHILOSOPHER_F",
            &mut f,
            bus
        ),
        localside::philosopher::run::<G, FORK_G_RESOURCE, FORK_F_RESOURCE>(
            "PHILOSOPHER_G",
            &mut g,
            bus
        ),
        localside::fork::run::<FORK_A, FORK_A_RESOURCE>(&mut fork_a, resource_a),
        localside::fork::run::<FORK_B, FORK_B_RESOURCE>(&mut fork_b, resource_b),
        localside::fork::run::<FORK_C, FORK_C_RESOURCE>(&mut fork_c, resource_c),
        localside::fork::run::<FORK_D, FORK_D_RESOURCE>(&mut fork_d, resource_d),
        localside::fork::run::<FORK_E, FORK_E_RESOURCE>(&mut fork_e, resource_e),
        localside::fork::run::<FORK_F, FORK_F_RESOURCE>(&mut fork_f, resource_f),
        localside::fork::run::<FORK_G, FORK_G_RESOURCE>(&mut fork_g, resource_g),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run::<A_WASI, FORK_A_RESOURCE, FORK_G_RESOURCE>(
                    aguest,
                    &mut a_wasi,
                ),
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
                localside::wasi::run::<B_WASI, FORK_B_RESOURCE, FORK_A_RESOURCE>(
                    bguest,
                    &mut b_wasi,
                ),
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
                localside::wasi::run::<C_WASI, FORK_C_RESOURCE, FORK_B_RESOURCE>(
                    cguest,
                    &mut c_wasi,
                ),
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
                localside::wasi::run::<D_WASI, FORK_D_RESOURCE, FORK_C_RESOURCE>(
                    dguest,
                    &mut d_wasi,
                ),
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
                localside::wasi::run::<E_WASI, FORK_E_RESOURCE, FORK_D_RESOURCE>(
                    eguest,
                    &mut e_wasi,
                ),
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
                localside::wasi::run::<F_WASI, FORK_F_RESOURCE, FORK_E_RESOURCE>(
                    fguest,
                    &mut f_wasi,
                ),
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
                localside::wasi::run::<G_WASI, FORK_G_RESOURCE, FORK_F_RESOURCE>(
                    gguest,
                    &mut g_wasi,
                ),
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
            let (bus, mut reader, terminals) = Bus::open(
                ["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"],
                [
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9e::FORK_A_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9e::FORK_B_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9e::FORK_C_RESOURCE,
                    ),
                ],
            );
            let result = tokio::select! {result=run_9e_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result {
Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
Err(error)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
            result
        }
        "9M" => {
            let (bus, mut reader, terminals) = Bus::open(
                [
                    "PHILOSOPHER_A",
                    "PHILOSOPHER_B",
                    "PHILOSOPHER_C",
                    "PHILOSOPHER_D",
                    "PHILOSOPHER_E",
                ],
                [
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9m::FORK_A_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9m::FORK_B_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9m::FORK_C_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9m::FORK_D_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9m::FORK_E_RESOURCE,
                    ),
                ],
            );
            let result = tokio::select! {result=run_9m_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result {
Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
Err(error)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
            result
        }
        "9H" => {
            let (bus, mut reader, terminals) = Bus::open(
                [
                    "PHILOSOPHER_A",
                    "PHILOSOPHER_B",
                    "PHILOSOPHER_C",
                    "PHILOSOPHER_D",
                    "PHILOSOPHER_E",
                    "PHILOSOPHER_F",
                    "PHILOSOPHER_G",
                ],
                [
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_A_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_B_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_C_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_D_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_E_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_F_RESOURCE,
                    ),
                    (
                        SessionId::new(identity::SESSION),
                        identity::roles9h::FORK_G_RESOURCE,
                    ),
                ],
            );
            let result = tokio::select! {result=run_9h_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result {
Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
Err(error)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
}
            result
        }
        _ => Err(ShellError::message("expected dining task 9E, 9M or 9H")),
    }
}
