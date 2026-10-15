pub mod global;
mod imp;
mod localside;

use crate::{
    RunError,
    imp::transport::InMemoryTransport,
    resource::{global as resource, localside as resource_roles},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
pub use imp::{Domain, Operation, Request};
use resource_roles::{First, Second};

/// 3 researchers. Common section order removes the circular wait.
pub async fn run_2e<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_2e();
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut a_first = rendezvous
        .enter(session, &programs.a_first)
        .map_err(RunError::Attach)?;
    let mut a_second = rendezvous
        .enter(session, &programs.a_second)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut b_first = rendezvous
        .enter(session, &programs.b_first)
        .map_err(RunError::Attach)?;
    let mut b_second = rendezvous
        .enter(session, &programs.b_second)
        .map_err(RunError::Attach)?;
    let mut c = rendezvous
        .enter(session, &programs.c)
        .map_err(RunError::Attach)?;
    let mut c_first = rendezvous
        .enter(session, &programs.c_first)
        .map_err(RunError::Attach)?;
    let mut c_second = rendezvous
        .enter(session, &programs.c_second)
        .map_err(RunError::Attach)?;
    let resource_programs = resource::project_resource();
    let mut intro_broker = rendezvous
        .enter(SessionId::new(2), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut intro_resource = rendezvous
        .enter(SessionId::new(2), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut methods_broker = rendezvous
        .enter(SessionId::new(3), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut methods_resource = rendezvous
        .enter(SessionId::new(3), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut results_broker = rendezvous
        .enter(SessionId::new(4), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut results_resource = rendezvous
        .enter(SessionId::new(4), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let researchers = async {
        tokio::try_join!(
            localside::researcher::run("RESEARCHER_A", &mut a, domain),
            localside::researcher::run("RESEARCHER_B", &mut b, domain),
            localside::researcher::run("RESEARCHER_C", &mut c, domain),
        )?;
        Ok::<(), RunError<String>>(())
    };
    let sections = async {
        tokio::try_join!(
            resource_roles::broker_once::<A_FIRST, C_FIRST, First, First>(
                &mut a_first,
                &mut c_first,
                &mut intro_broker
            ),
            resource_roles::resource(&mut intro_resource),
            resource_roles::broker_once::<A_SECOND, B_FIRST, Second, First>(
                &mut a_second,
                &mut b_first,
                &mut methods_broker
            ),
            resource_roles::resource(&mut methods_resource),
            resource_roles::broker_once::<B_SECOND, C_SECOND, Second, Second>(
                &mut b_second,
                &mut c_second,
                &mut results_broker
            ),
            resource_roles::resource(&mut results_resource),
        )?;
        Ok::<(), RunError<String>>(())
    };
    tokio::try_join!(researchers, sections)?;
    Ok(())
}

/// 5 researchers. Common section order removes the circular wait.
pub async fn run_2m<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_2m();
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut a_first = rendezvous
        .enter(session, &programs.a_first)
        .map_err(RunError::Attach)?;
    let mut a_second = rendezvous
        .enter(session, &programs.a_second)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut b_first = rendezvous
        .enter(session, &programs.b_first)
        .map_err(RunError::Attach)?;
    let mut b_second = rendezvous
        .enter(session, &programs.b_second)
        .map_err(RunError::Attach)?;
    let mut c = rendezvous
        .enter(session, &programs.c)
        .map_err(RunError::Attach)?;
    let mut c_first = rendezvous
        .enter(session, &programs.c_first)
        .map_err(RunError::Attach)?;
    let mut c_second = rendezvous
        .enter(session, &programs.c_second)
        .map_err(RunError::Attach)?;
    let mut d = rendezvous
        .enter(session, &programs.d)
        .map_err(RunError::Attach)?;
    let mut d_first = rendezvous
        .enter(session, &programs.d_first)
        .map_err(RunError::Attach)?;
    let mut d_second = rendezvous
        .enter(session, &programs.d_second)
        .map_err(RunError::Attach)?;
    let mut e = rendezvous
        .enter(session, &programs.e)
        .map_err(RunError::Attach)?;
    let mut e_first = rendezvous
        .enter(session, &programs.e_first)
        .map_err(RunError::Attach)?;
    let mut e_second = rendezvous
        .enter(session, &programs.e_second)
        .map_err(RunError::Attach)?;
    let resource_programs = resource::project_resource();
    let mut intro_broker = rendezvous
        .enter(SessionId::new(2), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut intro_resource = rendezvous
        .enter(SessionId::new(2), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut background_broker = rendezvous
        .enter(SessionId::new(3), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut background_resource = rendezvous
        .enter(SessionId::new(3), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut methods_broker = rendezvous
        .enter(SessionId::new(4), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut methods_resource = rendezvous
        .enter(SessionId::new(4), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut experiments_broker = rendezvous
        .enter(SessionId::new(5), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut experiments_resource = rendezvous
        .enter(SessionId::new(5), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut results_broker = rendezvous
        .enter(SessionId::new(6), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut results_resource = rendezvous
        .enter(SessionId::new(6), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let researchers = async {
        tokio::try_join!(
            localside::researcher::run("RESEARCHER_A", &mut a, domain),
            localside::researcher::run("RESEARCHER_B", &mut b, domain),
            localside::researcher::run("RESEARCHER_C", &mut c, domain),
            localside::researcher::run("RESEARCHER_D", &mut d, domain),
            localside::researcher::run("RESEARCHER_E", &mut e, domain),
        )?;
        Ok::<(), RunError<String>>(())
    };
    let sections = async {
        tokio::try_join!(
            resource_roles::broker_once::<A_FIRST, E_FIRST, First, First>(
                &mut a_first,
                &mut e_first,
                &mut intro_broker
            ),
            resource_roles::resource(&mut intro_resource),
            resource_roles::broker_once::<A_SECOND, B_FIRST, Second, First>(
                &mut a_second,
                &mut b_first,
                &mut background_broker
            ),
            resource_roles::resource(&mut background_resource),
            resource_roles::broker_once::<B_SECOND, C_FIRST, Second, First>(
                &mut b_second,
                &mut c_first,
                &mut methods_broker
            ),
            resource_roles::resource(&mut methods_resource),
            resource_roles::broker_once::<C_SECOND, D_FIRST, Second, First>(
                &mut c_second,
                &mut d_first,
                &mut experiments_broker
            ),
            resource_roles::resource(&mut experiments_resource),
            resource_roles::broker_once::<D_SECOND, E_SECOND, Second, Second>(
                &mut d_second,
                &mut e_second,
                &mut results_broker
            ),
            resource_roles::resource(&mut results_resource),
        )?;
        Ok::<(), RunError<String>>(())
    };
    tokio::try_join!(researchers, sections)?;
    Ok(())
}

/// 7 researchers. Common section order removes the circular wait.
pub async fn run_2h<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_2h();
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut a_first = rendezvous
        .enter(session, &programs.a_first)
        .map_err(RunError::Attach)?;
    let mut a_second = rendezvous
        .enter(session, &programs.a_second)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut b_first = rendezvous
        .enter(session, &programs.b_first)
        .map_err(RunError::Attach)?;
    let mut b_second = rendezvous
        .enter(session, &programs.b_second)
        .map_err(RunError::Attach)?;
    let mut c = rendezvous
        .enter(session, &programs.c)
        .map_err(RunError::Attach)?;
    let mut c_first = rendezvous
        .enter(session, &programs.c_first)
        .map_err(RunError::Attach)?;
    let mut c_second = rendezvous
        .enter(session, &programs.c_second)
        .map_err(RunError::Attach)?;
    let mut d = rendezvous
        .enter(session, &programs.d)
        .map_err(RunError::Attach)?;
    let mut d_first = rendezvous
        .enter(session, &programs.d_first)
        .map_err(RunError::Attach)?;
    let mut d_second = rendezvous
        .enter(session, &programs.d_second)
        .map_err(RunError::Attach)?;
    let mut e = rendezvous
        .enter(session, &programs.e)
        .map_err(RunError::Attach)?;
    let mut e_first = rendezvous
        .enter(session, &programs.e_first)
        .map_err(RunError::Attach)?;
    let mut e_second = rendezvous
        .enter(session, &programs.e_second)
        .map_err(RunError::Attach)?;
    let mut f = rendezvous
        .enter(session, &programs.f)
        .map_err(RunError::Attach)?;
    let mut f_first = rendezvous
        .enter(session, &programs.f_first)
        .map_err(RunError::Attach)?;
    let mut f_second = rendezvous
        .enter(session, &programs.f_second)
        .map_err(RunError::Attach)?;
    let mut g = rendezvous
        .enter(session, &programs.g)
        .map_err(RunError::Attach)?;
    let mut g_first = rendezvous
        .enter(session, &programs.g_first)
        .map_err(RunError::Attach)?;
    let mut g_second = rendezvous
        .enter(session, &programs.g_second)
        .map_err(RunError::Attach)?;
    let resource_programs = resource::project_resource();
    let mut intro_broker = rendezvous
        .enter(SessionId::new(2), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut intro_resource = rendezvous
        .enter(SessionId::new(2), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut background_broker = rendezvous
        .enter(SessionId::new(3), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut background_resource = rendezvous
        .enter(SessionId::new(3), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut related_broker = rendezvous
        .enter(SessionId::new(4), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut related_resource = rendezvous
        .enter(SessionId::new(4), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut methods_broker = rendezvous
        .enter(SessionId::new(5), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut methods_resource = rendezvous
        .enter(SessionId::new(5), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut experiments_broker = rendezvous
        .enter(SessionId::new(6), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut experiments_resource = rendezvous
        .enter(SessionId::new(6), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut results_broker = rendezvous
        .enter(SessionId::new(7), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut results_resource = rendezvous
        .enter(SessionId::new(7), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut discussion_broker = rendezvous
        .enter(SessionId::new(8), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut discussion_resource = rendezvous
        .enter(SessionId::new(8), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let researchers = async {
        tokio::try_join!(
            localside::researcher::run("RESEARCHER_A", &mut a, domain),
            localside::researcher::run("RESEARCHER_B", &mut b, domain),
            localside::researcher::run("RESEARCHER_C", &mut c, domain),
            localside::researcher::run("RESEARCHER_D", &mut d, domain),
            localside::researcher::run("RESEARCHER_E", &mut e, domain),
            localside::researcher::run("RESEARCHER_F", &mut f, domain),
            localside::researcher::run("RESEARCHER_G", &mut g, domain),
        )?;
        Ok::<(), RunError<String>>(())
    };
    let sections = async {
        tokio::try_join!(
            resource_roles::broker_once::<A_FIRST, G_FIRST, First, First>(
                &mut a_first,
                &mut g_first,
                &mut intro_broker
            ),
            resource_roles::resource(&mut intro_resource),
            resource_roles::broker_once::<A_SECOND, B_FIRST, Second, First>(
                &mut a_second,
                &mut b_first,
                &mut background_broker
            ),
            resource_roles::resource(&mut background_resource),
            resource_roles::broker_once::<B_SECOND, C_FIRST, Second, First>(
                &mut b_second,
                &mut c_first,
                &mut related_broker
            ),
            resource_roles::resource(&mut related_resource),
            resource_roles::broker_once::<C_SECOND, D_FIRST, Second, First>(
                &mut c_second,
                &mut d_first,
                &mut methods_broker
            ),
            resource_roles::resource(&mut methods_resource),
            resource_roles::broker_once::<D_SECOND, E_FIRST, Second, First>(
                &mut d_second,
                &mut e_first,
                &mut experiments_broker
            ),
            resource_roles::resource(&mut experiments_resource),
            resource_roles::broker_once::<E_SECOND, F_FIRST, Second, First>(
                &mut e_second,
                &mut f_first,
                &mut results_broker
            ),
            resource_roles::resource(&mut results_resource),
            resource_roles::broker_once::<F_SECOND, G_SECOND, Second, Second>(
                &mut f_second,
                &mut g_second,
                &mut discussion_broker
            ),
            resource_roles::resource(&mut discussion_resource),
        )?;
        Ok::<(), RunError<String>>(())
    };
    tokio::try_join!(researchers, sections)?;
    Ok(())
}
