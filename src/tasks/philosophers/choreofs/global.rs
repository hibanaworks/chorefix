//! Guest requests, thinking, two-fork ownership and eating share one global.
use super::super::global::{ForkA, ForkB, ForkC, ForkD, ForkE, ForkF, ForkG, ForkMessages};
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;
pub const SERVICE_FD: u8 = 4;
pub const SERVICE_PREFIX: &[u8] = b"service/";
pub const RESULT_PATH: &[u8] = b"service/result";
pub const CHECK_A: u16 = 480;
pub const CHECK_B: u16 = 481;
pub const CHECK_C: u16 = 482;
pub const CHECK_D: u16 = 483;
pub const CHECK_E: u16 = 484;
pub const CHECK_F: u16 = 485;
pub const CHECK_G: u16 = 486;
pub(crate) trait RequestMessages {
    type Open: g::Message<Payload = protocol::PathOpenReq>;
    type Opened: g::Message<Payload = protocol::PathOpenedRet>;
    type Write: g::Message<Payload = protocol::FdWriteReq>;
    type Written: g::Message<Payload = protocol::FdWriteDoneRet>;
    type Commit: g::Message<Payload = protocol::FdCloseReq>;
    type Admitted: g::Message<Payload = ()>;
    type Rejected: g::Message<Payload = &'static [u8]>;
    type RequestReady: g::Message<Payload = ()>;
    type Validate: g::Message<Payload = ()>;
}
pub(crate) struct Think;
pub type ThinkOpen = g::Msg<100, protocol::PathOpenReq>;
pub type ThinkOpened = g::Msg<101, protocol::PathOpenedRet>;
pub type ThinkWrite = g::Msg<102, protocol::FdWriteReq>;
pub type ThinkWritten = g::Msg<103, protocol::FdWriteDoneRet>;
pub type ThinkCommit = g::Msg<104, protocol::FdCloseReq>;
pub type ThinkAdmitted = g::Msg<105, ()>;
pub type ThinkRejected<'a> = g::Msg<106, &'a [u8]>;
pub type ThinkCompleted<'a> = g::Msg<107, &'a [u8]>;
pub type ThinkServiceDone = g::Msg<108, ()>;
pub type ThinkRequestReady = g::Msg<110, ()>;
pub type ThinkValidate = g::Msg<111, ()>;
impl RequestMessages for Think {
    type Open = ThinkOpen;
    type Opened = ThinkOpened;
    type Write = ThinkWrite;
    type Written = ThinkWritten;
    type Commit = ThinkCommit;
    type Admitted = ThinkAdmitted;
    type Rejected = ThinkRejected<'static>;
    type RequestReady = ThinkRequestReady;
    type Validate = ThinkValidate;
}
pub(crate) struct Eat;
pub type EatOpen = g::Msg<150, protocol::PathOpenReq>;
pub type EatOpened = g::Msg<151, protocol::PathOpenedRet>;
pub type EatWrite = g::Msg<152, protocol::FdWriteReq>;
pub type EatWritten = g::Msg<153, protocol::FdWriteDoneRet>;
pub type EatCommit = g::Msg<154, protocol::FdCloseReq>;
pub type EatAdmitted = g::Msg<155, ()>;
pub type EatRejected<'a> = g::Msg<156, &'a [u8]>;
pub type EatCompleted<'a> = g::Msg<157, &'a [u8]>;
pub type EatServiceDone = g::Msg<158, ()>;
pub type EatRequestReady = g::Msg<160, ()>;
pub type EatValidate = g::Msg<161, ()>;
impl RequestMessages for Eat {
    type Open = EatOpen;
    type Opened = EatOpened;
    type Write = EatWrite;
    type Written = EatWritten;
    type Commit = EatCommit;
    type Admitted = EatAdmitted;
    type Rejected = EatRejected<'static>;
    type RequestReady = EatRequestReady;
    type Validate = EatValidate;
}
pub type ClosedOpen = g::Msg<200, protocol::PathOpenReq>;
pub type ClosedOpened = g::Msg<201, protocol::PathOpenedRet>;
pub type ClosedWrite = g::Msg<202, protocol::FdWriteReq>;
pub type ClosedWritten = g::Msg<203, protocol::FdWriteDoneRet>;
pub type ClosedCommit = g::Msg<204, protocol::FdCloseReq>;
pub type ClosedUnavailable = g::Msg<209, protocol::FdClosedRet>;
pub type ClosedRequestReady = g::Msg<210, ()>;
pub type ThinkRequest = g::Msg<230, ()>;
pub type EatRequest = g::Msg<232, ()>;
pub type RejectRequest = g::Msg<234, ()>;
pub type ThinkEffect = g::Msg<235, ()>;
pub type EatEffect = g::Msg<237, ()>;
pub type ServiceExit = g::Msg<241, i32>;

