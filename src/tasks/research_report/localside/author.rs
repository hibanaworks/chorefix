use super::super::{
    global::*,
    imp::{Domain, Operation, Request, succeeded},
};
use crate::RunError;
use hibana::{Endpoint, g::Message};

#[hibana_runtime::localside]
pub(crate) async fn author_a<D: Domain>(
    endpoint: &mut Endpoint<A>,
    domain: &D,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain
            .request("RESEARCHER_A")
            .await
            .map_err(RunError::Domain)?;
        match call.operation() {
            Operation::Research => {
                endpoint.send::<Research>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_A", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
            }
            Operation::Write => {
                endpoint.recv::<author_a::DocAvailable>().await?;
                endpoint.send::<author_a::DocRequest>(&()).await?;
                endpoint.recv::<author_a::DocGranted>().await?;
                endpoint.send::<Write>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_A", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
                endpoint.send::<author_a::DocReturn>(&()).await?;
                endpoint.recv::<author_a::DocReturned>().await?;
            }
            Operation::References => {
                endpoint.send::<author_a::RefsRequest>(&()).await?;
                endpoint.recv::<author_a::RefsGranted>().await?;
                endpoint.send::<References>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_A", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
                endpoint.send::<author_a::RefsReturn>(&()).await?;
                endpoint.recv::<author_a::RefsReturned>().await?;
                endpoint.send::<author_a::Ready>(&()).await?;
                endpoint.recv::<author_a::BeginReview>().await?;
                {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == author_a::NeedsRevision::LOGICAL_LABEL => {
                            branch.recv::<author_a::NeedsRevision>().await?;
                            endpoint.send::<author_a::RevisionReady>(&()).await?;
                        }
                        l if l == author_a::Accepted::LOGICAL_LABEL => {
                            branch.recv::<author_a::Accepted>().await?;
                            return Ok(());
                        }

                        _ => return Err(RunError::Domain("unexpected author feedback".into())),
                    }
                }
            }
            Operation::Revise => {
                endpoint.recv::<author_a::RevisionAvailable>().await?;
                endpoint.send::<author_a::RevisionRequest>(&()).await?;
                endpoint.recv::<author_a::RevisionGranted>().await?;
                endpoint.send::<Revise>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_A", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
                endpoint.send::<author_a::RevisionReturn>(&()).await?;
                endpoint.recv::<author_a::RevisionReturned>().await?;
                endpoint.send::<author_a::Resubmitted>(&()).await?;
                {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == author_a::NeedsRevision::LOGICAL_LABEL => {
                            branch.recv::<author_a::NeedsRevision>().await?;
                            endpoint.send::<author_a::RevisionReady>(&()).await?;
                        }
                        l if l == author_a::Accepted::LOGICAL_LABEL => {
                            branch.recv::<author_a::Accepted>().await?;
                            return Ok(());
                        }

                        _ => return Err(RunError::Domain("unexpected author feedback".into())),
                    }
                }
            }
            Operation::ReviewA | Operation::ReviewB | Operation::Combine => {
                return Err(RunError::Domain(
                    "tool does not belong to this author".into(),
                ));
            }
        }
    }
}

#[hibana_runtime::localside]
pub(crate) async fn author_b<D: Domain>(
    endpoint: &mut Endpoint<B>,
    domain: &D,
) -> Result<(), RunError<String>> {
    loop {
        let call = domain
            .request("RESEARCHER_B")
            .await
            .map_err(RunError::Domain)?;
        match call.operation() {
            Operation::Research => {
                endpoint.send::<Research>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_B", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
            }
            Operation::Write => {
                endpoint.recv::<author_b::DocAvailable>().await?;
                endpoint.send::<author_b::DocRequest>(&()).await?;
                endpoint.recv::<author_b::DocGranted>().await?;
                endpoint.send::<Write>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_B", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
                endpoint.send::<author_b::DocReturn>(&()).await?;
                endpoint.recv::<author_b::DocReturned>().await?;
            }
            Operation::References => {
                endpoint.recv::<author_b::RefsAvailable>().await?;
                endpoint.send::<author_b::ReadyForFeedback>(&()).await?;
                endpoint.send::<author_b::RefsRequest>(&()).await?;
                endpoint.recv::<author_b::RefsGranted>().await?;
                endpoint.send::<References>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_B", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
                endpoint.send::<author_b::RefsReturn>(&()).await?;
                endpoint.recv::<author_b::RefsReturned>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == author_b::WaitForARevision::LOGICAL_LABEL => {
                            branch.recv::<author_b::WaitForARevision>().await?;
                        }
                        l if l == author_b::BeginReview::LOGICAL_LABEL => {
                            branch.recv::<author_b::BeginReview>().await?;
                            endpoint.send::<author_b::Ready>(&()).await?;
                            break;
                        }
                        _ => return Err(RunError::Domain("unexpected review notification".into())),
                    }
                }
                {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == author_b::NeedsRevision::LOGICAL_LABEL => {
                            branch.recv::<author_b::NeedsRevision>().await?;
                            endpoint.send::<author_b::RevisionReady>(&()).await?;
                        }
                        l if l == author_b::Accepted::LOGICAL_LABEL => {
                            branch.recv::<author_b::Accepted>().await?;
                            return Ok(());
                        }

                        _ => return Err(RunError::Domain("unexpected author feedback".into())),
                    }
                }
            }
            Operation::Revise => {
                endpoint.recv::<author_b::RevisionAvailable>().await?;
                endpoint.send::<author_b::RevisionRequest>(&()).await?;
                endpoint.recv::<author_b::RevisionGranted>().await?;
                endpoint.send::<Revise>(&()).await?;
                succeeded(
                    &domain
                        .execute("RESEARCHER_B", call)
                        .await
                        .map_err(RunError::Domain)?,
                )?;
                endpoint.send::<author_b::RevisionReturn>(&()).await?;
                endpoint.recv::<author_b::RevisionReturned>().await?;
                endpoint.send::<author_b::Resubmitted>(&()).await?;
                {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == author_b::NeedsRevision::LOGICAL_LABEL => {
                            branch.recv::<author_b::NeedsRevision>().await?;
                            endpoint.send::<author_b::RevisionReady>(&()).await?;
                        }
                        l if l == author_b::Accepted::LOGICAL_LABEL => {
                            branch.recv::<author_b::Accepted>().await?;
                            return Ok(());
                        }

                        _ => return Err(RunError::Domain("unexpected author feedback".into())),
                    }
                }
            }
            Operation::ReviewA | Operation::ReviewB | Operation::Combine => {
                return Err(RunError::Domain(
                    "tool does not belong to this author".into(),
                ));
            }
        }
    }
}
