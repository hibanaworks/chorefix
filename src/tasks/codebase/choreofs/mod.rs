//! Guest, materializer, private I/O and physical modules use one session.
pub mod global;
pub(crate) mod input;
pub(crate) mod localside;
#[cfg(test)]
mod tests;
pub mod wire;
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
#[hibana_runtime::localside]
async fn run_1e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::{roles1e::*, *};
    let programs = project_1e();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
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
    let auth_resource = rv.enter(session, &programs.auth_resource)?;
    let mut database = rv.enter(session, &programs.database)?;
    let database_resource = rv.enter(session, &programs.database_resource)?;
    let mut api = rv.enter(session, &programs.api)?;
    let api_resource = rv.enter(session, &programs.api_resource)?;
    let [term_a, term_b, term_c] = terminals;
    let fs_a = PrivateFs::new(term_a, bus);
    let mut memory_a = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_a = HibanaWasiGuestStorage::uninit();
    let guest_a = store_a.init(module, GuestMemory::new(&mut memory_a), bindings())?;
    let fs_b = PrivateFs::new(term_b, bus);
    let mut memory_b = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_b = HibanaWasiGuestStorage::uninit();
    let guest_b = store_b.init(module, GuestMemory::new(&mut memory_b), bindings())?;
    let fs_c = PrivateFs::new(term_c, bus);
    let mut memory_c = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_c = HibanaWasiGuestStorage::uninit();
    let guest_c = store_c.init(module, GuestMemory::new(&mut memory_c), bindings())?;
    #[localside]
    localside::developer::run::<EASY, A, AUTH_RESOURCE, DATABASE_RESOURCE>(&mut a, bus);
    #[localside]
    localside::developer::run::<EASY, B, API_RESOURCE, DATABASE_RESOURCE>(&mut b, bus);
    #[localside]
    localside::developer::run::<EASY, C, API_RESOURCE, AUTH_RESOURCE>(&mut c, bus);
    #[localside]
    localside::module::run::<AUTH, AUTH_RESOURCE>(&mut auth, auth_resource);
    #[localside]
    localside::module::run::<DATABASE, DATABASE_RESOURCE>(&mut database, database_resource);
    #[localside]
    localside::module::run::<API, API_RESOURCE>(&mut api, api_resource);
    #[localside]
    localside::wasi::run::<A_WASI, A, EASY, AUTH_RESOURCE, DATABASE_RESOURCE>(guest_a, &mut a_wasi);
    #[localside]
    private_io::run(&mut a_io, fs_a);
    #[localside]
    localside::wasi::run::<B_WASI, B, EASY, API_RESOURCE, DATABASE_RESOURCE>(guest_b, &mut b_wasi);
    #[localside]
    private_io::run(&mut b_io, fs_b);
    #[localside]
    localside::wasi::run::<C_WASI, C, EASY, API_RESOURCE, AUTH_RESOURCE>(guest_c, &mut c_wasi);
    #[localside]
    private_io::run(&mut c_io, fs_c);
    Ok(())
}
#[hibana_runtime::localside]
async fn run_1m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use global::{roles1m::*, *};
    let programs = project_1m();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
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
    let auth_resource = rv.enter(session, &programs.auth_resource)?;
    let mut database = rv.enter(session, &programs.database)?;
    let database_resource = rv.enter(session, &programs.database_resource)?;
    let mut api = rv.enter(session, &programs.api)?;
    let api_resource = rv.enter(session, &programs.api_resource)?;
    let mut payment = rv.enter(session, &programs.payment)?;
    let payment_resource = rv.enter(session, &programs.payment_resource)?;
    let mut notify = rv.enter(session, &programs.notify)?;
    let notify_resource = rv.enter(session, &programs.notify_resource)?;
    let [term_a, term_b, term_c, term_d, term_e] = terminals;
    let fs_a = PrivateFs::new(term_a, bus);
    let mut memory_a = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_a = HibanaWasiGuestStorage::uninit();
    let guest_a = store_a.init(module, GuestMemory::new(&mut memory_a), bindings())?;
    let fs_b = PrivateFs::new(term_b, bus);
    let mut memory_b = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_b = HibanaWasiGuestStorage::uninit();
    let guest_b = store_b.init(module, GuestMemory::new(&mut memory_b), bindings())?;
    let fs_c = PrivateFs::new(term_c, bus);
    let mut memory_c = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_c = HibanaWasiGuestStorage::uninit();
    let guest_c = store_c.init(module, GuestMemory::new(&mut memory_c), bindings())?;
    let fs_d = PrivateFs::new(term_d, bus);
    let mut memory_d = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_d = HibanaWasiGuestStorage::uninit();
    let guest_d = store_d.init(module, GuestMemory::new(&mut memory_d), bindings())?;
    let fs_e = PrivateFs::new(term_e, bus);
    let mut memory_e = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_e = HibanaWasiGuestStorage::uninit();
    let guest_e = store_e.init(module, GuestMemory::new(&mut memory_e), bindings())?;
    #[localside]
    localside::developer::run::<MEDIUM, A, AUTH_RESOURCE, DATABASE_RESOURCE>(&mut a, bus);
    #[localside]
    localside::developer::run::<MEDIUM, B, API_RESOURCE, DATABASE_RESOURCE>(&mut b, bus);
    #[localside]
    localside::developer::run::<MEDIUM, C, API_RESOURCE, PAYMENT_RESOURCE>(&mut c, bus);
    #[localside]
    localside::developer::run::<MEDIUM, D, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(&mut d, bus);
    #[localside]
    localside::developer::run::<MEDIUM, E, AUTH_RESOURCE, NOTIFY_RESOURCE>(&mut e, bus);
    #[localside]
    localside::module::run::<AUTH, AUTH_RESOURCE>(&mut auth, auth_resource);
    #[localside]
    localside::module::run::<DATABASE, DATABASE_RESOURCE>(&mut database, database_resource);
    #[localside]
    localside::module::run::<API, API_RESOURCE>(&mut api, api_resource);
    #[localside]
    localside::module::run::<PAYMENT, PAYMENT_RESOURCE>(&mut payment, payment_resource);
    #[localside]
    localside::module::run::<NOTIFY, NOTIFY_RESOURCE>(&mut notify, notify_resource);
    #[localside]
    localside::wasi::run::<A_WASI, A, MEDIUM, AUTH_RESOURCE, DATABASE_RESOURCE>(
        guest_a,
        &mut a_wasi,
    );
    #[localside]
    private_io::run(&mut a_io, fs_a);
    #[localside]
    localside::wasi::run::<B_WASI, B, MEDIUM, API_RESOURCE, DATABASE_RESOURCE>(
        guest_b,
        &mut b_wasi,
    );
    #[localside]
    private_io::run(&mut b_io, fs_b);
    #[localside]
    localside::wasi::run::<C_WASI, C, MEDIUM, API_RESOURCE, PAYMENT_RESOURCE>(guest_c, &mut c_wasi);
    #[localside]
    private_io::run(&mut c_io, fs_c);
    #[localside]
    localside::wasi::run::<D_WASI, D, MEDIUM, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(
        guest_d,
        &mut d_wasi,
    );
    #[localside]
    private_io::run(&mut d_io, fs_d);
    #[localside]
    localside::wasi::run::<E_WASI, E, MEDIUM, AUTH_RESOURCE, NOTIFY_RESOURCE>(guest_e, &mut e_wasi);
    #[localside]
    private_io::run(&mut e_io, fs_e);
    Ok(())
}
pub async fn run_stdio(module: &[u8], task: &str) -> ShellResult<()> {
    match task {
        "1E" => {
            let (bus, mut reader, terminals) = Bus::open(
                ["DEVELOPER_A", "DEVELOPER_B", "DEVELOPER_C"],
                [
                    (SessionId::new(20), global::roles1e::AUTH_RESOURCE),
                    (SessionId::new(20), global::roles1e::DATABASE_RESOURCE),
                    (SessionId::new(20), global::roles1e::API_RESOURCE),
                ],
            );
            let result =
                hibana_runtime::race(run_1e_guests(module, &bus, terminals), reader.run(&bus))
                    .await;
            match &result{Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(e)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()})).await?}
            result
        }
        "1M" => {
            let (bus, mut reader, terminals) = Bus::open(
                [
                    "DEVELOPER_A",
                    "DEVELOPER_B",
                    "DEVELOPER_C",
                    "DEVELOPER_D",
                    "DEVELOPER_E",
                ],
                [
                    (SessionId::new(20), global::roles1m::AUTH_RESOURCE),
                    (SessionId::new(20), global::roles1m::DATABASE_RESOURCE),
                    (SessionId::new(20), global::roles1m::API_RESOURCE),
                    (SessionId::new(20), global::roles1m::PAYMENT_RESOURCE),
                    (SessionId::new(20), global::roles1m::NOTIFY_RESOURCE),
                ],
            );
            let result =
                hibana_runtime::race(run_1m_guests(module, &bus, terminals), reader.run(&bus))
                    .await;
            match &result{Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(e)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()})).await?}
            result
        }
        "1H" => {
            let (bus, mut reader, terminals) = Bus::open(
                [
                    "DEVELOPER_A",
                    "DEVELOPER_B",
                    "DEVELOPER_C",
                    "DEVELOPER_D",
                    "DEVELOPER_E",
                    "DEVELOPER_F",
                    "DEVELOPER_G",
                ],
                [
                    (SessionId::new(20), global::roles1h::AUTH_RESOURCE),
                    (SessionId::new(20), global::roles1h::DATABASE_RESOURCE),
                    (SessionId::new(20), global::roles1h::API_RESOURCE),
                    (SessionId::new(20), global::roles1h::PAYMENT_RESOURCE),
                    (SessionId::new(20), global::roles1h::NOTIFY_RESOURCE),
                    (SessionId::new(20), global::roles1h::ANALYTICS_RESOURCE),
                    (SessionId::new(20), global::roles1h::SEARCH_RESOURCE),
                ],
            );
            let result =
                hibana_runtime::race(run_1h_guests(module, &bus, terminals), reader.run(&bus))
                    .await;
            match &result{Ok(())=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,Err(e)=>reader.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":e.to_string()})).await?}
            result
        }
        _ => Err(ShellError::message("expected codebase task 1E, 1M or 1H")),
    }
}

