//! Bounded shell integration. Task-specific coordination remains in tasks/*.
pub mod global;
pub(crate) mod imp {
    pub mod bus;
    pub mod environment;
    pub mod error;
    pub mod private_fs;
    pub mod service;
}
pub(crate) mod localside {
    pub mod environment;
    pub mod guest;
    pub mod private_io;
}
pub use imp::error::{ShellError, ShellResult};

use crate::imp::transport::InMemoryTransport;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{
    DEFAULT_GUEST_MEMORY_BYTES, GuestMemory, exchange::HibanaWasiGuestStorage,
};

pub async fn run(module: &[u8]) -> ShellResult<i32> {
    let (guest_program, env_program) = global::programs();
    let mut slab = vec![0; 256 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, InMemoryTransport::new())?;
    let mut guest_endpoint = rv.enter(SessionId::new(1), &guest_program)?;
    let mut env_endpoint = rv.enter(SessionId::new(1), &env_program)?;
    let mut memory = vec![0; DEFAULT_GUEST_MEMORY_BYTES];
    let mut guest_storage = HibanaWasiGuestStorage::uninit();
    let guest = guest_storage.init(
        module,
        GuestMemory::new(&mut memory),
        imp::environment::bindings(),
    )?;
    let mut environment = imp::environment::Environment::default();
    let (guest_status, env_status) = tokio::try_join!(
        localside::guest::run(guest, &mut guest_endpoint),
        localside::environment::run(&mut env_endpoint, &mut environment),
    )?;
    if guest_status != env_status {
        return Err(ShellError::message("guest and environment exit disagree"));
    }
    environment
        .emit(serde_json::json!({"type":"exit", "status":guest_status}))
        .await?;
    Ok(guest_status)
}
