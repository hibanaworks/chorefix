//! One affine Endpoint per physical module. Receiver lifetimes consume closure.
use super::super::global::{
    reconsider::{a, b, c, d, e, f, g_messages},
    *,
};
use crate::RunError;
use hibana::{Endpoint, g::Message};
pub(crate) async fn auth(endpoint: &mut Endpoint<'_, AUTH_MODULE>) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == a::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<a::AcquireFirst>().await?;
                endpoint.send::<a::Handoff>(&()).await?;
                endpoint.recv::<a::SecondReady>().await?;
                endpoint.send::<a::GrantedFirst>(&()).await?;
                endpoint.recv::<a::ReturnFirst>().await?;
                endpoint.send::<a::ReturnedFirst>(&()).await?;
            }
            l if l == a::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<a::CloseFirst>().await?;
                endpoint.send::<a::CloseHandoff>(&()).await?;
                endpoint.recv::<a::SecondReady>().await?;
                endpoint.send::<a::ClosePrimed>(&()).await?;
                endpoint.recv::<a::CloseFirstRequest>().await?;
                endpoint.send::<a::ClosedFirst>(&()).await?;
                endpoint.recv::<a::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == g_messages::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<g_messages::AcquireFirst>().await?;
                            endpoint.send::<g_messages::Handoff>(&()).await?;
                            endpoint.recv::<g_messages::SecondReady>().await?;
                            endpoint.send::<g_messages::GrantedFirst>(&()).await?;
                            endpoint.recv::<g_messages::ReturnFirst>().await?;
                            endpoint.send::<g_messages::ReturnedFirst>(&()).await?;
                        }
                        l if l == g_messages::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<g_messages::CloseFirst>().await?;
                            endpoint.send::<g_messages::CloseHandoff>(&()).await?;
                            endpoint.recv::<g_messages::SecondReady>().await?;
                            endpoint.send::<g_messages::ClosePrimed>(&()).await?;
                            endpoint.recv::<g_messages::CloseFirstRequest>().await?;
                            endpoint.send::<g_messages::ClosedFirst>(&()).await?;
                            endpoint.recv::<g_messages::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == g_messages::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<g_messages::AcquireFirst>().await?;
                endpoint.send::<g_messages::Handoff>(&()).await?;
                endpoint.recv::<g_messages::SecondReady>().await?;
                endpoint.send::<g_messages::GrantedFirst>(&()).await?;
                endpoint.recv::<g_messages::ReturnFirst>().await?;
                endpoint.send::<g_messages::ReturnedFirst>(&()).await?;
            }
            l if l == g_messages::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<g_messages::CloseFirst>().await?;
                endpoint.send::<g_messages::CloseHandoff>(&()).await?;
                endpoint.recv::<g_messages::SecondReady>().await?;
                endpoint.send::<g_messages::ClosePrimed>(&()).await?;
                endpoint.recv::<g_messages::CloseFirstRequest>().await?;
                endpoint.send::<g_messages::ClosedFirst>(&()).await?;
                endpoint.recv::<g_messages::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == a::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<a::AcquireFirst>().await?;
                            endpoint.send::<a::Handoff>(&()).await?;
                            endpoint.recv::<a::SecondReady>().await?;
                            endpoint.send::<a::GrantedFirst>(&()).await?;
                            endpoint.recv::<a::ReturnFirst>().await?;
                            endpoint.send::<a::ReturnedFirst>(&()).await?;
                        }
                        l if l == a::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<a::CloseFirst>().await?;
                            endpoint.send::<a::CloseHandoff>(&()).await?;
                            endpoint.recv::<a::SecondReady>().await?;
                            endpoint.send::<a::ClosePrimed>(&()).await?;
                            endpoint.recv::<a::CloseFirstRequest>().await?;
                            endpoint.send::<a::ClosedFirst>(&()).await?;
                            endpoint.recv::<a::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
