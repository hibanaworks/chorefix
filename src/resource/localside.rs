//! Forwarding uses projected messages on both sides. No lease channels or
//! independent ownership state authorize these operations.
use super::global as resource;

/// Identifies the actual Endpoint request selected by Tokio.
enum Borrower {
    First,
    Second,
}
use crate::RunError;
use hibana::{Endpoint, g::Message};

pub(crate) trait LoanMessages {
    type Acquire: Message<Payload = ()>;
    type Closed: Message<Payload = ()>;
    type Granted: Message<Payload = ()>;
    type Return: Message<Payload = ()>;
    type Returned: Message<Payload = ()>;
}
pub(crate) struct First;
pub(crate) struct Second;
impl LoanMessages for First {
    type Acquire = resource::AcquireFirst;
    type Closed = resource::ClosedFirst;
    type Granted = resource::GrantedFirst;
    type Return = resource::ReturnFirst;
    type Returned = resource::ReturnedFirst;
}
impl LoanMessages for Second {
    type Acquire = resource::AcquireSecond;
    type Closed = resource::ClosedSecond;
    type Granted = resource::GrantedSecond;
    type Return = resource::ReturnSecond;
    type Returned = resource::ReturnedSecond;
}

pub(crate) async fn resource(endpoint: &mut Endpoint<'_, 1>) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            resource::Request::LOGICAL_LABEL => {
                branch.recv::<resource::Request>().await?;
                endpoint.send::<resource::Granted>(&()).await?;
                endpoint.recv::<resource::Return>().await?;
                endpoint.send::<resource::Returned>(&()).await?;
            }
            resource::Closed::LOGICAL_LABEL => {
                branch.recv::<resource::Closed>().await?;
                return Ok(());
            }
            _ => unreachable!("projected resource operations"),
        }
    }
}

enum Request {
    Acquire(Borrower),
    Closed(Borrower),
}

pub(crate) async fn broker<const P1: u8, const P2: u8, M1: LoanMessages, M2: LoanMessages>(
    first: &mut Endpoint<'_, P1>,
    second: &mut Endpoint<'_, P2>,
    resource: &mut Endpoint<'_, 0>,
) -> Result<(), RunError<String>> {
    loop {
        // The enum records the actual received message. Only Endpoint previews
        // participate in scheduling; the losing preview remains unconsumed.
        let request = {
            tokio::select! {
                result = first.offer() => {
                    let branch = result?;
                    match branch.label() {
                        label if label == M1::Acquire::LOGICAL_LABEL => {
                            branch.recv::<M1::Acquire>().await?;
                            Request::Acquire(Borrower::First)
                        }
                        label if label == M1::Closed::LOGICAL_LABEL => {
                            branch.recv::<M1::Closed>().await?;
                            Request::Closed(Borrower::First)
                        }
                        _ => unreachable!("projected first borrower operations"),
                    }
                }
                result = second.offer() => {
                    let branch = result?;
                    match branch.label() {
                        label if label == M2::Acquire::LOGICAL_LABEL => {
                            branch.recv::<M2::Acquire>().await?;
                            Request::Acquire(Borrower::Second)
                        }
                        label if label == M2::Closed::LOGICAL_LABEL => {
                            branch.recv::<M2::Closed>().await?;
                            Request::Closed(Borrower::Second)
                        }
                        _ => unreachable!("projected second borrower operations"),
                    }
                }
            }
        };
        match request {
            Request::Acquire(Borrower::First) => {
                resource.send::<resource::Request>(&()).await?;
                resource.recv::<resource::Granted>().await?;
                first.send::<M1::Granted>(&()).await?;
                first.recv::<M1::Return>().await?;
                resource.send::<resource::Return>(&()).await?;
                resource.recv::<resource::Returned>().await?;
                first.send::<M1::Returned>(&()).await?;
            }
            Request::Acquire(Borrower::Second) => {
                resource.send::<resource::Request>(&()).await?;
                resource.recv::<resource::Granted>().await?;
                second.send::<M2::Granted>(&()).await?;
                second.recv::<M2::Return>().await?;
                resource.send::<resource::Return>(&()).await?;
                resource.recv::<resource::Returned>().await?;
                second.send::<M2::Returned>(&()).await?;
            }
            Request::Closed(Borrower::First) => {
                loop {
                    let branch = second.offer().await?;
                    match branch.label() {
                        label if label == M2::Acquire::LOGICAL_LABEL => {
                            branch.recv::<M2::Acquire>().await?;
                            resource.send::<resource::Request>(&()).await?;
                            resource.recv::<resource::Granted>().await?;
                            second.send::<M2::Granted>(&()).await?;
                            second.recv::<M2::Return>().await?;
                            resource.send::<resource::Return>(&()).await?;
                            resource.recv::<resource::Returned>().await?;
                            second.send::<M2::Returned>(&()).await?;
                        }
                        label if label == M2::Closed::LOGICAL_LABEL => {
                            branch.recv::<M2::Closed>().await?;
                            break;
                        }
                        _ => unreachable!("projected remaining borrower operations"),
                    }
                }
                break;
            }
            Request::Closed(Borrower::Second) => {
                loop {
                    let branch = first.offer().await?;
                    match branch.label() {
                        label if label == M1::Acquire::LOGICAL_LABEL => {
                            branch.recv::<M1::Acquire>().await?;
                            resource.send::<resource::Request>(&()).await?;
                            resource.recv::<resource::Granted>().await?;
                            first.send::<M1::Granted>(&()).await?;
                            first.recv::<M1::Return>().await?;
                            resource.send::<resource::Return>(&()).await?;
                            resource.recv::<resource::Returned>().await?;
                            first.send::<M1::Returned>(&()).await?;
                        }
                        label if label == M1::Closed::LOGICAL_LABEL => {
                            branch.recv::<M1::Closed>().await?;
                            break;
                        }
                        _ => unreachable!("projected remaining borrower operations"),
                    }
                }
                break;
            }
        }
    }
    resource.send::<resource::Closed>(&()).await?;
    Ok(())
}

