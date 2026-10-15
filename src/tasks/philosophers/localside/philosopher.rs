use crate::imp::domain::{Domain, RunError, Work, WorkRequest};
use crate::tasks::philosophers::global::{Eat, Finished, ForkMessages, Think};
use hibana::Endpoint;

pub(crate) async fn run<const ROLE: u8, L: ForkMessages, R: ForkMessages, D: Domain>(
    agent: &'static str,
    endpoint: &mut Endpoint<'_, ROLE>,
    domain: &D,
) -> Result<(), RunError<D::Error>> {
    loop {
        let call = domain.request(agent).await.map_err(RunError::Domain)?;
        match call.work() {
            Work::Think => {
                endpoint.send::<Think>(&()).await?;
                domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
            }
            Work::Eat => {
                endpoint.recv::<L::Available>().await?;
                endpoint.send::<L::Acquire>(&()).await?;
                endpoint.recv::<L::Granted>().await?;
                endpoint.recv::<R::Available>().await?;
                endpoint.send::<R::Acquire>(&()).await?;
                endpoint.recv::<R::Granted>().await?;
                endpoint.send::<Eat>(&()).await?;
                domain
                    .execute(agent, call)
                    .await
                    .map_err(RunError::Domain)?;
                endpoint.send::<R::Return>(&()).await?;
                endpoint.recv::<R::Returned>().await?;
                endpoint.send::<L::Return>(&()).await?;
                endpoint.recv::<L::Returned>().await?;
                endpoint.send::<Finished>(&()).await?;
                return Ok(());
            }
        }
    }
}