pub(crate) async fn database(
    endpoint: &mut Endpoint<'_, DATABASE_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == a::Handoff::LOGICAL_LABEL => {
                branch.recv::<a::Handoff>().await?;
                endpoint.send::<a::SecondReady>(&()).await?;
                endpoint.recv::<a::AcquireSecond>().await?;
                endpoint.recv::<a::ConfirmSecond>().await?;
                endpoint.send::<a::GrantedSecond>(&()).await?;
                endpoint.recv::<a::ReturnSecond>().await?;
                endpoint.send::<a::ReturnedSecond>(&()).await?;
            }
            l if l == a::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<a::CloseHandoff>().await?;
                endpoint.send::<a::SecondReady>(&()).await?;
                endpoint.recv::<a::AcquireSecond>().await?;
                endpoint.recv::<a::CloseSecondRequest>().await?;
                endpoint.send::<a::ClosedSecond>(&()).await?;
                endpoint.recv::<a::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == b::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<b::AcquireFirst>().await?;
                            endpoint.send::<b::Handoff>(&()).await?;
                            endpoint.recv::<b::SecondReady>().await?;
                            endpoint.send::<b::GrantedFirst>(&()).await?;
                            endpoint.recv::<b::ReturnFirst>().await?;
                            endpoint.send::<b::ReturnedFirst>(&()).await?;
                        }
                        l if l == b::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<b::CloseFirst>().await?;
                            endpoint.send::<b::CloseHandoff>(&()).await?;
                            endpoint.recv::<b::SecondReady>().await?;
                            endpoint.send::<b::ClosePrimed>(&()).await?;
                            endpoint.recv::<b::CloseFirstRequest>().await?;
                            endpoint.send::<b::ClosedFirst>(&()).await?;
                            endpoint.recv::<b::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == b::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<b::AcquireFirst>().await?;
                endpoint.send::<b::Handoff>(&()).await?;
                endpoint.recv::<b::SecondReady>().await?;
                endpoint.send::<b::GrantedFirst>(&()).await?;
                endpoint.recv::<b::ReturnFirst>().await?;
                endpoint.send::<b::ReturnedFirst>(&()).await?;
            }
            l if l == b::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<b::CloseFirst>().await?;
                endpoint.send::<b::CloseHandoff>(&()).await?;
                endpoint.recv::<b::SecondReady>().await?;
                endpoint.send::<b::ClosePrimed>(&()).await?;
                endpoint.recv::<b::CloseFirstRequest>().await?;
                endpoint.send::<b::ClosedFirst>(&()).await?;
                endpoint.recv::<b::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == a::Handoff::LOGICAL_LABEL => {
                            branch.recv::<a::Handoff>().await?;
                            endpoint.send::<a::SecondReady>(&()).await?;
                            endpoint.recv::<a::AcquireSecond>().await?;
                            endpoint.recv::<a::ConfirmSecond>().await?;
                            endpoint.send::<a::GrantedSecond>(&()).await?;
                            endpoint.recv::<a::ReturnSecond>().await?;
                            endpoint.send::<a::ReturnedSecond>(&()).await?;
                        }
                        l if l == a::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<a::CloseHandoff>().await?;
                            endpoint.send::<a::SecondReady>(&()).await?;
                            endpoint.recv::<a::AcquireSecond>().await?;
                            endpoint.recv::<a::CloseSecondRequest>().await?;
                            endpoint.send::<a::ClosedSecond>(&()).await?;
                            endpoint.recv::<a::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
