use super::super::global::{Attempt, Passed};
use hibana::{Endpoint, EndpointError, g::Message};

pub(crate) async fn run<const ROLE: u8>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> Result<(), EndpointError> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            Passed::LOGICAL_LABEL => {
                branch.recv::<Passed>().await?;
                return Ok(());
            }
            Attempt::LOGICAL_LABEL => {
                branch.recv::<Attempt>().await?;
            }
            _ => unreachable!("projected test alternatives"),
        }
    }
}