#[hibana_runtime::localside]
async fn run_1h_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use global::{roles1h::*, *};
    use hibana::runtime::resolver::ResolverRef;
    let programs = project_1h();
    let result_a = input::TestResult::default();
    let result_b = input::TestResult::default();
    let result_c = input::TestResult::default();
    let result_d = input::TestResult::default();
    let result_e = input::TestResult::default();
    let result_f = input::TestResult::default();
    let result_g = input::TestResult::default();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&result_a, input::TestResult::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&result_b, input::TestResult::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&result_c, input::TestResult::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<CHECK_D>::decision_state(&result_d, input::TestResult::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<CHECK_E>::decision_state(&result_e, input::TestResult::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.f,
        ResolverRef::<CHECK_F>::decision_state(&result_f, input::TestResult::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.g,
        ResolverRef::<CHECK_G>::decision_state(&result_g, input::TestResult::decide),
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
    let mut f = rv.enter(session, &programs.f)?;
    let mut f_wasi = rv.enter(session, &programs.f_wasi)?;
    let mut f_io = rv.enter(session, &programs.f_io)?;
    let mut g = rv.enter(session, &programs.g)?;
    let mut g_wasi = rv.enter(session, &programs.g_wasi)?;
    let mut g_io = rv.enter(session, &programs.g_io)?;
    let mut auth = rv.enter(session, &programs.auth)?;
    let auth_resource = rv.enter(session, &programs.auth_resource)?;
    let mut database = rv.enter(session, &programs.database)?;
    let database_resource = rv.enter(session, &programs.database_resource)?;
    let mut api = rv.enter(session, &programs.api)?;
    let api_resource = rv.enter(session, &programs.api_resource)?;
    let mut payment = rv.enter(session, &programs.payment)?;
    let payment_resource = rv.enter(session, &programs.payment_resource)?;
    let mut notify = rv.enter(session, &programs.notify)?;
    let notify_resource = rv.enter(session, &programs.notify_resource)?;
    let mut analytics = rv.enter(session, &programs.analytics)?;
    let analytics_resource = rv.enter(session, &programs.analytics_resource)?;
    let mut search = rv.enter(session, &programs.search)?;
    let search_resource = rv.enter(session, &programs.search_resource)?;
    let [term_a, term_b, term_c, term_d, term_e, term_f, term_g] = terminals;
    let fs_a = PrivateFs::new(term_a, bus);
    let mut memory_a = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_a = HibanaWasiGuestStorage::uninit();
    let guest_a = store_a.init(module, GuestMemory::new(&mut memory_a), bindings())?;
    let fs_b = PrivateFs::new(term_b, bus);
    let mut memory_b = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_b = HibanaWasiGuestStorage::uninit();
    let guest_b = store_b.init(module, GuestMemory::new(&mut memory_b), bindings())?;
    let fs_c = PrivateFs::new(term_c, bus);
    let mut memory_c = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_c = HibanaWasiGuestStorage::uninit();
    let guest_c = store_c.init(module, GuestMemory::new(&mut memory_c), bindings())?;
    let fs_d = PrivateFs::new(term_d, bus);
    let mut memory_d = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_d = HibanaWasiGuestStorage::uninit();
    let guest_d = store_d.init(module, GuestMemory::new(&mut memory_d), bindings())?;
    let fs_e = PrivateFs::new(term_e, bus);
    let mut memory_e = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_e = HibanaWasiGuestStorage::uninit();
    let guest_e = store_e.init(module, GuestMemory::new(&mut memory_e), bindings())?;
    let fs_f = PrivateFs::new(term_f, bus);
    let mut memory_f = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_f = HibanaWasiGuestStorage::uninit();
    let guest_f = store_f.init(module, GuestMemory::new(&mut memory_f), bindings())?;
    let fs_g = PrivateFs::new(term_g, bus);
    let mut memory_g = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut store_g = HibanaWasiGuestStorage::uninit();
    let guest_g = store_g.init(module, GuestMemory::new(&mut memory_g), bindings())?;
    #[localside]
    localside::developer_reconsider::run::<HARD, A, AUTH_RESOURCE, DATABASE_RESOURCE, AUTH, DATABASE>(
        &mut a, bus, &result_a,
    );
    #[localside]
    localside::developer_reconsider::run::<HARD, B, API_RESOURCE, DATABASE_RESOURCE, API, DATABASE>(
        &mut b, bus, &result_b,
    );
    #[localside]
    localside::developer_reconsider::run::<HARD, C, API_RESOURCE, PAYMENT_RESOURCE, API, PAYMENT>(
        &mut c, bus, &result_c,
    );
    #[localside]
    localside::developer_reconsider::run::<
        HARD,
        D,
        NOTIFY_RESOURCE,
        PAYMENT_RESOURCE,
        NOTIFY,
        PAYMENT,
    >(&mut d, bus, &result_d);
    #[localside]
    localside::developer_reconsider::run::<
        HARD,
        E,
        ANALYTICS_RESOURCE,
        NOTIFY_RESOURCE,
        ANALYTICS,
        NOTIFY,
    >(&mut e, bus, &result_e);
    #[localside]
    localside::developer_reconsider::run::<
        HARD,
        F,
        ANALYTICS_RESOURCE,
        SEARCH_RESOURCE,
        ANALYTICS,
        SEARCH,
    >(&mut f, bus, &result_f);
    #[localside]
    localside::developer_reconsider::run::<HARD, G, AUTH_RESOURCE, SEARCH_RESOURCE, AUTH, SEARCH>(
        &mut g, bus, &result_g,
    );
    #[localside]
    localside::module_reconsider::run::<
        AUTH,
        AUTH_RESOURCE,
        DATABASE,
        DATABASE_RESOURCE,
        SEARCH,
        SEARCH_RESOURCE,
    >(&mut auth, auth_resource);
    #[localside]
    localside::module_reconsider::run::<
        DATABASE,
        DATABASE_RESOURCE,
        AUTH,
        AUTH_RESOURCE,
        API,
        API_RESOURCE,
    >(&mut database, database_resource);
    #[localside]
    localside::module_reconsider::run::<
        API,
        API_RESOURCE,
        PAYMENT,
        PAYMENT_RESOURCE,
        DATABASE,
        DATABASE_RESOURCE,
    >(&mut api, api_resource);
    #[localside]
    localside::module_reconsider::run::<
        PAYMENT,
        PAYMENT_RESOURCE,
        API,
        API_RESOURCE,
        NOTIFY,
        NOTIFY_RESOURCE,
    >(&mut payment, payment_resource);
    #[localside]
    localside::module_reconsider::run::<
        NOTIFY,
        NOTIFY_RESOURCE,
        ANALYTICS,
        ANALYTICS_RESOURCE,
        PAYMENT,
        PAYMENT_RESOURCE,
    >(&mut notify, notify_resource);
    #[localside]
    localside::module_reconsider::run::<
        ANALYTICS,
        ANALYTICS_RESOURCE,
        NOTIFY,
        NOTIFY_RESOURCE,
        SEARCH,
        SEARCH_RESOURCE,
    >(&mut analytics, analytics_resource);
    #[localside]
    localside::module_reconsider::run::<
        SEARCH,
        SEARCH_RESOURCE,
        ANALYTICS,
        ANALYTICS_RESOURCE,
        AUTH,
        AUTH_RESOURCE,
    >(&mut search, search_resource);
    #[localside]
    localside::wasi_reconsider::run::<A_WASI, A, HARD, AUTH_RESOURCE, DATABASE_RESOURCE>(
        guest_a,
        &mut a_wasi,
    );
    #[localside]
    private_io::run(&mut a_io, fs_a);
    #[localside]
    localside::wasi_reconsider::run::<B_WASI, B, HARD, API_RESOURCE, DATABASE_RESOURCE>(
        guest_b,
        &mut b_wasi,
    );
    #[localside]
    private_io::run(&mut b_io, fs_b);
    #[localside]
    localside::wasi_reconsider::run::<C_WASI, C, HARD, API_RESOURCE, PAYMENT_RESOURCE>(
        guest_c,
        &mut c_wasi,
    );
    #[localside]
    private_io::run(&mut c_io, fs_c);
    #[localside]
    localside::wasi_reconsider::run::<D_WASI, D, HARD, NOTIFY_RESOURCE, PAYMENT_RESOURCE>(
        guest_d,
        &mut d_wasi,
    );
    #[localside]
    private_io::run(&mut d_io, fs_d);
    #[localside]
    localside::wasi_reconsider::run::<E_WASI, E, HARD, ANALYTICS_RESOURCE, NOTIFY_RESOURCE>(
        guest_e,
        &mut e_wasi,
    );
    #[localside]
    private_io::run(&mut e_io, fs_e);
    #[localside]
    localside::wasi_reconsider::run::<F_WASI, F, HARD, ANALYTICS_RESOURCE, SEARCH_RESOURCE>(
        guest_f,
        &mut f_wasi,
    );
    #[localside]
    private_io::run(&mut f_io, fs_f);
    #[localside]
    localside::wasi_reconsider::run::<G_WASI, G, HARD, AUTH_RESOURCE, SEARCH_RESOURCE>(
        guest_g,
        &mut g_wasi,
    );
    #[localside]
    private_io::run(&mut g_io, fs_g);
    Ok(())
}
