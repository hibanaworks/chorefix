use super::super::global::{author_a as a, author_b as b, *};
use crate::RunError;
use hibana::{Endpoint, g::Message};

pub(crate) async fn database(
    endpoint: &mut Endpoint<'_, DATABASE>,
) -> Result<(), RunError<String>> {
    endpoint.recv::<a::RefsRequest>().await?;
    endpoint.send::<a::RefsGranted>(&()).await?;
    endpoint.recv::<a::RefsReturn>().await?;
    endpoint.send::<a::RefsReturned>(&()).await?;
    endpoint.send::<b::RefsAvailable>(&()).await?;
    endpoint.recv::<b::RefsRequest>().await?;
    endpoint.send::<b::RefsGranted>(&()).await?;
    endpoint.recv::<b::RefsReturn>().await?;
    endpoint.send::<b::RefsReturned>(&()).await?;
    Ok(())
}
pub(crate) async fn document(
    endpoint: &mut Endpoint<'_, DOCUMENT>,
) -> Result<(), RunError<String>> {
    endpoint.send::<a::DocAvailable>(&()).await?;
    endpoint.recv::<a::DocRequest>().await?;
    endpoint.send::<a::DocGranted>(&()).await?;
    endpoint.recv::<a::DocReturn>().await?;
    endpoint.send::<a::DocReturned>(&()).await?;
    endpoint.send::<b::DocAvailable>(&()).await?;
    endpoint.recv::<b::DocRequest>().await?;
    endpoint.send::<b::DocGranted>(&()).await?;
    endpoint.recv::<b::DocReturn>().await?;
    endpoint.send::<b::DocReturned>(&()).await?;
    endpoint.recv::<a::BeginDocumentReview>().await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == a::NeedsDocumentRevision::LOGICAL_LABEL => {
                branch.recv::<a::NeedsDocumentRevision>().await?;
                endpoint.send::<a::RevisionAvailable>(&()).await?;
                endpoint.recv::<a::RevisionRequest>().await?;
                endpoint.send::<a::RevisionGranted>(&()).await?;
                endpoint.recv::<a::RevisionReturn>().await?;
                endpoint.send::<a::RevisionReturned>(&()).await?;
            }
            l if l == b::BeginDocumentReview::LOGICAL_LABEL => {
                branch.recv::<b::BeginDocumentReview>().await?;
                break;
            }

            label => {
                return Err(RunError::Domain(format!(
                    "unexpected document revision A: {label}"
                )));
            }
        }
    }
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == b::NeedsDocumentRevision::LOGICAL_LABEL => {
                branch.recv::<b::NeedsDocumentRevision>().await?;
                endpoint.send::<b::RevisionAvailable>(&()).await?;
                endpoint.recv::<b::RevisionRequest>().await?;
                endpoint.send::<b::RevisionGranted>(&()).await?;
                endpoint.recv::<b::RevisionReturn>().await?;
                endpoint.send::<b::RevisionReturned>(&()).await?;
            }
            l if l == BeginCombine::LOGICAL_LABEL => {
                branch.recv::<BeginCombine>().await?;
                break;
            }

            label => {
                return Err(RunError::Domain(format!(
                    "unexpected document revision B: {label}"
                )));
            }
        }
    }
    endpoint.recv::<CombineRequest>().await?;
    endpoint.send::<CombineGranted>(&()).await?;
    endpoint.recv::<CombineReturn>().await?;
    endpoint.send::<CombineReturned>(&()).await?;
    Ok(())
}