pub(crate) async fn api(endpoint: &mut Endpoint<'_, API_MODULE>) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == b::Handoff::LOGICAL_LABEL => {
                branch.recv::<b::Handoff>().await?;
                endpoint.send::<b::SecondReady>(&()).await?;
                endpoint.recv::<b::AcquireSecond>().await?;
                endpoint.recv::<b::ConfirmSecond>().await?;
                endpoint.send::<b::GrantedSecond>(&()).await?;
                endpoint.recv::<b::ReturnSecond>().await?;
                endpoint.send::<b::ReturnedSecond>(&()).await?;
            }
            l if l == b::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<b::CloseHandoff>().await?;
                endpoint.send::<b::SecondReady>(&()).await?;
                endpoint.recv::<b::AcquireSecond>().await?;
                endpoint.recv::<b::CloseSecondRequest>().await?;
                endpoint.send::<b::ClosedSecond>(&()).await?;
                endpoint.recv::<b::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == c::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<c::AcquireFirst>().await?;
                            endpoint.send::<c::Handoff>(&()).await?;
                            endpoint.recv::<c::SecondReady>().await?;
                            endpoint.send::<c::GrantedFirst>(&()).await?;
                            endpoint.recv::<c::ReturnFirst>().await?;
                            endpoint.send::<c::ReturnedFirst>(&()).await?;
                        }
                        l if l == c::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<c::CloseFirst>().await?;
                            endpoint.send::<c::CloseHandoff>(&()).await?;
                            endpoint.recv::<c::SecondReady>().await?;
                            endpoint.send::<c::ClosePrimed>(&()).await?;
                            endpoint.recv::<c::CloseFirstRequest>().await?;
                            endpoint.send::<c::ClosedFirst>(&()).await?;
                            endpoint.recv::<c::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == c::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<c::AcquireFirst>().await?;
                endpoint.send::<c::Handoff>(&()).await?;
                endpoint.recv::<c::SecondReady>().await?;
                endpoint.send::<c::GrantedFirst>(&()).await?;
                endpoint.recv::<c::ReturnFirst>().await?;
                endpoint.send::<c::ReturnedFirst>(&()).await?;
            }
            l if l == c::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<c::CloseFirst>().await?;
                endpoint.send::<c::CloseHandoff>(&()).await?;
                endpoint.recv::<c::SecondReady>().await?;
                endpoint.send::<c::ClosePrimed>(&()).await?;
                endpoint.recv::<c::CloseFirstRequest>().await?;
                endpoint.send::<c::ClosedFirst>(&()).await?;
                endpoint.recv::<c::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == b::Handoff::LOGICAL_LABEL => {
                            branch.recv::<b::Handoff>().await?;
                            endpoint.send::<b::SecondReady>(&()).await?;
                            endpoint.recv::<b::AcquireSecond>().await?;
                            endpoint.recv::<b::ConfirmSecond>().await?;
                            endpoint.send::<b::GrantedSecond>(&()).await?;
                            endpoint.recv::<b::ReturnSecond>().await?;
                            endpoint.send::<b::ReturnedSecond>(&()).await?;
                        }
                        l if l == b::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<b::CloseHandoff>().await?;
                            endpoint.send::<b::SecondReady>(&()).await?;
                            endpoint.recv::<b::AcquireSecond>().await?;
                            endpoint.recv::<b::CloseSecondRequest>().await?;
                            endpoint.send::<b::ClosedSecond>(&()).await?;
                            endpoint.recv::<b::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
pub(crate) async fn payment(
    endpoint: &mut Endpoint<'_, PAYMENT_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == c::Handoff::LOGICAL_LABEL => {
                branch.recv::<c::Handoff>().await?;
                endpoint.send::<c::SecondReady>(&()).await?;
                endpoint.recv::<c::AcquireSecond>().await?;
                endpoint.recv::<c::ConfirmSecond>().await?;
                endpoint.send::<c::GrantedSecond>(&()).await?;
                endpoint.recv::<c::ReturnSecond>().await?;
                endpoint.send::<c::ReturnedSecond>(&()).await?;
            }
            l if l == c::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<c::CloseHandoff>().await?;
                endpoint.send::<c::SecondReady>(&()).await?;
                endpoint.recv::<c::AcquireSecond>().await?;
                endpoint.recv::<c::CloseSecondRequest>().await?;
                endpoint.send::<c::ClosedSecond>(&()).await?;
                endpoint.recv::<c::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == d::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<d::AcquireFirst>().await?;
                            endpoint.send::<d::Handoff>(&()).await?;
                            endpoint.recv::<d::SecondReady>().await?;
                            endpoint.send::<d::GrantedFirst>(&()).await?;
                            endpoint.recv::<d::ReturnFirst>().await?;
                            endpoint.send::<d::ReturnedFirst>(&()).await?;
                        }
                        l if l == d::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<d::CloseFirst>().await?;
                            endpoint.send::<d::CloseHandoff>(&()).await?;
                            endpoint.recv::<d::SecondReady>().await?;
                            endpoint.send::<d::ClosePrimed>(&()).await?;
                            endpoint.recv::<d::CloseFirstRequest>().await?;
                            endpoint.send::<d::ClosedFirst>(&()).await?;
                            endpoint.recv::<d::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == d::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<d::AcquireFirst>().await?;
                endpoint.send::<d::Handoff>(&()).await?;
                endpoint.recv::<d::SecondReady>().await?;
                endpoint.send::<d::GrantedFirst>(&()).await?;
                endpoint.recv::<d::ReturnFirst>().await?;
                endpoint.send::<d::ReturnedFirst>(&()).await?;
            }
            l if l == d::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<d::CloseFirst>().await?;
                endpoint.send::<d::CloseHandoff>(&()).await?;
                endpoint.recv::<d::SecondReady>().await?;
                endpoint.send::<d::ClosePrimed>(&()).await?;
                endpoint.recv::<d::CloseFirstRequest>().await?;
                endpoint.send::<d::ClosedFirst>(&()).await?;
                endpoint.recv::<d::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == c::Handoff::LOGICAL_LABEL => {
                            branch.recv::<c::Handoff>().await?;
                            endpoint.send::<c::SecondReady>(&()).await?;
                            endpoint.recv::<c::AcquireSecond>().await?;
                            endpoint.recv::<c::ConfirmSecond>().await?;
                            endpoint.send::<c::GrantedSecond>(&()).await?;
                            endpoint.recv::<c::ReturnSecond>().await?;
                            endpoint.send::<c::ReturnedSecond>(&()).await?;
                        }
                        l if l == c::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<c::CloseHandoff>().await?;
                            endpoint.send::<c::SecondReady>(&()).await?;
                            endpoint.recv::<c::AcquireSecond>().await?;
                            endpoint.recv::<c::CloseSecondRequest>().await?;
                            endpoint.send::<c::ClosedSecond>(&()).await?;
                            endpoint.recv::<c::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
