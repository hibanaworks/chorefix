//! Drive the global's optional visits using the actual pending model requests.
use super::super::{
    global::reconsider::*,
    imp::{Request, Requests, ready},
};
use crate::RunError;
use hibana::{Endpoint, runtime::resolver::DecisionArm};
pub(crate) async fn run<C: Request>(
    endpoint: &mut Endpoint<'_, DISPATCH>,
    requests: &Requests<C>,
) -> Result<(), RunError<String>> {
    loop {
        if requests.all_passed() {
            endpoint.send::<FinishA>(&()).await?;
            endpoint.recv::<DoneA>().await?;
            endpoint.send::<FinishB>(&()).await?;
            endpoint.recv::<DoneB>().await?;
            endpoint.send::<FinishC>(&()).await?;
            endpoint.recv::<DoneC>().await?;
            endpoint.send::<FinishD>(&()).await?;
            endpoint.recv::<DoneD>().await?;
            endpoint.send::<FinishE>(&()).await?;
            endpoint.recv::<DoneE>().await?;
            endpoint.send::<FinishF>(&()).await?;
            endpoint.recv::<DoneF>().await?;
            endpoint.send::<FinishG>(&()).await?;
            endpoint.recv::<DoneG>().await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            endpoint.send::<Complete>(&()).await?;
            return Ok(());
        }
        if requests.pending.borrow().is_empty() {
            requests.available.notified().await;
            continue;
        }
        match ready::<0, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseA>(&()).await?;
                endpoint.send::<ChooseA>(&()).await?;
                endpoint.send::<ChooseA>(&()).await?;
                endpoint.recv::<SubmittedA>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipA>(&()).await?;
                endpoint.send::<SkipA>(&()).await?;
                endpoint.send::<SkipA>(&()).await?;
            }
        }
        match ready::<2, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseC>(&()).await?;
                endpoint.send::<ChooseC>(&()).await?;
                endpoint.send::<ChooseC>(&()).await?;
                endpoint.recv::<SubmittedC>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipC>(&()).await?;
                endpoint.send::<SkipC>(&()).await?;
                endpoint.send::<SkipC>(&()).await?;
            }
        }
        match ready::<4, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseE>(&()).await?;
                endpoint.send::<ChooseE>(&()).await?;
                endpoint.send::<ChooseE>(&()).await?;
                endpoint.recv::<SubmittedE>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipE>(&()).await?;
                endpoint.send::<SkipE>(&()).await?;
                endpoint.send::<SkipE>(&()).await?;
            }
        }
        match ready::<1, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseB>(&()).await?;
                endpoint.send::<ChooseB>(&()).await?;
                endpoint.send::<ChooseB>(&()).await?;
                endpoint.recv::<SubmittedB>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipB>(&()).await?;
                endpoint.send::<SkipB>(&()).await?;
                endpoint.send::<SkipB>(&()).await?;
            }
        }
        match ready::<3, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseD>(&()).await?;
                endpoint.send::<ChooseD>(&()).await?;
                endpoint.send::<ChooseD>(&()).await?;
                endpoint.recv::<SubmittedD>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipD>(&()).await?;
                endpoint.send::<SkipD>(&()).await?;
                endpoint.send::<SkipD>(&()).await?;
            }
        }
        match ready::<5, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseF>(&()).await?;
                endpoint.send::<ChooseF>(&()).await?;
                endpoint.send::<ChooseF>(&()).await?;
                endpoint.recv::<SubmittedF>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipF>(&()).await?;
                endpoint.send::<SkipF>(&()).await?;
                endpoint.send::<SkipF>(&()).await?;
            }
        }
        match ready::<6, C>(requests).map_err(RunError::Resolver)? {
            DecisionArm::Left => {
                endpoint.send::<ChooseG>(&()).await?;
                endpoint.send::<ChooseG>(&()).await?;
                endpoint.send::<ChooseG>(&()).await?;
                endpoint.recv::<SubmittedG>().await?;
            }
            DecisionArm::Right => {
                endpoint.send::<SkipG>(&()).await?;
                endpoint.send::<SkipG>(&()).await?;
                endpoint.send::<SkipG>(&()).await?;
            }
        }
    }
}
