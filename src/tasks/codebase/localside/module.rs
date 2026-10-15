//! Physical module owners follow their one projected session. No broker or owner table.
use super::super::global::*;
use crate::{RunError, resource::global::*};
use hibana::Endpoint;
pub(crate) async fn auth_1e(
    endpoint: &mut Endpoint<'_, AUTH_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    Ok(())
}
pub(crate) async fn database_1e(
    endpoint: &mut Endpoint<'_, DATABASE_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    Ok(())
}
pub(crate) async fn api_1e(
    endpoint: &mut Endpoint<'_, API_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    Ok(())
}
pub(crate) async fn auth_1m(
    endpoint: &mut Endpoint<'_, AUTH_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    Ok(())
}
pub(crate) async fn database_1m(
    endpoint: &mut Endpoint<'_, DATABASE_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    Ok(())
}
pub(crate) async fn api_1m(
    endpoint: &mut Endpoint<'_, API_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    Ok(())
}
pub(crate) async fn payment_1m(
    endpoint: &mut Endpoint<'_, PAYMENT_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    endpoint.send::<AvailableFirst>(&()).await?;
    endpoint.recv::<AcquireFirst>().await?;
    endpoint.send::<GrantedFirst>(&()).await?;
    endpoint.recv::<ReturnFirst>().await?;
    endpoint.send::<ReturnedFirst>(&()).await?;
    Ok(())
}
pub(crate) async fn notify_1m(
    endpoint: &mut Endpoint<'_, NOTIFY_MODULE>,
) -> Result<(), RunError<String>> {
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    endpoint.send::<AvailableSecond>(&()).await?;
    endpoint.recv::<AcquireSecond>().await?;
    endpoint.send::<GrantedSecond>(&()).await?;
    endpoint.recv::<ReturnSecond>().await?;
    endpoint.send::<ReturnedSecond>(&()).await?;
    Ok(())
}
