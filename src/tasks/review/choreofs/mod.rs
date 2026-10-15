//! The direct 6E WASI boundary. Every task request and response is projected
//! from global.rs; localside operations remain in their role files.
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
    use super::imp::{ReviewInput, decide};
    use crate::shell::{
        imp::private_fs::{PrivateFs, bindings},
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{DEFAULT_GUEST_MEMORY_BYTES, GuestMemory, HibanaWasiGuestStorage};
    use std::{cell::RefCell, rc::Rc};
    let p = programs();
    let a_args = Validation::new();
    let b_args = Validation::new();
    let a_merge_args = Validation::new();
    let b_merge_args = Validation::new();
    let review_a_args = Validation::new();
    let review_b_args = Validation::new();
    let review_a_result = ReviewInput::default();
    let review_b_result = ReviewInput::default();
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &p.repo,
        ResolverRef::<CHECK_A>::decision_state(&a_args, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.repo,
        ResolverRef::<CHECK_B>::decision_state(&b_args, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.review_a,
        ResolverRef::<CHECK_REVIEW_A>::decision_state(&review_a_args, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.review_b,
        ResolverRef::<CHECK_REVIEW_B>::decision_state(&review_b_args, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.review_a,
        ResolverRef::<DECIDE_A>::decision_state(&review_a_result, decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.review_b,
        ResolverRef::<DECIDE_B>::decision_state(&review_b_result, decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.repo,
        ResolverRef::<CHECK_MERGE_A>::decision_state(&a_merge_args, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &p.repo,
        ResolverRef::<CHECK_MERGE_B>::decision_state(&b_merge_args, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(6);
    let mut a_wasi = rv.enter(session, &p.a_wasi)?;
    let mut b_wasi = rv.enter(session, &p.b_wasi)?;
    let mut review_wasi = rv.enter(session, &p.review_wasi)?;
    let mut review_a_wasi = rv.enter(session, &p.review_a_wasi)?;
    let mut review_b_wasi = rv.enter(session, &p.review_b_wasi)?;
    let mut a = rv.enter(session, &p.a)?;
    let mut b = rv.enter(session, &p.b)?;
    let mut review_a = rv.enter(session, &p.review_a)?;
    let mut review_b = rv.enter(session, &p.review_b)?;
    let mut repo = rv.enter(session, &p.repo)?;
    let mut a_io = rv.enter(session, &p.a_io)?;
    let mut b_io = rv.enter(session, &p.b_io)?;
    let mut review_io = rv.enter(session, &p.review_io)?;
    let [a_terminal, b_terminal, review_terminal] = terminals;
    let a_bytes = Rc::new(RefCell::new(Vec::new()));
    let mut a_fs = PrivateFs::new(a_terminal, bus, a_bytes.clone());
    let mut a_memory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut a_store = HibanaWasiGuestStorage::uninit();
    let a_guest = a_store.init(module, GuestMemory::new(&mut a_memory), bindings())?;
    let b_bytes = Rc::new(RefCell::new(Vec::new()));
    let mut b_fs = PrivateFs::new(b_terminal, bus, b_bytes.clone());
    let mut b_memory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut b_store = HibanaWasiGuestStorage::uninit();
    let b_guest = b_store.init(module, GuestMemory::new(&mut b_memory), bindings())?;
    let review_bytes = Rc::new(RefCell::new(Vec::new()));
    let mut review_fs = PrivateFs::new(review_terminal, bus, review_bytes.clone());
    let mut review_memory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut review_store = HibanaWasiGuestStorage::uninit();
    let review_guest =
        review_store.init(module, GuestMemory::new(&mut review_memory), bindings())?;
    tokio::try_join!(
        localside::developer_a::run(&mut a),
        localside::developer_b::run(&mut b),
        localside::reviewer_a::run(&mut review_a, bus, &review_a_args, &review_a_result),
        localside::reviewer_b::run(&mut review_b, bus, &review_b_args, &review_b_result),
        localside::repo::run(
            &mut repo,
            bus,
            &a_args,
            &a_merge_args,
            &b_args,
            &b_merge_args
        ),
        async {
            let (vm, io) = tokio::try_join!(
                localside::developer_wasi::run(a_guest, &mut a_wasi, a_bytes),
                private_io::run(&mut a_io, &mut a_fs)
            )?;
            if vm != io {
                return Err(ShellError::message("a guest/IO status differs"));
            }
            a_fs.terminal
                .finish(bus, vm, std::mem::take(&mut a_fs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::developer_wasi::run(b_guest, &mut b_wasi, b_bytes),
                private_io::run(&mut b_io, &mut b_fs)
            )?;
            if vm != io {
                return Err(ShellError::message("b guest/IO status differs"));
            }
            b_fs.terminal
                .finish(bus, vm, std::mem::take(&mut b_fs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::reviewer_wasi::run(
                    review_guest,
                    &mut review_wasi,
                    &mut review_a_wasi,
                    &mut review_b_wasi,
                    review_bytes,
                ),
                private_io::run(&mut review_io, &mut review_fs)
            )?;
            if vm != io {
                return Err(ShellError::message("review guest/IO status differs"));
            }
            review_fs
                .terminal
                .finish(bus, vm, std::mem::take(&mut review_fs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let (bus, mut reader, terminals) =
        crate::shell::imp::bus::Bus::open(["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"]);
    let result = tokio::select! {
        result=run_guests(module,&bus,terminals)=>result,
        result=reader.run(&bus)=>result,
    };
    match &result {
        Ok(())=>bus.emit(serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error)=>bus.emit(serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
