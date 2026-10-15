pub mod choreofs;
pub mod global;
mod imp;
mod localside;

use crate::resource::{global as resource, localside as resource_roles};
use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
pub use imp::{Domain, Operation, Request};
use imp::{TestInput, decide};
use resource_roles::{First, Second};

/// Seven concurrent workflows. Every inter-role operation uses an Endpoint.
pub async fn run_1h<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_1h();
    let inputs: [TestInput; 7] = std::array::from_fn(|_| TestInput::default());
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    rendezvous
        .set_resolver(
            &programs.a,
            ResolverRef::<TEST_A>::decision_state(&inputs[0], decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.b,
            ResolverRef::<TEST_B>::decision_state(&inputs[1], decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.c,
            ResolverRef::<TEST_C>::decision_state(&inputs[2], decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.d,
            ResolverRef::<TEST_D>::decision_state(&inputs[3], decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.e,
            ResolverRef::<TEST_E>::decision_state(&inputs[4], decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.f,
            ResolverRef::<TEST_F>::decision_state(&inputs[5], decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.g,
            ResolverRef::<TEST_G>::decision_state(&inputs[6], decide),
        )
        .map_err(RunError::Resolver)?;
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
    let mut result_a = rendezvous
        .enter(session, &programs.result_a)
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
    let mut result_b = rendezvous
        .enter(session, &programs.result_b)
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
    let mut result_c = rendezvous
        .enter(session, &programs.result_c)
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
    let mut result_d = rendezvous
        .enter(session, &programs.result_d)
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
    let mut result_e = rendezvous
        .enter(session, &programs.result_e)
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
    let mut result_f = rendezvous
        .enter(session, &programs.result_f)
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
    let mut result_g = rendezvous
        .enter(session, &programs.result_g)
        .map_err(RunError::Attach)?;
    let resource_programs = resource::project_resource();
    let mut auth_broker = rendezvous
        .enter(SessionId::new(2), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut auth_resource = rendezvous
        .enter(SessionId::new(2), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut database_broker = rendezvous
        .enter(SessionId::new(3), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut database_resource = rendezvous
        .enter(SessionId::new(3), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut api_broker = rendezvous
        .enter(SessionId::new(4), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut api_resource = rendezvous
        .enter(SessionId::new(4), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut payment_broker = rendezvous
        .enter(SessionId::new(5), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut payment_resource = rendezvous
        .enter(SessionId::new(5), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut notify_broker = rendezvous
        .enter(SessionId::new(6), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut notify_resource = rendezvous
        .enter(SessionId::new(6), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut analytics_broker = rendezvous
        .enter(SessionId::new(7), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut analytics_resource = rendezvous
        .enter(SessionId::new(7), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut search_broker = rendezvous
        .enter(SessionId::new(8), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut search_resource = rendezvous
        .enter(SessionId::new(8), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let developers = async {
        tokio::try_join!(
            localside::developer::run("DEVELOPER_A", &mut a, domain, &inputs[0]),
            localside::developer::run("DEVELOPER_B", &mut b, domain, &inputs[1]),
            localside::developer::run("DEVELOPER_C", &mut c, domain, &inputs[2]),
            localside::developer::run("DEVELOPER_D", &mut d, domain, &inputs[3]),
            localside::developer::run("DEVELOPER_E", &mut e, domain, &inputs[4]),
            localside::developer::run("DEVELOPER_F", &mut f, domain, &inputs[5]),
            localside::developer::run("DEVELOPER_G", &mut g, domain, &inputs[6]),
            async {
                localside::result::run(&mut result_a)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::result::run(&mut result_b)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::result::run(&mut result_c)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::result::run(&mut result_d)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::result::run(&mut result_e)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::result::run(&mut result_f)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::result::run(&mut result_g)
                    .await
                    .map_err(RunError::Endpoint)
            },
        )?;
        Ok::<(), RunError<String>>(())
    };
    let modules = async {
        tokio::try_join!(
            resource_roles::broker::<A_FIRST, G_FIRST, First, First>(
                &mut a_first,
                &mut g_first,
                &mut auth_broker
            ),
            resource_roles::resource(&mut auth_resource),
            resource_roles::broker::<A_SECOND, B_FIRST, Second, First>(
                &mut a_second,
                &mut b_first,
                &mut database_broker
            ),
            resource_roles::resource(&mut database_resource),
            resource_roles::broker::<B_SECOND, C_FIRST, Second, First>(
                &mut b_second,
                &mut c_first,
                &mut api_broker
            ),
            resource_roles::resource(&mut api_resource),
            resource_roles::broker::<C_SECOND, D_FIRST, Second, First>(
                &mut c_second,
                &mut d_first,
                &mut payment_broker
            ),
            resource_roles::resource(&mut payment_resource),
            resource_roles::broker::<D_SECOND, E_FIRST, Second, First>(
                &mut d_second,
                &mut e_first,
                &mut notify_broker
            ),
            resource_roles::resource(&mut notify_resource),
            resource_roles::broker::<E_SECOND, F_FIRST, Second, First>(
                &mut e_second,
                &mut f_first,
                &mut analytics_broker
            ),
            resource_roles::resource(&mut analytics_resource),
            resource_roles::broker::<F_SECOND, G_SECOND, Second, Second>(
                &mut f_second,
                &mut g_second,
                &mut search_broker
            ),
            resource_roles::resource(&mut search_resource),
        )?;
        Ok::<(), RunError<String>>(())
    };
    tokio::try_join!(developers, modules)?;
    Ok(())
}

/// Three concurrent workflows. Every inter-role operation uses an Endpoint.
pub async fn run_1e<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_1e();
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
    let mut auth_broker = rendezvous
        .enter(SessionId::new(2), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut auth_resource = rendezvous
        .enter(SessionId::new(2), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut database_broker = rendezvous
        .enter(SessionId::new(3), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut database_resource = rendezvous
        .enter(SessionId::new(3), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut api_broker = rendezvous
        .enter(SessionId::new(4), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut api_resource = rendezvous
        .enter(SessionId::new(4), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let developers = async {
        tokio::try_join!(
            localside::developer::terminal("DEVELOPER_A", &mut a, domain),
            localside::developer::terminal("DEVELOPER_B", &mut b, domain),
            localside::developer::terminal("DEVELOPER_C", &mut c, domain),
        )?;
        Ok::<(), RunError<String>>(())
    };
    let modules = async {
        tokio::try_join!(
            resource_roles::broker_once::<A_FIRST, C_FIRST, First, First>(
                &mut a_first,
                &mut c_first,
                &mut auth_broker
            ),
            resource_roles::resource(&mut auth_resource),
            resource_roles::broker_once::<A_SECOND, B_FIRST, Second, First>(
                &mut a_second,
                &mut b_first,
                &mut database_broker
            ),
            resource_roles::resource(&mut database_resource),
            resource_roles::broker_once::<B_SECOND, C_SECOND, Second, Second>(
                &mut b_second,
                &mut c_second,
                &mut api_broker
            ),
            resource_roles::resource(&mut api_resource),
        )?;
        Ok::<(), RunError<String>>(())
    };
    tokio::try_join!(developers, modules)?;
    Ok(())
}

/// Five concurrent workflows. Every inter-role operation uses an Endpoint.
pub async fn run_1m<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_1m();
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
    let mut auth_broker = rendezvous
        .enter(SessionId::new(2), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut auth_resource = rendezvous
        .enter(SessionId::new(2), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut database_broker = rendezvous
        .enter(SessionId::new(3), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut database_resource = rendezvous
        .enter(SessionId::new(3), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut api_broker = rendezvous
        .enter(SessionId::new(4), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut api_resource = rendezvous
        .enter(SessionId::new(4), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut payment_broker = rendezvous
        .enter(SessionId::new(5), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut payment_resource = rendezvous
        .enter(SessionId::new(5), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let mut notify_broker = rendezvous
        .enter(SessionId::new(6), &resource_programs.broker)
        .map_err(RunError::Attach)?;
    let mut notify_resource = rendezvous
        .enter(SessionId::new(6), &resource_programs.resource)
        .map_err(RunError::Attach)?;
    let developers = async {
        tokio::try_join!(
            localside::developer::terminal("DEVELOPER_A", &mut a, domain),
            localside::developer::terminal("DEVELOPER_B", &mut b, domain),
            localside::developer::terminal("DEVELOPER_C", &mut c, domain),
            localside::developer::terminal("DEVELOPER_D", &mut d, domain),
            localside::developer::terminal("DEVELOPER_E", &mut e, domain),
        )?;
        Ok::<(), RunError<String>>(())
    };
    let modules = async {
        tokio::try_join!(
            resource_roles::broker_once::<A_FIRST, E_FIRST, First, First>(
                &mut a_first,
                &mut e_first,
                &mut auth_broker
            ),
            resource_roles::resource(&mut auth_resource),
            resource_roles::broker_once::<A_SECOND, B_FIRST, Second, First>(
                &mut a_second,
                &mut b_first,
                &mut database_broker
            ),
            resource_roles::resource(&mut database_resource),
            resource_roles::broker_once::<B_SECOND, C_FIRST, Second, First>(
                &mut b_second,
                &mut c_first,
                &mut api_broker
            ),
            resource_roles::resource(&mut api_resource),
            resource_roles::broker_once::<C_SECOND, D_FIRST, Second, First>(
                &mut c_second,
                &mut d_first,
                &mut payment_broker
            ),
            resource_roles::resource(&mut payment_resource),
            resource_roles::broker_once::<D_SECOND, E_SECOND, Second, Second>(
                &mut d_second,
                &mut e_second,
                &mut notify_broker
            ),
            resource_roles::resource(&mut notify_resource),
        )?;
        Ok::<(), RunError<String>>(())
    };
    tokio::try_join!(developers, modules)?;
    Ok(())
}
