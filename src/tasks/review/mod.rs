pub mod global;
mod imp;
mod localside;
pub use imp::{Domain, Operation, Request};

use crate::{
    RunError,
    imp::transport::InMemoryTransport,
    resource::{global as resource, localside as resource_roles},
};
use hibana::{
    g::Message,
    runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef},
};
use imp::{ReviewInput, decide};

/// Message selection for the two actual resource-port conversations.
struct Port;
impl resource_roles::LoanMessages for Port {
    type Acquire = resource::Request;
    type Closed = resource::Closed;
    type Granted = resource::Granted;
    type Return = resource::Return;
    type Returned = resource::Returned;
}

pub async fn run_6e<D: Domain>(domain: &D) -> Result<(), RunError<String>> {
    use global::*;
    let programs = project_6e();
    let input_a = ReviewInput::default();
    let input_b = ReviewInput::default();
    let mut slab = vec![0; 128 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    rendezvous
        .set_resolver(
            &programs.reviewer,
            ResolverRef::<REVIEW_A>::decision_state(&input_a, decide),
        )
        .map_err(RunError::Resolver)?;
    rendezvous
        .set_resolver(
            &programs.reviewer,
            ResolverRef::<REVIEW_B>::decision_state(&input_b, decide),
        )
        .map_err(RunError::Resolver)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut reviewer = rendezvous
        .enter(session, &programs.reviewer)
        .map_err(RunError::Attach)?;
    let mut port_a = rendezvous
        .enter(session, &programs.port_a)
        .map_err(RunError::Attach)?;
    let mut port_b = rendezvous
        .enter(session, &programs.port_b)
        .map_err(RunError::Attach)?;
    let ports = resource::project_resource();
    let mut client_a = rendezvous
        .enter(SessionId::new(2), &ports.broker)
        .map_err(RunError::Attach)?;
    let mut broker_a = rendezvous
        .enter(SessionId::new(2), &ports.resource)
        .map_err(RunError::Attach)?;
    let mut client_b = rendezvous
        .enter(SessionId::new(3), &ports.broker)
        .map_err(RunError::Attach)?;
    let mut broker_b = rendezvous
        .enter(SessionId::new(3), &ports.resource)
        .map_err(RunError::Attach)?;
    let mut repo_broker = rendezvous
        .enter(SessionId::new(4), &ports.broker)
        .map_err(RunError::Attach)?;
    let mut repo = rendezvous
        .enter(SessionId::new(4), &ports.resource)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::developer::run::<
            A,
            { ReadyA::LOGICAL_LABEL },
            { MergedA::LOGICAL_LABEL },
            { ApprovedA::LOGICAL_LABEL },
            { ReviseA::LOGICAL_LABEL },
            D,
        >("DEVELOPER_A", &mut a, domain),
        localside::developer::run::<
            B,
            { ReadyB::LOGICAL_LABEL },
            { MergedB::LOGICAL_LABEL },
            { ApprovedB::LOGICAL_LABEL },
            { ReviseB::LOGICAL_LABEL },
            D,
        >("DEVELOPER_B", &mut b, domain),
        localside::reviewer::run(&mut reviewer, domain, &input_a, &input_b),
        localside::port::run::<
            PORT_A,
            { ApprovedPortA::LOGICAL_LABEL },
            { RevisePortA::LOGICAL_LABEL },
        >(&mut port_a, &mut client_a),
        localside::port::run::<
            PORT_B,
            { ApprovedPortB::LOGICAL_LABEL },
            { RevisePortB::LOGICAL_LABEL },
        >(&mut port_b, &mut client_b),
        resource_roles::broker::<1, 1, Port, Port>(&mut broker_a, &mut broker_b, &mut repo_broker),
        resource_roles::resource(&mut repo),
    )?;
    Ok(())
}