/// Each borrower has exactly one projected loan. The first received
/// request determines service order; no resource or participant state is kept.
pub(crate) async fn broker_once<const P1: u8, const P2: u8, M1: LoanMessages, M2: LoanMessages>(
    first: &mut Endpoint<'_, P1>,
    second: &mut Endpoint<'_, P2>,
    resource: &mut Endpoint<'_, 0>,
) -> Result<(), RunError<String>> {
    let origin = tokio::select! {
        received = first.recv::<M1::Acquire>() => { received?; Borrower::First }
        received = second.recv::<M2::Acquire>() => { received?; Borrower::Second }
    };
    match origin {
        Borrower::First => {
            resource.send::<resource::Request>(&()).await?;
            resource.recv::<resource::Granted>().await?;
            first.send::<M1::Granted>(&()).await?;
            first.recv::<M1::Return>().await?;
            resource.send::<resource::Return>(&()).await?;
            resource.recv::<resource::Returned>().await?;
            first.send::<M1::Returned>(&()).await?;
            second.recv::<M2::Acquire>().await?;
            resource.send::<resource::Request>(&()).await?;
            resource.recv::<resource::Granted>().await?;
            second.send::<M2::Granted>(&()).await?;
            second.recv::<M2::Return>().await?;
            resource.send::<resource::Return>(&()).await?;
            resource.recv::<resource::Returned>().await?;
            second.send::<M2::Returned>(&()).await?;
        }
        Borrower::Second => {
            resource.send::<resource::Request>(&()).await?;
            resource.recv::<resource::Granted>().await?;
            second.send::<M2::Granted>(&()).await?;
            second.recv::<M2::Return>().await?;
            resource.send::<resource::Return>(&()).await?;
            resource.recv::<resource::Returned>().await?;
            second.send::<M2::Returned>(&()).await?;
            first.recv::<M1::Acquire>().await?;
            resource.send::<resource::Request>(&()).await?;
            resource.recv::<resource::Granted>().await?;
            first.send::<M1::Granted>(&()).await?;
            first.recv::<M1::Return>().await?;
            resource.send::<resource::Return>(&()).await?;
            resource.recv::<resource::Returned>().await?;
            first.send::<M1::Returned>(&()).await?;
        }
    }
    resource.send::<resource::Closed>(&()).await?;
    Ok(())
}
