//! Real WASI requests, physical modules and effects share one global.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;
pub const SERVICE_FD: u8 = 4;
pub const SERVICE_PREFIX: &[u8] = b"service/";
pub const RESULT_PATH: &[u8] = b"service/result";
pub const CHECK_A: u16 = 580;
pub const CHECK_B: u16 = 581;
pub const CHECK_C: u16 = 582;
pub const CHECK_D: u16 = 583;
pub const CHECK_E: u16 = 584;
pub const CHECK_F: u16 = 585;
pub const CHECK_G: u16 = 586;
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
pub(crate) struct Design;
pub type DesignOpen = g::Msg<100, protocol::PathOpenReq>;
pub type DesignOpened = g::Msg<101, protocol::PathOpenedRet>;
pub type DesignWrite = g::Msg<102, protocol::FdWriteReq>;
pub type DesignWritten = g::Msg<103, protocol::FdWriteDoneRet>;
pub type DesignCommit = g::Msg<104, protocol::FdCloseReq>;
pub type DesignAdmitted = g::Msg<105, ()>;
pub type DesignRejected<'a> = g::Msg<106, &'a [u8]>;
pub type DesignCompleted<'a> = g::Msg<107, &'a [u8]>;
pub type DesignServiceDone = g::Msg<108, ()>;
pub type DesignRequestReady = g::Msg<110, ()>;
pub type DesignValidate = g::Msg<111, ()>;
impl RequestMessages for Design {
    type Open = DesignOpen;
    type Opened = DesignOpened;
    type Write = DesignWrite;
    type Written = DesignWritten;
    type Commit = DesignCommit;
    type Admitted = DesignAdmitted;
    type Rejected = DesignRejected<'static>;
    type RequestReady = DesignRequestReady;
    type Validate = DesignValidate;
}
pub(crate) struct Implement;
pub type ImplementOpen = g::Msg<125, protocol::PathOpenReq>;
pub type ImplementOpened = g::Msg<126, protocol::PathOpenedRet>;
pub type ImplementWrite = g::Msg<127, protocol::FdWriteReq>;
pub type ImplementWritten = g::Msg<128, protocol::FdWriteDoneRet>;
pub type ImplementCommit = g::Msg<129, protocol::FdCloseReq>;
pub type ImplementAdmitted = g::Msg<130, ()>;
pub type ImplementRejected<'a> = g::Msg<131, &'a [u8]>;
pub type ImplementCompleted<'a> = g::Msg<132, &'a [u8]>;
pub type ImplementServiceDone = g::Msg<133, ()>;
pub type ImplementRequestReady = g::Msg<135, ()>;
pub type ImplementValidate = g::Msg<136, ()>;
impl RequestMessages for Implement {
    type Open = ImplementOpen;
    type Opened = ImplementOpened;
    type Write = ImplementWrite;
    type Written = ImplementWritten;
    type Commit = ImplementCommit;
    type Admitted = ImplementAdmitted;
    type Rejected = ImplementRejected<'static>;
    type RequestReady = ImplementRequestReady;
    type Validate = ImplementValidate;
}
pub(crate) struct Commit;
pub type CommitOpen = g::Msg<150, protocol::PathOpenReq>;
pub type CommitOpened = g::Msg<151, protocol::PathOpenedRet>;
pub type CommitWrite = g::Msg<152, protocol::FdWriteReq>;
pub type CommitWritten = g::Msg<153, protocol::FdWriteDoneRet>;
pub type CommitCommit = g::Msg<154, protocol::FdCloseReq>;
pub type CommitAdmitted = g::Msg<155, ()>;
pub type CommitRejected<'a> = g::Msg<156, &'a [u8]>;
pub type CommitCompleted<'a> = g::Msg<157, &'a [u8]>;
pub type CommitServiceDone = g::Msg<158, ()>;
pub type CommitRequestReady = g::Msg<160, ()>;
pub type CommitValidate = g::Msg<161, ()>;
impl RequestMessages for Commit {
    type Open = CommitOpen;
    type Opened = CommitOpened;
    type Write = CommitWrite;
    type Written = CommitWritten;
    type Commit = CommitCommit;
    type Admitted = CommitAdmitted;
    type Rejected = CommitRejected<'static>;
    type RequestReady = CommitRequestReady;
    type Validate = CommitValidate;
}
pub(crate) struct Test;
pub type TestOpen = g::Msg<175, protocol::PathOpenReq>;
pub type TestOpened = g::Msg<176, protocol::PathOpenedRet>;
pub type TestWrite = g::Msg<177, protocol::FdWriteReq>;
pub type TestWritten = g::Msg<178, protocol::FdWriteDoneRet>;
pub type TestCommit = g::Msg<179, protocol::FdCloseReq>;
pub type TestAdmitted = g::Msg<180, ()>;
pub type TestRejected<'a> = g::Msg<181, &'a [u8]>;
pub type TestCompleted<'a> = g::Msg<182, &'a [u8]>;
pub type TestServiceDone = g::Msg<183, ()>;
pub type TestRequestReady = g::Msg<185, ()>;
pub type TestValidate = g::Msg<186, ()>;
impl RequestMessages for Test {
    type Open = TestOpen;
    type Opened = TestOpened;
    type Write = TestWrite;
    type Written = TestWritten;
    type Commit = TestCommit;
    type Admitted = TestAdmitted;
    type Rejected = TestRejected<'static>;
    type RequestReady = TestRequestReady;
    type Validate = TestValidate;
}
pub type ClosedOpen = g::Msg<200, protocol::PathOpenReq>;
pub type ClosedOpened = g::Msg<201, protocol::PathOpenedRet>;
pub type ClosedWrite = g::Msg<202, protocol::FdWriteReq>;
pub type ClosedWritten = g::Msg<203, protocol::FdWriteDoneRet>;
pub type ClosedCommit = g::Msg<204, protocol::FdCloseReq>;
pub type ClosedUnavailable = g::Msg<209, protocol::FdClosedRet>;
pub type ClosedRequestReady = g::Msg<210, ()>;
pub type DesignRequest = g::Msg<230, ()>;
pub type ImplementRequest = g::Msg<231, ()>;
pub type CommitRequest = g::Msg<232, ()>;
pub type TestRequest = g::Msg<233, ()>;
pub type RejectRequest = g::Msg<234, ()>;
pub type DesignEffect = g::Msg<235, ()>;
pub type ImplementEffect = g::Msg<236, ()>;
pub type CommitEffect = g::Msg<237, ()>;
pub type TestEffect = g::Msg<238, ()>;
pub type ServiceExit = g::Msg<241, i32>;

