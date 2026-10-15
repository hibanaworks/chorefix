//! Guest, materializer, private I/O and physical modules use one session.
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
use localside::developer::Assignment;
use std::{cell::RefCell, rc::Rc};
async fn run_1e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::{roles1e::*, *};
    let programs = project_1e();
    let check_a = Validation::new();
    let check_b = Validation::new();
    let check_c = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&check_a, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&check_b, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&check_c, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(20);
    let mut a = rv.enter(session, &programs.a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut auth = rv.enter(session, &programs.auth)?;
    let mut database = rv.enter(session, &programs.database)?;
    let mut api = rv.enter(session, &programs.api)?;
    let [term_a, term_b, term_c] = terminals;
    let bytes_a = Rc::new(RefCell::new(Vec::new()));
    let mut fs_a = PrivateFs::new(term_a, bus, bytes_a.clone());
    let mut memory_a = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_a = HibanaWasiGuestStorage::uninit();
    let guest_a = store_a.init(module, GuestMemory::new(&mut memory_a), bindings())?;
    let bytes_b = Rc::new(RefCell::new(Vec::new()));
    let mut fs_b = PrivateFs::new(term_b, bus, bytes_b.clone());
    let mut memory_b = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_b = HibanaWasiGuestStorage::uninit();
    let guest_b = store_b.init(module, GuestMemory::new(&mut memory_b), bindings())?;
    let bytes_c = Rc::new(RefCell::new(Vec::new()));
    let mut fs_c = PrivateFs::new(term_c, bus, bytes_c.clone());
    let mut memory_c = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_c = HibanaWasiGuestStorage::uninit();
    let guest_c = store_c.init(module, GuestMemory::new(&mut memory_c), bindings())?;
    tokio::try_join!(
        localside::developer::run::<A, Auth, Database, _>(
            &mut a,
            bus,
            &check_a,
            &Assignment {
                agent: "DEVELOPER_A",
                feature: "user_authentication",
                modules: ["AUTH_MODULE", "DATABASE_MODULE"]
            }
        ),
        localside::developer::run::<B, Api, Database, _>(
            &mut b,
            bus,
            &check_b,
            &Assignment {
                agent: "DEVELOPER_B",
                feature: "rest_api_endpoints",
                modules: ["API_MODULE", "DATABASE_MODULE"]
            }
        ),
        localside::developer::run::<C, Api, Auth, _>(
            &mut c,
            bus,
            &check_c,
            &Assignment {
                agent: "DEVELOPER_C",
                feature: "api_auth_middleware",
                modules: ["API_MODULE", "AUTH_MODULE"]
            }
        ),
        localside::module::run::<AUTH, Auth>(&mut auth),
        localside::module::run::<DATABASE, Database>(&mut database),
        localside::module::run::<API, Api>(&mut api),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_a, &mut a_wasi, bytes_a),
                private_io::run(&mut a_io, &mut fs_a)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_a.terminal
                .finish(bus, vm, std::mem::take(&mut fs_a.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_b, &mut b_wasi, bytes_b),
                private_io::run(&mut b_io, &mut fs_b)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_b.terminal
                .finish(bus, vm, std::mem::take(&mut fs_b.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_c, &mut c_wasi, bytes_c),
                private_io::run(&mut c_io, &mut fs_c)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_c.terminal
                .finish(bus, vm, std::mem::take(&mut fs_c.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}
async fn run_1m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use global::{roles1m::*, *};
    let programs = project_1m();
    let check_a = Validation::new();
    let check_b = Validation::new();
    let check_c = Validation::new();
    let check_d = Validation::new();
    let check_e = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&check_a, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&check_b, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&check_c, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<CHECK_D>::decision_state(&check_d, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<CHECK_E>::decision_state(&check_e, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(20);
    let mut a = rv.enter(session, &programs.a)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let mut auth = rv.enter(session, &programs.auth)?;
    let mut database = rv.enter(session, &programs.database)?;
    let mut api = rv.enter(session, &programs.api)?;
    let mut payment = rv.enter(session, &programs.payment)?;
    let mut notify = rv.enter(session, &programs.notify)?;
    let [term_a, term_b, term_c, term_d, term_e] = terminals;
    let bytes_a = Rc::new(RefCell::new(Vec::new()));
    let mut fs_a = PrivateFs::new(term_a, bus, bytes_a.clone());
    let mut memory_a = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_a = HibanaWasiGuestStorage::uninit();
    let guest_a = store_a.init(module, GuestMemory::new(&mut memory_a), bindings())?;
    let bytes_b = Rc::new(RefCell::new(Vec::new()));
    let mut fs_b = PrivateFs::new(term_b, bus, bytes_b.clone());
    let mut memory_b = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_b = HibanaWasiGuestStorage::uninit();
    let guest_b = store_b.init(module, GuestMemory::new(&mut memory_b), bindings())?;
    let bytes_c = Rc::new(RefCell::new(Vec::new()));
    let mut fs_c = PrivateFs::new(term_c, bus, bytes_c.clone());
    let mut memory_c = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_c = HibanaWasiGuestStorage::uninit();
    let guest_c = store_c.init(module, GuestMemory::new(&mut memory_c), bindings())?;
    let bytes_d = Rc::new(RefCell::new(Vec::new()));
    let mut fs_d = PrivateFs::new(term_d, bus, bytes_d.clone());
    let mut memory_d = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_d = HibanaWasiGuestStorage::uninit();
    let guest_d = store_d.init(module, GuestMemory::new(&mut memory_d), bindings())?;
    let bytes_e = Rc::new(RefCell::new(Vec::new()));
    let mut fs_e = PrivateFs::new(term_e, bus, bytes_e.clone());
    let mut memory_e = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut store_e = HibanaWasiGuestStorage::uninit();
    let guest_e = store_e.init(module, GuestMemory::new(&mut memory_e), bindings())?;
    tokio::try_join!(
        localside::developer::run::<A, Auth, Database, _>(
            &mut a,
            bus,
            &check_a,
            &Assignment {
                agent: "DEVELOPER_A",
                feature: "user_authentication",
                modules: ["AUTH_MODULE", "DATABASE_MODULE"]
            }
        ),
        localside::developer::run::<B, Api, Database, _>(
            &mut b,
            bus,
            &check_b,
            &Assignment {
                agent: "DEVELOPER_B",
                feature: "rest_api_endpoints",
                modules: ["API_MODULE", "DATABASE_MODULE"]
            }
        ),
        localside::developer::run::<C, Api, Payment, _>(
            &mut c,
            bus,
            &check_c,
            &Assignment {
                agent: "DEVELOPER_C",
                feature: "payment_system",
                modules: ["API_MODULE", "PAYMENT_MODULE"]
            }
        ),
        localside::developer::run::<D, Notify, Payment, _>(
            &mut d,
            bus,
            &check_d,
            &Assignment {
                agent: "DEVELOPER_D",
                feature: "notification_service",
                modules: ["NOTIFY_MODULE", "PAYMENT_MODULE"]
            }
        ),
        localside::developer::run::<E, Auth, Notify, _>(
            &mut e,
            bus,
            &check_e,
            &Assignment {
                agent: "DEVELOPER_E",
                feature: "admin_dashboard",
                modules: ["AUTH_MODULE", "NOTIFY_MODULE"]
            }
        ),
        localside::module::run::<AUTH, Auth>(&mut auth),
        localside::module::run::<DATABASE, Database>(&mut database),
        localside::module::run::<API, Api>(&mut api),
        localside::module::run::<PAYMENT, Payment>(&mut payment),
        localside::module::run::<NOTIFY, Notify>(&mut notify),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_a, &mut a_wasi, bytes_a),
                private_io::run(&mut a_io, &mut fs_a)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_a.terminal
                .finish(bus, vm, std::mem::take(&mut fs_a.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_b, &mut b_wasi, bytes_b),
                private_io::run(&mut b_io, &mut fs_b)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_b.terminal
                .finish(bus, vm, std::mem::take(&mut fs_b.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_c, &mut c_wasi, bytes_c),
                private_io::run(&mut c_io, &mut fs_c)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_c.terminal
                .finish(bus, vm, std::mem::take(&mut fs_c.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_d, &mut d_wasi, bytes_d),
                private_io::run(&mut d_io, &mut fs_d)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_d.terminal
                .finish(bus, vm, std::mem::take(&mut fs_d.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(guest_e, &mut e_wasi, bytes_e),
                private_io::run(&mut e_io, &mut fs_e)
            )?;
            if vm != io {
                return Err(ShellError::message("developer guest/IO status differs"));
            }
            fs_e.terminal
                .finish(bus, vm, std::mem::take(&mut fs_e.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}
pub async fn run_stdio(module: &[u8], task: &str) -> ShellResult<()> {
    match task {
        "1E" => {
            let (bus, mut reader, terminals) =
                Bus::open(["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"]);
            let result = tokio::select! {result=run_1e_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result{Ok(())=>bus.emit(serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(e)=>bus.emit(serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()})).await?}
            result
        }
        "1M" => {
            let (bus, mut reader, terminals) = Bus::open([
                "DEVELOPER_A",
                "DEVELOPER_B",
                "DEVELOPER_C",
                "DEVELOPER_D",
                "DEVELOPER_E",
            ]);
            let result = tokio::select! {result=run_1m_guests(module,&bus,terminals)=>result,result=reader.run(&bus)=>result};
            match &result{Ok(())=>bus.emit(serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(e)=>bus.emit(serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()})).await?}
            result
        }
        _ => Err(ShellError::message("expected codebase task 1E or 1M")),
    }
}