fn requests<
    const VM: u8,
    const OWNER: u8,
    const CHECK: u16,
    START: g::Message<Payload = ()>,
    P: RequestMessages,
>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, OWNER, START>(),
        g::seq(
            g::send::<OWNER, VM, P::RequestReady>(),
            g::route(
                g::seq(
                    g::send::<VM, OWNER, P::Open>(),
                    g::send::<OWNER, VM, P::Opened>(),
                ),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, P::Write>(),
                        g::send::<OWNER, VM, P::Written>(),
                    ),
                    g::seq(
                        g::send::<VM, OWNER, P::Commit>(),
                        g::route(
                            g::seq(
                                g::send::<OWNER, OWNER, P::Validate>(),
                                g::send::<OWNER, VM, P::Admitted>(),
                            ),
                            g::send::<OWNER, VM, P::Rejected>(),
                        )
                        .resolve::<CHECK>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn closed<const VM: u8, const OWNER: u8>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, OWNER, RejectRequest>(),
        g::seq(
            g::send::<OWNER, VM, ClosedRequestReady>(),
            g::route(
                g::seq(
                    g::send::<VM, OWNER, ClosedOpen>(),
                    g::send::<OWNER, VM, ClosedOpened>(),
                ),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, ClosedWrite>(),
                        g::send::<OWNER, VM, ClosedWritten>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<VM, OWNER, ClosedCommit>(),
                            g::send::<OWNER, VM, ClosedUnavailable>(),
                        ),
                        g::send::<VM, OWNER, ServiceExit>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}

fn think<const VM: u8, const OWNER: u8, const CHECK: u16>() -> impl Projectable {
    g::seq(
        requests::<VM, OWNER, CHECK, ThinkRequest, Think>(),
        g::seq(
            g::send::<OWNER, OWNER, ThinkEffect>(),
            g::seq(
                g::send::<OWNER, VM, ThinkCompleted<'static>>(),
                g::send::<OWNER, VM, ThinkServiceDone>(),
            ),
        ),
    )
}
fn dine<
    const VM: u8,
    const OWNER: u8,
    const LEFT: u8,
    const RIGHT: u8,
    const CHECK: u16,
    L: ForkMessages,
    R: ForkMessages,
>() -> impl Projectable {
    let preparation = g::seq(
        think::<VM, OWNER, CHECK>(),
        requests::<VM, OWNER, CHECK, EatRequest, Eat>(),
    );
    let availability = g::par(
        g::send::<LEFT, OWNER, L::Available>(),
        g::send::<RIGHT, OWNER, R::Available>(),
    );
    let acquire = g::par(
        g::seq(
            g::send::<OWNER, LEFT, L::Acquire>(),
            g::send::<LEFT, OWNER, L::Granted>(),
        ),
        g::seq(
            g::send::<OWNER, RIGHT, R::Acquire>(),
            g::send::<RIGHT, OWNER, R::Granted>(),
        ),
    );
    let release = g::par(
        g::seq(
            g::send::<OWNER, LEFT, L::Return>(),
            g::send::<LEFT, OWNER, L::Returned>(),
        ),
        g::seq(
            g::send::<OWNER, RIGHT, R::Return>(),
            g::send::<RIGHT, OWNER, R::Returned>(),
        ),
    );
    let eat = g::seq(
        g::par(preparation, availability),
        g::seq(
            acquire,
            g::seq(
                g::send::<OWNER, OWNER, EatEffect>(),
                g::seq(
                    release,
                    g::seq(
                        g::send::<OWNER, VM, EatCompleted<'static>>(),
                        g::send::<OWNER, VM, EatServiceDone>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(eat, closed::<VM, OWNER>())
}

pub mod roles9e {
    pub const A: u8 = 0;
    pub const FORK_A: u8 = 7;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const FORK_B: u8 = 8;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const FORK_C: u8 = 9;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
}
pub struct Programs9E {
    pub a: RoleProgram<{ roles9e::A }>,
    pub fork_a: RoleProgram<{ roles9e::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles9e::A_WASI }>,
    pub a_io: RoleProgram<{ roles9e::A_IO }>,
    pub b: RoleProgram<{ roles9e::B }>,
    pub fork_b: RoleProgram<{ roles9e::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles9e::B_WASI }>,
    pub b_io: RoleProgram<{ roles9e::B_IO }>,
    pub c: RoleProgram<{ roles9e::C }>,
    pub fork_c: RoleProgram<{ roles9e::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles9e::C_WASI }>,
    pub c_io: RoleProgram<{ roles9e::C_IO }>,
}
pub fn project_9e() -> Programs9E {
    use roles9e::*;
    let task = g::seq(
        dine::<A_WASI, A, FORK_A, FORK_C, CHECK_A, ForkA, ForkC>(),
        g::seq(
            dine::<B_WASI, B, FORK_B, FORK_A, CHECK_B, ForkB, ForkA>(),
            dine::<C_WASI, C, FORK_C, FORK_B, CHECK_C, ForkC, ForkB>(),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<A_WASI, A_IO>(),
        g::par(
            crate::shell::global::private_io::<B_WASI, B_IO>(),
            crate::shell::global::private_io::<C_WASI, C_IO>(),
        ),
    );
    let protocol = g::par(task, private);
    Programs9E {
        a: project(&protocol),
        fork_a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        fork_b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        fork_c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
    }
}

pub mod roles9m {
    pub const A: u8 = 0;
    pub const FORK_A: u8 = 7;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const FORK_B: u8 = 8;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const FORK_C: u8 = 9;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const D: u8 = 3;
    pub const FORK_D: u8 = 10;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const E: u8 = 4;
    pub const FORK_E: u8 = 11;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
}
pub struct Programs9M {
    pub a: RoleProgram<{ roles9m::A }>,
    pub fork_a: RoleProgram<{ roles9m::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles9m::A_WASI }>,
    pub a_io: RoleProgram<{ roles9m::A_IO }>,
    pub b: RoleProgram<{ roles9m::B }>,
    pub fork_b: RoleProgram<{ roles9m::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles9m::B_WASI }>,
    pub b_io: RoleProgram<{ roles9m::B_IO }>,
    pub c: RoleProgram<{ roles9m::C }>,
    pub fork_c: RoleProgram<{ roles9m::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles9m::C_WASI }>,
    pub c_io: RoleProgram<{ roles9m::C_IO }>,
    pub d: RoleProgram<{ roles9m::D }>,
    pub fork_d: RoleProgram<{ roles9m::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles9m::D_WASI }>,
    pub d_io: RoleProgram<{ roles9m::D_IO }>,
    pub e: RoleProgram<{ roles9m::E }>,
    pub fork_e: RoleProgram<{ roles9m::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles9m::E_WASI }>,
    pub e_io: RoleProgram<{ roles9m::E_IO }>,
}
pub fn project_9m() -> Programs9M {
    use roles9m::*;
    let task = g::seq(
        g::par(
            dine::<A_WASI, A, FORK_A, FORK_E, CHECK_A, ForkA, ForkE>(),
            dine::<C_WASI, C, FORK_C, FORK_B, CHECK_C, ForkC, ForkB>(),
        ),
        g::seq(
            g::par(
                dine::<B_WASI, B, FORK_B, FORK_A, CHECK_B, ForkB, ForkA>(),
                dine::<D_WASI, D, FORK_D, FORK_C, CHECK_D, ForkD, ForkC>(),
            ),
            dine::<E_WASI, E, FORK_E, FORK_D, CHECK_E, ForkE, ForkD>(),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<A_WASI, A_IO>(),
        g::par(
            crate::shell::global::private_io::<B_WASI, B_IO>(),
            g::par(
                crate::shell::global::private_io::<C_WASI, C_IO>(),
                g::par(
                    crate::shell::global::private_io::<D_WASI, D_IO>(),
                    crate::shell::global::private_io::<E_WASI, E_IO>(),
                ),
            ),
        ),
    );
    let protocol = g::par(task, private);
    Programs9M {
        a: project(&protocol),
        fork_a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        fork_b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        fork_c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        d: project(&protocol),
        fork_d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        e: project(&protocol),
        fork_e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
    }
}

pub mod roles9h {
    pub const A: u8 = 0;
    pub const FORK_A: u8 = 7;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const FORK_B: u8 = 8;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const FORK_C: u8 = 9;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const D: u8 = 3;
    pub const FORK_D: u8 = 10;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const E: u8 = 4;
    pub const FORK_E: u8 = 11;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
    pub const F: u8 = 5;
    pub const FORK_F: u8 = 12;
    pub const F_WASI: u8 = 19;
    pub const F_IO: u8 = 26;
    pub const G: u8 = 6;
    pub const FORK_G: u8 = 13;
    pub const G_WASI: u8 = 20;
    pub const G_IO: u8 = 27;
}
pub struct Programs9H {
    pub a: RoleProgram<{ roles9h::A }>,
    pub fork_a: RoleProgram<{ roles9h::FORK_A }>,
    pub a_wasi: RoleProgram<{ roles9h::A_WASI }>,
    pub a_io: RoleProgram<{ roles9h::A_IO }>,
    pub b: RoleProgram<{ roles9h::B }>,
    pub fork_b: RoleProgram<{ roles9h::FORK_B }>,
    pub b_wasi: RoleProgram<{ roles9h::B_WASI }>,
    pub b_io: RoleProgram<{ roles9h::B_IO }>,
    pub c: RoleProgram<{ roles9h::C }>,
    pub fork_c: RoleProgram<{ roles9h::FORK_C }>,
    pub c_wasi: RoleProgram<{ roles9h::C_WASI }>,
    pub c_io: RoleProgram<{ roles9h::C_IO }>,
    pub d: RoleProgram<{ roles9h::D }>,
    pub fork_d: RoleProgram<{ roles9h::FORK_D }>,
    pub d_wasi: RoleProgram<{ roles9h::D_WASI }>,
    pub d_io: RoleProgram<{ roles9h::D_IO }>,
    pub e: RoleProgram<{ roles9h::E }>,
    pub fork_e: RoleProgram<{ roles9h::FORK_E }>,
    pub e_wasi: RoleProgram<{ roles9h::E_WASI }>,
    pub e_io: RoleProgram<{ roles9h::E_IO }>,
    pub f: RoleProgram<{ roles9h::F }>,
    pub fork_f: RoleProgram<{ roles9h::FORK_F }>,
    pub f_wasi: RoleProgram<{ roles9h::F_WASI }>,
    pub f_io: RoleProgram<{ roles9h::F_IO }>,
    pub g: RoleProgram<{ roles9h::G }>,
    pub fork_g: RoleProgram<{ roles9h::FORK_G }>,
    pub g_wasi: RoleProgram<{ roles9h::G_WASI }>,
    pub g_io: RoleProgram<{ roles9h::G_IO }>,
}
pub fn project_9h() -> Programs9H {
    use roles9h::*;
    let task = g::seq(
        g::par(
            dine::<A_WASI, A, FORK_A, FORK_G, CHECK_A, ForkA, ForkG>(),
            g::par(
                dine::<C_WASI, C, FORK_C, FORK_B, CHECK_C, ForkC, ForkB>(),
                dine::<E_WASI, E, FORK_E, FORK_D, CHECK_E, ForkE, ForkD>(),
            ),
        ),
        g::seq(
            g::par(
                dine::<B_WASI, B, FORK_B, FORK_A, CHECK_B, ForkB, ForkA>(),
                g::par(
                    dine::<D_WASI, D, FORK_D, FORK_C, CHECK_D, ForkD, ForkC>(),
                    dine::<F_WASI, F, FORK_F, FORK_E, CHECK_F, ForkF, ForkE>(),
                ),
            ),
            dine::<G_WASI, G, FORK_G, FORK_F, CHECK_G, ForkG, ForkF>(),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<A_WASI, A_IO>(),
        g::par(
            crate::shell::global::private_io::<B_WASI, B_IO>(),
            g::par(
                crate::shell::global::private_io::<C_WASI, C_IO>(),
                g::par(
                    crate::shell::global::private_io::<D_WASI, D_IO>(),
                    g::par(
                        crate::shell::global::private_io::<E_WASI, E_IO>(),
                        g::par(
                            crate::shell::global::private_io::<F_WASI, F_IO>(),
                            crate::shell::global::private_io::<G_WASI, G_IO>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let protocol = g::par(task, private);
    Programs9H {
        a: project(&protocol),
        fork_a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        fork_b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        fork_c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        d: project(&protocol),
        fork_d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        e: project(&protocol),
        fork_e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
        f: project(&protocol),
        fork_f: project(&protocol),
        f_wasi: project(&protocol),
        f_io: project(&protocol),
        g: project(&protocol),
        fork_g: project(&protocol),
        g_wasi: project(&protocol),
        g_io: project(&protocol),
    }
}