pub(crate) async fn notify(
    endpoint: &mut Endpoint<'_, NOTIFY_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == d::Handoff::LOGICAL_LABEL => {
                branch.recv::<d::Handoff>().await?;
                endpoint.send::<d::SecondReady>(&()).await?;
                endpoint.recv::<d::AcquireSecond>().await?;
                endpoint.recv::<d::ConfirmSecond>().await?;
                endpoint.send::<d::GrantedSecond>(&()).await?;
                endpoint.recv::<d::ReturnSecond>().await?;
                endpoint.send::<d::ReturnedSecond>(&()).await?;
            }
            l if l == d::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<d::CloseHandoff>().await?;
                endpoint.send::<d::SecondReady>(&()).await?;
                endpoint.recv::<d::AcquireSecond>().await?;
                endpoint.recv::<d::CloseSecondRequest>().await?;
                endpoint.send::<d::ClosedSecond>(&()).await?;
                endpoint.recv::<d::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == e::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<e::AcquireFirst>().await?;
                            endpoint.send::<e::Handoff>(&()).await?;
                            endpoint.recv::<e::SecondReady>().await?;
                            endpoint.send::<e::GrantedFirst>(&()).await?;
                            endpoint.recv::<e::ReturnFirst>().await?;
                            endpoint.send::<e::ReturnedFirst>(&()).await?;
                        }
                        l if l == e::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<e::CloseFirst>().await?;
                            endpoint.send::<e::CloseHandoff>(&()).await?;
                            endpoint.recv::<e::SecondReady>().await?;
                            endpoint.send::<e::ClosePrimed>(&()).await?;
                            endpoint.recv::<e::CloseFirstRequest>().await?;
                            endpoint.send::<e::ClosedFirst>(&()).await?;
                            endpoint.recv::<e::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == e::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<e::AcquireFirst>().await?;
                endpoint.send::<e::Handoff>(&()).await?;
                endpoint.recv::<e::SecondReady>().await?;
                endpoint.send::<e::GrantedFirst>(&()).await?;
                endpoint.recv::<e::ReturnFirst>().await?;
                endpoint.send::<e::ReturnedFirst>(&()).await?;
            }
            l if l == e::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<e::CloseFirst>().await?;
                endpoint.send::<e::CloseHandoff>(&()).await?;
                endpoint.recv::<e::SecondReady>().await?;
                endpoint.send::<e::ClosePrimed>(&()).await?;
                endpoint.recv::<e::CloseFirstRequest>().await?;
                endpoint.send::<e::ClosedFirst>(&()).await?;
                endpoint.recv::<e::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == d::Handoff::LOGICAL_LABEL => {
                            branch.recv::<d::Handoff>().await?;
                            endpoint.send::<d::SecondReady>(&()).await?;
                            endpoint.recv::<d::AcquireSecond>().await?;
                            endpoint.recv::<d::ConfirmSecond>().await?;
                            endpoint.send::<d::GrantedSecond>(&()).await?;
                            endpoint.recv::<d::ReturnSecond>().await?;
                            endpoint.send::<d::ReturnedSecond>(&()).await?;
                        }
                        l if l == d::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<d::CloseHandoff>().await?;
                            endpoint.send::<d::SecondReady>(&()).await?;
                            endpoint.recv::<d::AcquireSecond>().await?;
                            endpoint.recv::<d::CloseSecondRequest>().await?;
                            endpoint.send::<d::ClosedSecond>(&()).await?;
                            endpoint.recv::<d::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
