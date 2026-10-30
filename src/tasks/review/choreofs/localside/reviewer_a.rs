//! Actual review requests and results for user_authentication.
use super::super::global::*;
use crate::{
    shell::{ShellError, ShellResult, imp::service::Domain},
    tasks::review::imp::ReviewInput,
};
use hibana::Endpoint;

pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<'_, REVIEW_A>,
    domain: &D,
    review: &ReviewInput,
) -> ShellResult<()> {
    let mut bytes = Vec::new();
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            71 | 77 => {
                let arguments = match branch.label() {
                    71 => branch.recv::<ReviewA>().await?,
                    77 => branch.recv::<ReReviewA>().await?,
                    _ => unreachable!(),
                }
                .into_value();
                if arguments["pr_id"] != FEATURE_A {
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
            30 => {
                branch.send::<ApprovedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<ApprovedA<'_>>(&bytes.as_slice()).await?;
            }
            31 => {
                branch.send::<RejectedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedA<'_>>(&bytes.as_slice()).await?;
                endpoint.send::<RejectedA<'_>>(&bytes.as_slice()).await?;
            }
            40 => {
                branch.recv::<ApprovedB<'_>>().await?;
            }
            41 => {
                branch.recv::<RejectedB<'_>>().await?;
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
            232 => {
                branch.recv::<ReviewExitA>().await?;
                return Ok(());
            }
            label => {
                return Err(ShellError::message(format!(
                    "unexpected review-A operation {label}"
                )));
            }
        }
    }
}