// START makes the four distinct request lifetimes explicit at their owner.
// This fragment validates bytes, never executes the task or chooses its phase.
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
fn prepare<const VM: u8, const OWNER: u8, const CHECK: u16>() -> impl Projectable {
    let design = g::seq(
        requests::<VM, OWNER, CHECK, DesignRequest, Design>(),
        g::seq(
            g::send::<OWNER, OWNER, DesignEffect>(),
            g::seq(
                g::send::<OWNER, VM, DesignCompleted<'static>>(),
                g::send::<OWNER, VM, DesignServiceDone>(),
            ),
        ),
    );
    let implementation = g::seq(
        requests::<VM, OWNER, CHECK, ImplementRequest, Implement>(),
        g::seq(
            g::send::<OWNER, OWNER, ImplementEffect>(),
            g::seq(
                g::send::<OWNER, VM, ImplementCompleted<'static>>(),
                g::send::<OWNER, VM, ImplementServiceDone>(),
            ),
        ),
    );
    g::seq(design, implementation)
}
fn commit<
    const VM: u8,
    const OWNER: u8,
    const FIRST: u8,
    const SECOND: u8,
    const CHECK: u16,
    F: ModuleMessages,
    S: ModuleMessages,
>() -> impl Projectable {
    let preparation = g::seq(
        prepare::<VM, OWNER, CHECK>(),
        requests::<VM, OWNER, CHECK, CommitRequest, Commit>(),
    );
    let availability = g::par(
        g::send::<FIRST, OWNER, F::Available>(),
        g::send::<SECOND, OWNER, S::Available>(),
    );
    let acquire = g::par(
        g::seq(
            g::send::<OWNER, FIRST, F::Acquire>(),
            g::send::<FIRST, OWNER, F::Granted>(),
        ),
        g::seq(
            g::send::<OWNER, SECOND, S::Acquire>(),
            g::send::<SECOND, OWNER, S::Granted>(),
        ),
    );
    let release = g::par(
        g::seq(
            g::send::<OWNER, FIRST, F::Return>(),
            g::send::<FIRST, OWNER, F::Returned>(),
        ),
        g::seq(
            g::send::<OWNER, SECOND, S::Return>(),
            g::send::<SECOND, OWNER, S::Returned>(),
        ),
    );
    let commit = g::seq(
        g::par(preparation, availability),
        g::seq(
            acquire,
            g::seq(
                g::send::<OWNER, OWNER, CommitEffect>(),
                g::seq(
                    release,
                    g::seq(
                        g::send::<OWNER, VM, CommitCompleted<'static>>(),
                        g::send::<OWNER, VM, CommitServiceDone>(),
                    ),
                ),
            ),
        ),
    );
    let test = g::seq(
        requests::<VM, OWNER, CHECK, TestRequest, Test>(),
        g::seq(
            g::send::<OWNER, OWNER, TestEffect>(),
            g::seq(
                g::send::<OWNER, VM, TestCompleted<'static>>(),
                g::send::<OWNER, VM, TestServiceDone>(),
            ),
        ),
    );
    g::seq(commit, g::seq(test, closed::<VM, OWNER>()))
}

