//! Physical owners execute the selected developer's distinct loan messages.
use super::super::global::{reconsider::*, *};
use crate::RunError;
use hibana::Endpoint;
pub(crate) async fn auth(endpoint: &mut Endpoint<'_, AUTH_MODULE>) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            200 => {
                branch.recv::<ChooseA>().await?;
                endpoint.send::<<ChooseA as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseA as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseA as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseA as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseA as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseA as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            220 => {
                branch.recv::<SkipA>().await?;
            }
            206 => {
                branch.recv::<ChooseG>().await?;
                endpoint.send::<<ChooseG as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseG as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseG as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseG as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseG as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseG as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            226 => {
                branch.recv::<SkipG>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}

pub(crate) async fn database(
    endpoint: &mut Endpoint<'_, DATABASE_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            200 => {
                branch.recv::<ChooseA>().await?;
                endpoint.send::<<ChooseA as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseA as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseA as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseA as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseA as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseA as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            220 => {
                branch.recv::<SkipA>().await?;
            }
            201 => {
                branch.recv::<ChooseB>().await?;
                endpoint.send::<<ChooseB as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseB as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseB as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseB as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseB as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseB as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            221 => {
                branch.recv::<SkipB>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}

pub(crate) async fn api(endpoint: &mut Endpoint<'_, API_MODULE>) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            201 => {
                branch.recv::<ChooseB>().await?;
                endpoint.send::<<ChooseB as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseB as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseB as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseB as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseB as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseB as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            221 => {
                branch.recv::<SkipB>().await?;
            }
            202 => {
                branch.recv::<ChooseC>().await?;
                endpoint.send::<<ChooseC as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseC as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseC as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseC as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseC as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseC as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            222 => {
                branch.recv::<SkipC>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}

pub(crate) async fn payment(
    endpoint: &mut Endpoint<'_, PAYMENT_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            202 => {
                branch.recv::<ChooseC>().await?;
                endpoint.send::<<ChooseC as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseC as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseC as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseC as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseC as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseC as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            222 => {
                branch.recv::<SkipC>().await?;
            }
            203 => {
                branch.recv::<ChooseD>().await?;
                endpoint.send::<<ChooseD as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseD as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseD as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseD as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseD as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseD as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            223 => {
                branch.recv::<SkipD>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}

pub(crate) async fn notify(
    endpoint: &mut Endpoint<'_, NOTIFY_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            203 => {
                branch.recv::<ChooseD>().await?;
                endpoint.send::<<ChooseD as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseD as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseD as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseD as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseD as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseD as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            223 => {
                branch.recv::<SkipD>().await?;
            }
            204 => {
                branch.recv::<ChooseE>().await?;
                endpoint.send::<<ChooseE as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseE as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseE as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseE as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseE as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseE as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            224 => {
                branch.recv::<SkipE>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}

pub(crate) async fn analytics(
    endpoint: &mut Endpoint<'_, ANALYTICS_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            204 => {
                branch.recv::<ChooseE>().await?;
                endpoint.send::<<ChooseE as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseE as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseE as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseE as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseE as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseE as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            224 => {
                branch.recv::<SkipE>().await?;
            }
            205 => {
                branch.recv::<ChooseF>().await?;
                endpoint.send::<<ChooseF as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseF as Loan>::AvailableFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseF as Loan>::AcquireFirst>().await?;
                endpoint
                    .send::<<ChooseF as Loan>::GrantedFirst>(&())
                    .await?;
                endpoint.recv::<<ChooseF as Loan>::ReturnFirst>().await?;
                endpoint
                    .send::<<ChooseF as Loan>::ReturnedFirst>(&())
                    .await?;
            }
            225 => {
                branch.recv::<SkipF>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}

pub(crate) async fn search(
    endpoint: &mut Endpoint<'_, SEARCH_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            205 => {
                branch.recv::<ChooseF>().await?;
                endpoint.send::<<ChooseF as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseF as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseF as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseF as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseF as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseF as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            225 => {
                branch.recv::<SkipF>().await?;
            }
            206 => {
                branch.recv::<ChooseG>().await?;
                endpoint.send::<<ChooseG as Loan>::Local>(&()).await?;
                endpoint
                    .send::<<ChooseG as Loan>::AvailableSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseG as Loan>::AcquireSecond>().await?;
                endpoint
                    .send::<<ChooseG as Loan>::GrantedSecond>(&())
                    .await?;
                endpoint.recv::<<ChooseG as Loan>::ReturnSecond>().await?;
                endpoint
                    .send::<<ChooseG as Loan>::ReturnedSecond>(&())
                    .await?;
            }
            226 => {
                branch.recv::<SkipG>().await?;
            }
            209 => {
                branch.recv::<Complete>().await?;
                return Ok(());
            }
            _ => return Err(RunError::Domain("unexpected physical module visit".into())),
        }
    }
}
