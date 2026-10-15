//! Translate one task port to its own projected resource conversation.
use super::super::global::*;
use crate::{RunError, resource::global as resource};
use hibana::{Endpoint, g};

pub(crate) async fn run<const ROLE: u8, const APPROVED: u8, const REVISE: u8>(
    endpoint: &mut Endpoint<'_, ROLE>,
    resource: &mut Endpoint<'_, 0>,
) -> Result<(), RunError<String>> {
    endpoint.recv::<Acquire>().await?;
    resource.send::<resource::Request>(&()).await?;
    resource.recv::<resource::Granted>().await?;
    endpoint.send::<Granted>(&()).await?;
    endpoint.recv::<Return>().await?;
    resource.send::<resource::Return>(&()).await?;
    resource.recv::<resource::Returned>().await?;
    endpoint.send::<Returned>(&()).await?;
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            label if label == APPROVED => {
                branch.recv::<g::Msg<APPROVED, ()>>().await?;
                endpoint.recv::<Acquire>().await?;
                resource.send::<resource::Request>(&()).await?;
                resource.recv::<resource::Granted>().await?;
                endpoint.send::<Granted>(&()).await?;
                endpoint.recv::<Return>().await?;
                resource.send::<resource::Return>(&()).await?;
                resource.recv::<resource::Returned>().await?;
                endpoint.send::<Returned>(&()).await?;
                resource.send::<resource::Closed>(&()).await?;
                return Ok(());
            }
            label if label == REVISE => {
                branch.recv::<g::Msg<REVISE, ()>>().await?;
                endpoint.recv::<Acquire>().await?;
                resource.send::<resource::Request>(&()).await?;
                resource.recv::<resource::Granted>().await?;
                endpoint.send::<Granted>(&()).await?;
                endpoint.recv::<Return>().await?;
                resource.send::<resource::Return>(&()).await?;
                resource.recv::<resource::Returned>().await?;
                endpoint.send::<Returned>(&()).await?;
            }
            _ => unreachable!("projected review port decisions"),
        }
    }
}
