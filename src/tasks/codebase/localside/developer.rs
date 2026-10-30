//! Model inputs are data. Only the projected Endpoint authorizes their effects.
use super::super::{
    global::reconsider::*,
    global::*,
    imp::{Domain, Operation, Request, Requests, response},
};
use crate::{RunError, resource::global::*};
use hibana::{Endpoint, g::Message};

pub(crate) async fn run<
    const ROLE: u8,
    D: Domain,
    Choice: Loan,
    Skip: Message<Payload = ()>,
    Submitted: Message<Payload = ()>,
    Finish: Message<Payload = ()>,
    Done: Message<Payload = ()>,
>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
    requests: &Requests<D::Call>,
) -> Result<(), RunError<String>> {
    // Initial private preparation precedes this role's first shared commit.
    let call = domain.request(agent).await.map_err(RunError::Domain)?;
    let operation = call.operation();
    match operation {
        Operation::Design => endpoint.send::<Design>(&()).await?,
        Operation::Implement => endpoint.send::<Implement>(&()).await?,
        Operation::Commit => endpoint.send::<Commit>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
    }
    let result = domain
        .execute(agent, call)
        .await
        .map_err(RunError::Domain)?;
    response(operation, result)?;
    let call = domain.request(agent).await.map_err(RunError::Domain)?;
    let operation = call.operation();
    match operation {
        Operation::Design => endpoint.send::<Design>(&()).await?,
        Operation::Implement => endpoint.send::<Implement>(&()).await?,
        Operation::Commit => endpoint.send::<Commit>(&()).await?,
        Operation::Test => endpoint.send::<Test>(&()).await?,
    }
    let result = domain
        .execute(agent, call)
        .await
        .map_err(RunError::Domain)?;
    response(operation, result)?;
    loop {
        // Poll the independent model while consuming notifications for other
        // developers. Neither model latency nor an unrelated visit holds a loan.
        let next = domain.request(agent);
        tokio::pin!(next);
        let call = loop {
            tokio::select! {
                call = &mut next => break call.map_err(RunError::Domain)?,
                branch = endpoint.offer() => {
                    let branch = branch?;
                    if branch.label() == Skip::LOGICAL_LABEL {
                        branch.recv::<Skip>().await?;
                    } else {
                        return Err(RunError::Domain("unexpected choice while model computes".into()));
                    }
                }
            }
        };
        // These are the actual, unconsumed model calls. Submitted acknowledges
        // their removal before the dispatcher can select another request.
        requests.pending.borrow_mut().push_back((ROLE, call));
        requests.available.notify_waiters();
        loop {
            let branch = endpoint.offer().await?;
            if branch.label() == Choice::LOGICAL_LABEL {
                branch.recv::<Choice>().await?;
                break;
            }
            if branch.label() == Skip::LOGICAL_LABEL {
                branch.recv::<Skip>().await?;
            } else {
                return Err(RunError::Domain("unexpected request selection".into()));
            }
        }
        let call = {
            let mut pending = requests.pending.borrow_mut();
            let index = pending
                .iter()
                .position(|(role, _)| *role == ROLE)
                .ok_or_else(|| RunError::Domain("selected request is absent".into()))?;
            pending
                .remove(index)
                .expect("position came from the same queue")
                .1
        };
        endpoint.send::<Submitted>(&()).await?;
        endpoint.send::<Choice::Local>(&()).await?;
        endpoint.recv::<Choice::AvailableFirst>().await?;
        endpoint.recv::<Choice::AvailableSecond>().await?;
        endpoint.send::<Choice::AcquireFirst>(&()).await?;
        endpoint.send::<Choice::AcquireSecond>(&()).await?;
        endpoint.recv::<Choice::GrantedFirst>().await?;
        endpoint.recv::<Choice::GrantedSecond>().await?;
        endpoint.send::<Commit>(&()).await?;
        let result = domain
            .execute(agent, call)
            .await
            .map_err(RunError::Domain)?;
        response(Operation::Commit, result)?;
        endpoint.send::<Choice::ReturnFirst>(&()).await?;
        endpoint.send::<Choice::ReturnSecond>(&()).await?;
        endpoint.recv::<Choice::ReturnedFirst>().await?;
        endpoint.recv::<Choice::ReturnedSecond>().await?;

        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        let operation = call.operation();
        match operation {
            Operation::Design => endpoint.send::<Design>(&()).await?,
            Operation::Implement => endpoint.send::<Implement>(&()).await?,
            Operation::Commit => endpoint.send::<Commit>(&()).await?,
            Operation::Test => endpoint.send::<Test>(&()).await?,
        }
        let result = domain
            .execute(agent, call)
            .await
            .map_err(RunError::Domain)?;
        requests.results[ROLE as usize].record(response(operation, result)?);
        requests.available.notify_waiters();
        let branch = endpoint.offer().await?;
        match branch.label() {
            5 => {
                branch.recv::<Finished>().await?;
                break;
            }
            2 => {
                // The resolver's rejected-test branch requires a new actual
                // implementation, rather than repeating a failed tool call.
                let call = domain.request(agent).await.map_err(RunError::Domain)?;
                let operation = call.operation();
                match operation {
                    Operation::Design => {
                        branch.recv::<Design>().await?;
                    }
                    Operation::Implement => {
                        branch.recv::<Implement>().await?;
                    }
                    Operation::Commit => {
                        branch.recv::<Commit>().await?;
                    }
                    Operation::Test => {
                        branch.recv::<Test>().await?;
                    }
                }
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            _ => return Err(RunError::Domain("unexpected test outcome".into())),
        }
    }
    // The explicit final handshake proves completion once, after the successful branch.
    loop {
        let branch = endpoint.offer().await?;
        if branch.label() == Skip::LOGICAL_LABEL {
            branch.recv::<Skip>().await?;
        } else if branch.label() == Finish::LOGICAL_LABEL {
            branch.recv::<Finish>().await?;
            endpoint.send::<Done>(&()).await?;
            break;
        } else {
            return Err(RunError::Domain("unexpected final handshake".into()));
        }
    }
    endpoint.recv::<Complete>().await?;
    Ok(())
}

/// 1E/1M tests are terminal, including a failed test result.
pub(crate) async fn terminal<const ROLE: u8, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        let operation = call.operation();
        match operation {
            Operation::Design => {
                endpoint.send::<Design>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Implement => {
                endpoint.send::<Implement>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
            }
            Operation::Commit => {
                endpoint.recv::<AvailableFirst>().await?;
                endpoint.recv::<AvailableSecond>().await?;
                endpoint.send::<AcquireFirst>(&()).await?;
                endpoint.send::<AcquireSecond>(&()).await?;
                endpoint.recv::<GrantedFirst>().await?;
                endpoint.recv::<GrantedSecond>().await?;
                endpoint.send::<Commit>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
                endpoint.send::<ReturnSecond>(&()).await?;
                endpoint.send::<ReturnFirst>(&()).await?;
                endpoint.recv::<ReturnedSecond>().await?;
                endpoint.recv::<ReturnedFirst>().await?;
            }
            Operation::Test => {
                endpoint.send::<Test>(&()).await?;
                let result = domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                response(operation, result)?;
                endpoint.send::<Finished>(&()).await?;
                return Ok(());
            }
        }
    }
}
