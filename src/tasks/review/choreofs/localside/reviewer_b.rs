//! Actual review requests and results for notification_feature.
use super::super::global::*;
use crate::{
    shell::{ShellError, ShellResult, imp::service::Domain},
    tasks::review::imp::ReviewInput,
};
use hibana::Endpoint;

pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEW_B>,
    domain: &D,
    review: &ReviewInput,
) -> ShellResult<()> {
    let mut bytes = Vec::new();
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            171 | 177 => {
                let arguments = match branch.label() {
                    171 => branch.recv::<ReviewB>().await?,
                    177 => branch.recv::<ReReviewB>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["pr_id"] != FEATURE_B {
                    return Err(ShellError::message("review target differs"));
                }
                let result = domain
                    .execute("REVIEWER", "code_review", arguments)
                    .await
                    .map_err(ShellError::message)?;
                bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                review.record(result);
            }
            40 => {
                branch.send::<ApprovedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedB<'_>>(&bytes.as_slice()).await?;
            }
            41 => {
                branch.send::<RejectedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedB<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedB<'_>>(&bytes.as_slice()).await?;
            }
            30 => {
                branch.recv::<ApprovedA<'_>>().await?;
            }
            31 => {
                branch.recv::<RejectedA<'_>>().await?;
            }
            80 => {
                branch.recv::<FirstA>().await?;
            }
            81 => {
                branch.recv::<FirstB>().await?;
            }
            82 => {
                branch.recv::<AgainABeforeB>().await?;
            }
            83 => {
                branch.recv::<BDuringARevision>().await?;
            }
            84 => {
                branch.recv::<BAfterAMerge>().await?;
            }
            85 => {
                branch.recv::<AgainAAfterB>().await?;
            }
            86 => {
                branch.recv::<AgainBBeforeA>().await?;
            }
            87 => {
                branch.recv::<ADuringBRevision>().await?;
            }
            88 => {
                branch.recv::<AAfterBMerge>().await?;
            }
            89 => {
                branch.recv::<AgainBAfterA>().await?;
            }
            92 => {
                branch.recv::<AgainADuringBRevision>().await?;
            }
            93 => {
                branch.recv::<AgainBDuringARevision>().await?;
            }
            94 => {
                branch.recv::<AgainBBeforeAReview>().await?;
            }
            95 => {
                branch.recv::<AgainABeforeBReview>().await?;
            }
            233 => {
                branch.recv::<ReviewExitB>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-B operation {label}"
                )));
            }
        }
    }
}
