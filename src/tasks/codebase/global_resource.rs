//! A module has one active loan. The borrower is message data; every loan uses
//! the same projected request/grant/return/acknowledgement sequence.
use super::imp::Borrower;
use hibana::{
    g,
    runtime::program::{RoleProgram, project},
};

pub type Request = g::Msg<1, Borrower>;
pub type Granted = g::Msg<2, ()>;
pub type Return = g::Msg<3, ()>;
pub type Returned = g::Msg<4, ()>;
pub type Closed = g::Msg<5, ()>;

pub struct Programs {
    pub broker: RoleProgram<0>,
    pub resource: RoleProgram<1>,
}

pub fn project_resource() -> Programs {
    let loan = g::seq(
        g::send::<0, 1, Request>(),
        g::seq(
            g::send::<1, 0, Granted>(),
            g::seq(g::send::<0, 1, Return>(), g::send::<1, 0, Returned>()),
        ),
    );
    let protocol = g::route(loan, g::send::<0, 1, Closed>()).roll();
    Programs {
        broker: project(&protocol),
        resource: project(&protocol),
    }
}