pub(crate) async fn analytics(
    endpoint: &mut Endpoint<'_, ANALYTICS_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == e::Handoff::LOGICAL_LABEL => {
                branch.recv::<e::Handoff>().await?;
                endpoint.send::<e::SecondReady>(&()).await?;
                endpoint.recv::<e::AcquireSecond>().await?;
                endpoint.recv::<e::ConfirmSecond>().await?;
                endpoint.send::<e::GrantedSecond>(&()).await?;
                endpoint.recv::<e::ReturnSecond>().await?;
                endpoint.send::<e::ReturnedSecond>(&()).await?;
            }
            l if l == e::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<e::CloseHandoff>().await?;
                endpoint.send::<e::SecondReady>(&()).await?;
                endpoint.recv::<e::AcquireSecond>().await?;
                endpoint.recv::<e::CloseSecondRequest>().await?;
                endpoint.send::<e::ClosedSecond>(&()).await?;
                endpoint.recv::<e::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == f::AcquireFirst::LOGICAL_LABEL => {
                            branch.recv::<f::AcquireFirst>().await?;
                            endpoint.send::<f::Handoff>(&()).await?;
                            endpoint.recv::<f::SecondReady>().await?;
                            endpoint.send::<f::GrantedFirst>(&()).await?;
                            endpoint.recv::<f::ReturnFirst>().await?;
                            endpoint.send::<f::ReturnedFirst>(&()).await?;
                        }
                        l if l == f::CloseFirst::LOGICAL_LABEL => {
                            branch.recv::<f::CloseFirst>().await?;
                            endpoint.send::<f::CloseHandoff>(&()).await?;
                            endpoint.recv::<f::SecondReady>().await?;
                            endpoint.send::<f::ClosePrimed>(&()).await?;
                            endpoint.recv::<f::CloseFirstRequest>().await?;
                            endpoint.send::<f::ClosedFirst>(&()).await?;
                            endpoint.recv::<f::DoneFirst>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == f::AcquireFirst::LOGICAL_LABEL => {
                branch.recv::<f::AcquireFirst>().await?;
                endpoint.send::<f::Handoff>(&()).await?;
                endpoint.recv::<f::SecondReady>().await?;
                endpoint.send::<f::GrantedFirst>(&()).await?;
                endpoint.recv::<f::ReturnFirst>().await?;
                endpoint.send::<f::ReturnedFirst>(&()).await?;
            }
            l if l == f::CloseFirst::LOGICAL_LABEL => {
                branch.recv::<f::CloseFirst>().await?;
                endpoint.send::<f::CloseHandoff>(&()).await?;
                endpoint.recv::<f::SecondReady>().await?;
                endpoint.send::<f::ClosePrimed>(&()).await?;
                endpoint.recv::<f::CloseFirstRequest>().await?;
                endpoint.send::<f::ClosedFirst>(&()).await?;
                endpoint.recv::<f::DoneFirst>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == e::Handoff::LOGICAL_LABEL => {
                            branch.recv::<e::Handoff>().await?;
                            endpoint.send::<e::SecondReady>(&()).await?;
                            endpoint.recv::<e::AcquireSecond>().await?;
                            endpoint.recv::<e::ConfirmSecond>().await?;
                            endpoint.send::<e::GrantedSecond>(&()).await?;
                            endpoint.recv::<e::ReturnSecond>().await?;
                            endpoint.send::<e::ReturnedSecond>(&()).await?;
                        }
                        l if l == e::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<e::CloseHandoff>().await?;
                            endpoint.send::<e::SecondReady>(&()).await?;
                            endpoint.recv::<e::AcquireSecond>().await?;
                            endpoint.recv::<e::CloseSecondRequest>().await?;
                            endpoint.send::<e::ClosedSecond>(&()).await?;
                            endpoint.recv::<e::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
pub(crate) async fn search(
    endpoint: &mut Endpoint<'_, SEARCH_MODULE>,
) -> Result<(), RunError<String>> {
    loop {
        let branch = endpoint.offer().await?;
        match branch.label() {
            l if l == f::Handoff::LOGICAL_LABEL => {
                branch.recv::<f::Handoff>().await?;
                endpoint.send::<f::SecondReady>(&()).await?;
                endpoint.recv::<f::AcquireSecond>().await?;
                endpoint.recv::<f::ConfirmSecond>().await?;
                endpoint.send::<f::GrantedSecond>(&()).await?;
                endpoint.recv::<f::ReturnSecond>().await?;
                endpoint.send::<f::ReturnedSecond>(&()).await?;
            }
            l if l == f::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<f::CloseHandoff>().await?;
                endpoint.send::<f::SecondReady>(&()).await?;
                endpoint.recv::<f::AcquireSecond>().await?;
                endpoint.recv::<f::CloseSecondRequest>().await?;
                endpoint.send::<f::ClosedSecond>(&()).await?;
                endpoint.recv::<f::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == g_messages::Handoff::LOGICAL_LABEL => {
                            branch.recv::<g_messages::Handoff>().await?;
                            endpoint.send::<g_messages::SecondReady>(&()).await?;
                            endpoint.recv::<g_messages::AcquireSecond>().await?;
                            endpoint.recv::<g_messages::ConfirmSecond>().await?;
                            endpoint.send::<g_messages::GrantedSecond>(&()).await?;
                            endpoint.recv::<g_messages::ReturnSecond>().await?;
                            endpoint.send::<g_messages::ReturnedSecond>(&()).await?;
                        }
                        l if l == g_messages::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<g_messages::CloseHandoff>().await?;
                            endpoint.send::<g_messages::SecondReady>(&()).await?;
                            endpoint.recv::<g_messages::AcquireSecond>().await?;
                            endpoint.recv::<g_messages::CloseSecondRequest>().await?;
                            endpoint.send::<g_messages::ClosedSecond>(&()).await?;
                            endpoint.recv::<g_messages::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            l if l == g_messages::Handoff::LOGICAL_LABEL => {
                branch.recv::<g_messages::Handoff>().await?;
                endpoint.send::<g_messages::SecondReady>(&()).await?;
                endpoint.recv::<g_messages::AcquireSecond>().await?;
                endpoint.recv::<g_messages::ConfirmSecond>().await?;
                endpoint.send::<g_messages::GrantedSecond>(&()).await?;
                endpoint.recv::<g_messages::ReturnSecond>().await?;
                endpoint.send::<g_messages::ReturnedSecond>(&()).await?;
            }
            l if l == g_messages::CloseHandoff::LOGICAL_LABEL => {
                branch.recv::<g_messages::CloseHandoff>().await?;
                endpoint.send::<g_messages::SecondReady>(&()).await?;
                endpoint.recv::<g_messages::AcquireSecond>().await?;
                endpoint.recv::<g_messages::CloseSecondRequest>().await?;
                endpoint.send::<g_messages::ClosedSecond>(&()).await?;
                endpoint.recv::<g_messages::DoneSecond>().await?;
                loop {
                    let branch = endpoint.offer().await?;
                    match branch.label() {
                        l if l == f::Handoff::LOGICAL_LABEL => {
                            branch.recv::<f::Handoff>().await?;
                            endpoint.send::<f::SecondReady>(&()).await?;
                            endpoint.recv::<f::AcquireSecond>().await?;
                            endpoint.recv::<f::ConfirmSecond>().await?;
                            endpoint.send::<f::GrantedSecond>(&()).await?;
                            endpoint.recv::<f::ReturnSecond>().await?;
                            endpoint.send::<f::ReturnedSecond>(&()).await?;
                        }
                        l if l == f::CloseHandoff::LOGICAL_LABEL => {
                            branch.recv::<f::CloseHandoff>().await?;
                            endpoint.send::<f::SecondReady>(&()).await?;
                            endpoint.recv::<f::AcquireSecond>().await?;
                            endpoint.recv::<f::CloseSecondRequest>().await?;
                            endpoint.send::<f::ClosedSecond>(&()).await?;
                            endpoint.recv::<f::DoneSecond>().await?;
                            return Ok(());
                        }
                        _ => {
                            return Err(RunError::Domain(
                                "unexpected physical module request".into(),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(RunError::Domain(
                    "unexpected physical module request".into(),
                ));
            }
        }
    }
}