pub(crate) trait ModuleMessages {
    type Available: g::Message<Payload = ()>;
    type Acquire: g::Message<Payload = ()>;
    type Granted: g::Message<Payload = ()>;
    type Return: g::Message<Payload = ()>;
    type Returned: g::Message<Payload = ()>;
}
pub(crate) struct Auth;
impl ModuleMessages for Auth {
    type Available = g::Msg<80, ()>;
    type Acquire = g::Msg<10, ()>;
    type Granted = g::Msg<11, ()>;
    type Return = g::Msg<12, ()>;
    type Returned = g::Msg<13, ()>;
}
pub(crate) struct Database;
impl ModuleMessages for Database {
    type Available = g::Msg<81, ()>;
    type Acquire = g::Msg<14, ()>;
    type Granted = g::Msg<15, ()>;
    type Return = g::Msg<16, ()>;
    type Returned = g::Msg<17, ()>;
}
pub(crate) struct Api;
impl ModuleMessages for Api {
    type Available = g::Msg<82, ()>;
    type Acquire = g::Msg<18, ()>;
    type Granted = g::Msg<19, ()>;
    type Return = g::Msg<20, ()>;
    type Returned = g::Msg<21, ()>;
}
pub(crate) struct Payment;
impl ModuleMessages for Payment {
    type Available = g::Msg<83, ()>;
    type Acquire = g::Msg<22, ()>;
    type Granted = g::Msg<23, ()>;
    type Return = g::Msg<24, ()>;
    type Returned = g::Msg<25, ()>;
}
pub(crate) struct Notify;
impl ModuleMessages for Notify {
    type Available = g::Msg<84, ()>;
    type Acquire = g::Msg<26, ()>;
    type Granted = g::Msg<27, ()>;
    type Return = g::Msg<28, ()>;
    type Returned = g::Msg<29, ()>;
}
pub mod roles1e {
    pub const A: u8 = 0;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const AUTH: u8 = 7;
    pub const DATABASE: u8 = 8;
    pub const API: u8 = 9;
}
pub struct Programs1E {
    pub a: RoleProgram<{ roles1e::A }>,
    pub a_wasi: RoleProgram<{ roles1e::A_WASI }>,
    pub a_io: RoleProgram<{ roles1e::A_IO }>,
    pub b: RoleProgram<{ roles1e::B }>,
    pub b_wasi: RoleProgram<{ roles1e::B_WASI }>,
    pub b_io: RoleProgram<{ roles1e::B_IO }>,
    pub c: RoleProgram<{ roles1e::C }>,
    pub c_wasi: RoleProgram<{ roles1e::C_WASI }>,
    pub c_io: RoleProgram<{ roles1e::C_IO }>,
    pub auth: RoleProgram<{ roles1e::AUTH }>,
    pub database: RoleProgram<{ roles1e::DATABASE }>,
    pub api: RoleProgram<{ roles1e::API }>,
}
pub fn project_1e() -> Programs1E {
    use roles1e::*;
    let dev_0 = commit::<A_WASI, A, AUTH, DATABASE, CHECK_A, Auth, Database>();
    let dev_1 = commit::<B_WASI, B, API, DATABASE, CHECK_B, Api, Database>();
    let dev_2 = commit::<C_WASI, C, API, AUTH, CHECK_C, Api, Auth>();
    let task = g::seq(dev_0, g::seq(dev_1, dev_2));
    let protocol = g::par(
        task,
        g::par(
            crate::shell::global::private_io::<A_WASI, A_IO>(),
            g::par(
                crate::shell::global::private_io::<B_WASI, B_IO>(),
                crate::shell::global::private_io::<C_WASI, C_IO>(),
            ),
        ),
    );
    Programs1E {
        a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        auth: project(&protocol),
        database: project(&protocol),
        api: project(&protocol),
    }
}
pub mod roles1m {
    pub const A: u8 = 0;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B: u8 = 1;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C: u8 = 2;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const D: u8 = 3;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const E: u8 = 4;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
    pub const AUTH: u8 = 7;
    pub const DATABASE: u8 = 8;
    pub const API: u8 = 9;
    pub const PAYMENT: u8 = 10;
    pub const NOTIFY: u8 = 11;
}
pub struct Programs1M {
    pub a: RoleProgram<{ roles1m::A }>,
    pub a_wasi: RoleProgram<{ roles1m::A_WASI }>,
    pub a_io: RoleProgram<{ roles1m::A_IO }>,
    pub b: RoleProgram<{ roles1m::B }>,
    pub b_wasi: RoleProgram<{ roles1m::B_WASI }>,
    pub b_io: RoleProgram<{ roles1m::B_IO }>,
    pub c: RoleProgram<{ roles1m::C }>,
    pub c_wasi: RoleProgram<{ roles1m::C_WASI }>,
    pub c_io: RoleProgram<{ roles1m::C_IO }>,
    pub d: RoleProgram<{ roles1m::D }>,
    pub d_wasi: RoleProgram<{ roles1m::D_WASI }>,
    pub d_io: RoleProgram<{ roles1m::D_IO }>,
    pub e: RoleProgram<{ roles1m::E }>,
    pub e_wasi: RoleProgram<{ roles1m::E_WASI }>,
    pub e_io: RoleProgram<{ roles1m::E_IO }>,
    pub auth: RoleProgram<{ roles1m::AUTH }>,
    pub database: RoleProgram<{ roles1m::DATABASE }>,
    pub api: RoleProgram<{ roles1m::API }>,
    pub payment: RoleProgram<{ roles1m::PAYMENT }>,
    pub notify: RoleProgram<{ roles1m::NOTIFY }>,
}
pub fn project_1m() -> Programs1M {
    use roles1m::*;
    let dev_0 = commit::<A_WASI, A, AUTH, DATABASE, CHECK_A, Auth, Database>();
    let dev_1 = commit::<B_WASI, B, API, DATABASE, CHECK_B, Api, Database>();
    let dev_2 = commit::<C_WASI, C, API, PAYMENT, CHECK_C, Api, Payment>();
    let dev_3 = commit::<D_WASI, D, NOTIFY, PAYMENT, CHECK_D, Notify, Payment>();
    let dev_4 = commit::<E_WASI, E, AUTH, NOTIFY, CHECK_E, Auth, Notify>();
    let task = g::seq(g::par(dev_0, dev_2), g::seq(g::par(dev_1, dev_3), dev_4));
    let protocol = g::par(
        task,
        g::par(
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
        ),
    );
    Programs1M {
        a: project(&protocol),
        a_wasi: project(&protocol),
        a_io: project(&protocol),
        b: project(&protocol),
        b_wasi: project(&protocol),
        b_io: project(&protocol),
        c: project(&protocol),
        c_wasi: project(&protocol),
        c_io: project(&protocol),
        d: project(&protocol),
        d_wasi: project(&protocol),
        d_io: project(&protocol),
        e: project(&protocol),
        e_wasi: project(&protocol),
        e_io: project(&protocol),
        auth: project(&protocol),
        database: project(&protocol),
        api: project(&protocol),
        payment: project(&protocol),
        notify: project(&protocol),
    }
}
