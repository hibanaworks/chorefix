//! Forwarding uses projected messages on both sides. No lease channels or
//! independent ownership state authorize these operations.
use super::super::imp::Borrower;
use super::super::{global, global_resource as resource};
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
    type Acquire = global::AcquireFirst;
    type Closed = global::ClosedFirst;
    type Granted = global::GrantedFirst;
    type Return = global::ReturnFirst;
    type Returned = global::ReturnedFirst;
}
impl LoanMessages for Second {
    type Acquire = global::AcquireSecond;
    type Closed = global::ClosedSecond;
    type Granted = global::GrantedSecond;
    type Return = global::ReturnSecond;
    type Returned = global::ReturnedSecond;
}

pub(crate) async fn resource(endpoint: &mut Endpoint<'_, 1>) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            resource::Request::LOGICAL_LABEL => {
                let _request = branch.recv::<resource::Request>().await?;
                endpoint.send::<resource::Granted>(&()).await?;
                endpoint.recv::<resource::Return>().await?;
                endpoint.send::<resource::Returned>(&()).await?;
            }
            resource::Closed::LOGICAL_LABEL => {
                branch.recv::<resource::Closed>().await?;
                return Ok(());
            }
            _ => unreachable!("projected module operations"),
        }
    }
}

async fn remaining<const ROLE: u8, M: LoanMessages>(
    borrower: &mut Endpoint<'_, ROLE>,
    module: &mut Endpoint<'_, 0>,
    origin: Borrower,
) -> Result<(), RunError<String>> {
    loop {
        let branch = borrower.offer().await?;
        match branch.label() {
            label if label == M::Acquire::LOGICAL_LABEL => {
                branch.recv::<M::Acquire>().await?;
                module.send::<resource::Request>(&origin).await?;
                module.recv::<resource::Granted>().await?;
                borrower.send::<M::Granted>(&()).await?;
                borrower.recv::<M::Return>().await?;
                module.send::<resource::Return>(&()).await?;
                module.recv::<resource::Returned>().await?;
                borrower.send::<M::Returned>(&()).await?;
            }
            label if label == M::Closed::LOGICAL_LABEL => {
                branch.recv::<M::Closed>().await?;
                return Ok(());
            }
            _ => unreachable!("projected borrower operations"),
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
    module: &mut Endpoint<'_, 0>,
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
                module.send::<resource::Request>(&Borrower::First).await?;
                module.recv::<resource::Granted>().await?;
                first.send::<M1::Granted>(&()).await?;
                first.recv::<M1::Return>().await?;
                module.send::<resource::Return>(&()).await?;
                module.recv::<resource::Returned>().await?;
                first.send::<M1::Returned>(&()).await?;
            }
            Request::Acquire(Borrower::Second) => {
                module.send::<resource::Request>(&Borrower::Second).await?;
                module.recv::<resource::Granted>().await?;
                second.send::<M2::Granted>(&()).await?;
                second.recv::<M2::Return>().await?;
                module.send::<resource::Return>(&()).await?;
                module.recv::<resource::Returned>().await?;
                second.send::<M2::Returned>(&()).await?;
            }
            Request::Closed(Borrower::First) => {
                remaining::<P2, M2>(second, module, Borrower::Second).await?;
                break;
            }
            Request::Closed(Borrower::Second) => {
                remaining::<P1, M1>(first, module, Borrower::First).await?;
                break;
            }
        }
    }
    module.send::<resource::Closed>(&()).await?;
    Ok(())
}

/// Each 1E/1M borrower has exactly one projected loan. The first received
/// request determines service order; no resource or participant state is kept.
pub(crate) async fn broker_once<const P1: u8, const P2: u8, M1: LoanMessages, M2: LoanMessages>(
    first: &mut Endpoint<'_, P1>,
    second: &mut Endpoint<'_, P2>,
    module: &mut Endpoint<'_, 0>,
) -> Result<(), RunError<String>> {
    let origin = tokio::select! {
        received = first.recv::<M1::Acquire>() => { received?; Borrower::First }
        received = second.recv::<M2::Acquire>() => { received?; Borrower::Second }
    };
    match origin {
        Borrower::First => {
            module.send::<resource::Request>(&Borrower::First).await?;
            module.recv::<resource::Granted>().await?;
            first.send::<M1::Granted>(&()).await?;
            first.recv::<M1::Return>().await?;
            module.send::<resource::Return>(&()).await?;
            module.recv::<resource::Returned>().await?;
            first.send::<M1::Returned>(&()).await?;
            second.recv::<M2::Acquire>().await?;
            module.send::<resource::Request>(&Borrower::Second).await?;
            module.recv::<resource::Granted>().await?;
            second.send::<M2::Granted>(&()).await?;
            second.recv::<M2::Return>().await?;
            module.send::<resource::Return>(&()).await?;
            module.recv::<resource::Returned>().await?;
            second.send::<M2::Returned>(&()).await?;
        }
        Borrower::Second => {
            module.send::<resource::Request>(&Borrower::Second).await?;
            module.recv::<resource::Granted>().await?;
            second.send::<M2::Granted>(&()).await?;
            second.recv::<M2::Return>().await?;
            module.send::<resource::Return>(&()).await?;
            module.recv::<resource::Returned>().await?;
            second.send::<M2::Returned>(&()).await?;
            first.recv::<M1::Acquire>().await?;
            module.send::<resource::Request>(&Borrower::First).await?;
            module.recv::<resource::Granted>().await?;
            first.send::<M1::Granted>(&()).await?;
            first.recv::<M1::Return>().await?;
            module.send::<resource::Return>(&()).await?;
            module.recv::<resource::Returned>().await?;
            first.send::<M1::Returned>(&()).await?;
        }
    }
    module.send::<resource::Closed>(&()).await?;
    Ok(())
}
