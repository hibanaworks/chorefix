//! One resource permits one active projected loan at a time.
use hibana::{
    g,
    runtime::program::{RoleProgram, project},
};

pub type AcquireFirst = g::Msg<10, ()>;
pub type GrantedFirst = g::Msg<11, ()>;
pub type ReturnFirst = g::Msg<12, ()>;
pub type ReturnedFirst = g::Msg<13, ()>;
pub type AcquireSecond = g::Msg<20, ()>;
pub type GrantedSecond = g::Msg<21, ()>;
pub type ReturnSecond = g::Msg<22, ()>;
pub type ReturnedSecond = g::Msg<23, ()>;
pub type ClosedFirst = g::Msg<30, ()>;
pub type ClosedSecond = g::Msg<31, ()>;

pub type Request = g::Msg<1, ()>;
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
