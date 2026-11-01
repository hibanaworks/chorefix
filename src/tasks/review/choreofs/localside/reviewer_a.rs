//! Actual review requests and results for user_authentication.
use super::super::global::*;
use super::super::imp::ReviewReply;
use crate::{
    shell::{ShellError, ShellResult, imp::service::Domain},
    tasks::review::imp::ReviewInput,
};
use hibana::Endpoint;
use hibana::runtime::resolver::DecisionArm;

#[hibana_runtime::scope]
pub async fn run<D: Domain>(
    endpoint: &mut Endpoint<REVIEW_A>,
    domain: &D,
    review: &ReviewInput,
) -> ShellResult<()> {
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
                let bytes =
                    serde_json::to_vec(&result).map_err(|e| ShellError::message(e.to_string()))?;
                review.record(result);
                match crate::tasks::review::imp::decide(review)
                    .map_err(|_| ShellError::message("review result has no success field"))?
                {
                    DecisionArm::Left => {
                        let reply = ReviewReply::<true>::from_bytes(&bytes)?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                        endpoint.send::<ApprovedA>(&reply).await?;
                    }
                    DecisionArm::Right => {
                        let reply = ReviewReply::<false>::from_bytes(&bytes)?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                        endpoint.send::<RejectedA>(&reply).await?;
                    }
                }
            }
            40 => {
                branch.recv::<ApprovedB>().await?;
            }
            41 => {
                branch.recv::<RejectedB>().await?;
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
