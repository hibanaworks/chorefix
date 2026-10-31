//! Actual WASI requests, task effects and physical sections share one global.
//! Service loops accept ordinary I/O errors without a second task state machine.
use super::imp::WriteArguments;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;

pub const SERVICE_FD: u8 = 4;
pub const SERVICE_PREFIX: &[u8] = b"service/";
pub const RESULT_PATH: &[u8] = b"service/result";
pub const CHECK_A: u16 = 280;
pub const CHECK_B: u16 = 281;
pub const CHECK_C: u16 = 282;
pub const CHECK_D: u16 = 283;
pub const CHECK_E: u16 = 284;
pub const CHECK_F: u16 = 285;
pub const CHECK_G: u16 = 286;
pub(crate) trait RequestMessages {
    type Open: g::Message<Payload = protocol::PathOpenReq>;
    type Opened: g::Message<Payload = protocol::PathOpenedRet>;
    type Write: g::Message<Payload = protocol::FdWriteObjectReq>;
    type Written: g::Message<Payload = protocol::FdWriteDoneRet>;
    type Commit: g::Message<Payload = protocol::FdCloseReq>;
    type Admitted: g::Message;
    type Rejected: g::Message<Payload = &'static [u8]>;
    type RequestReady: g::Message<Payload = ()>;
    type Validate: g::Message<Payload = ()>;
}
pub(crate) struct Read;
pub type ReadOpen = g::Msg<100, protocol::PathOpenReq>;
pub type ReadOpened = g::Msg<101, protocol::PathOpenedRet>;
pub type ReadWrite = g::Msg<102, protocol::FdWriteObjectReq>;
pub type ReadWritten = g::Msg<103, protocol::FdWriteDoneRet>;
pub type ReadCommit = g::Msg<104, protocol::FdCloseReq>;
pub type ReadAdmitted = g::Msg<105, ()>;
pub type ReadRejected<'a> = g::Msg<106, &'a [u8]>;
pub type ReadCompleted<'a> = g::Msg<107, &'a [u8]>;
pub type ReadRequestReady = g::Msg<110, ()>;
pub type ReadValidate = g::Msg<111, ()>;
impl RequestMessages for Read {
    type Open = ReadOpen;
    type Opened = ReadOpened;
    type Write = ReadWrite;
    type Written = ReadWritten;
    type Commit = ReadCommit;
    type Admitted = ReadAdmitted;
    type Rejected = ReadRejected<'static>;
    type RequestReady = ReadRequestReady;
    type Validate = ReadValidate;
}
pub(crate) struct Draft;
pub type DraftOpen = g::Msg<125, protocol::PathOpenReq>;
pub type DraftOpened = g::Msg<126, protocol::PathOpenedRet>;
pub type DraftWrite = g::Msg<127, protocol::FdWriteObjectReq>;
pub type DraftWritten = g::Msg<128, protocol::FdWriteDoneRet>;
pub type DraftCommit = g::Msg<129, protocol::FdCloseReq>;
pub type DraftAdmitted = g::Msg<130, ()>;
pub type DraftRejected<'a> = g::Msg<131, &'a [u8]>;
pub type DraftCompleted<'a> = g::Msg<132, &'a [u8]>;
pub type DraftRequestReady = g::Msg<135, ()>;
pub type DraftValidate = g::Msg<136, ()>;
impl RequestMessages for Draft {
    type Open = DraftOpen;
    type Opened = DraftOpened;
    type Write = DraftWrite;
    type Written = DraftWritten;
    type Commit = DraftCommit;
    type Admitted = DraftAdmitted;
    type Rejected = DraftRejected<'static>;
    type RequestReady = DraftRequestReady;
    type Validate = DraftValidate;
}
pub(crate) struct Write;
pub type WriteOpen = g::Msg<150, protocol::PathOpenReq>;
pub type WriteOpened = g::Msg<151, protocol::PathOpenedRet>;
pub type WriteWrite = g::Msg<152, protocol::FdWriteObjectReq>;
pub type WriteWritten = g::Msg<153, protocol::FdWriteDoneRet>;
pub type WriteCommit = g::Msg<154, protocol::FdCloseReq>;
pub type WriteAdmitted<'a> = g::Msg<155, &'a [u8]>;
pub type WriteRejected<'a> = g::Msg<156, &'a [u8]>;
pub type WriteRequestReady = g::Msg<160, ()>;
pub type WriteValidate = g::Msg<161, ()>;
impl RequestMessages for Write {
    type Open = WriteOpen;
    type Opened = WriteOpened;
    type Write = WriteWrite;
    type Written = WriteWritten;
    type Commit = WriteCommit;
    type Admitted = WriteAdmitted<'static>;
    type Rejected = WriteRejected<'static>;
    type RequestReady = WriteRequestReady;
    type Validate = WriteValidate;
}
pub(crate) struct Review;
pub type ReviewOpen = g::Msg<175, protocol::PathOpenReq>;
pub type ReviewOpened = g::Msg<176, protocol::PathOpenedRet>;
pub type ReviewWrite = g::Msg<177, protocol::FdWriteObjectReq>;
pub type ReviewWritten = g::Msg<178, protocol::FdWriteDoneRet>;
pub type ReviewCommit = g::Msg<179, protocol::FdCloseReq>;
pub type ReviewAdmitted = g::Msg<180, ()>;
pub type ReviewRejected<'a> = g::Msg<181, &'a [u8]>;
pub type ReviewCompleted<'a> = g::Msg<182, &'a [u8]>;
pub type ReviewRequestReady = g::Msg<185, ()>;
pub type ReviewValidate = g::Msg<186, ()>;
impl RequestMessages for Review {
    type Open = ReviewOpen;
    type Opened = ReviewOpened;
    type Write = ReviewWrite;
    type Written = ReviewWritten;
    type Commit = ReviewCommit;
    type Admitted = ReviewAdmitted;
    type Rejected = ReviewRejected<'static>;
    type RequestReady = ReviewRequestReady;
    type Validate = ReviewValidate;
}
pub type ClosedOpen = g::Msg<200, protocol::PathOpenReq>;
pub type ClosedOpened = g::Msg<201, protocol::PathOpenedRet>;
pub type ClosedWrite = g::Msg<202, protocol::FdWriteObjectReq>;
pub type ClosedWritten = g::Msg<203, protocol::FdWriteDoneRet>;
pub type ClosedCommit = g::Msg<204, protocol::FdCloseReq>;
pub type ClosedUnavailable = g::Msg<209, protocol::FdClosedRet>;
pub type ClosedRequestReady = g::Msg<210, ()>;
pub type ReadRequest = g::Msg<230, ()>;
pub type DraftRequest = g::Msg<231, ()>;
pub type WriteRequest = g::Msg<232, ()>;
pub type ReviewRequest = g::Msg<233, ()>;
pub type RejectRequest = g::Msg<234, ()>;
pub type ReadEffect = g::Msg<235, ()>;
pub type DraftEffect = g::Msg<236, ()>;
pub type Loan<'r, const RESOURCE: u8> = g::Msg<10, Endpoint<'r, RESOURCE>>;
pub type SectionRequest<const RESOURCE: u8> = g::Msg<RESOURCE, WriteArguments>;
pub type SectionWritten<'a, const RESOURCE: u8> = g::Msg<RESOURCE, &'a [u8]>;
pub type ReviewEffect = g::Msg<238, ()>;
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
    let read = g::seq(
        requests::<VM, OWNER, CHECK, ReadRequest, Read>(),
        g::seq(
            g::send::<OWNER, OWNER, ReadEffect>(),
            g::send::<OWNER, VM, ReadCompleted>(),
        ),
    );
    let draft = g::seq(
        requests::<VM, OWNER, CHECK, DraftRequest, Draft>(),
        g::seq(
            g::send::<OWNER, OWNER, DraftEffect>(),
            g::send::<OWNER, VM, DraftCompleted>(),
        ),
    );
    g::seq(read, draft)
}
fn write<
    const VM: u8,
    const OWNER: u8,
    const FIRST_OWNER: u8,
    const SECOND_OWNER: u8,
    const FIRST: u8,
    const SECOND: u8,
    const CHECK: u16,
>() -> impl Projectable {
    let request = g::seq(
        prepare::<VM, OWNER, CHECK>(),
        requests::<VM, OWNER, CHECK, WriteRequest, Write>(),
    );
    let sections = g::seq(
        g::par(
            g::seq(
                g::send::<FIRST_OWNER, VM, Loan<FIRST>>(),
                g::send::<FIRST, OWNER, SectionRequest<FIRST>>(),
            ),
            g::seq(
                g::send::<SECOND_OWNER, VM, Loan<SECOND>>(),
                g::send::<SECOND, OWNER, SectionRequest<SECOND>>(),
            ),
        ),
        g::seq(
            g::par(
                g::send::<OWNER, FIRST, SectionWritten<FIRST>>(),
                g::send::<OWNER, SECOND, SectionWritten<SECOND>>(),
            ),
            g::par(
                g::send::<VM, FIRST_OWNER, Loan<FIRST>>(),
                g::send::<VM, SECOND_OWNER, Loan<SECOND>>(),
            ),
        ),
    );
    let review = g::seq(
        requests::<VM, OWNER, CHECK, ReviewRequest, Review>(),
        g::seq(
            g::send::<OWNER, OWNER, ReviewEffect>(),
            g::send::<OWNER, VM, ReviewCompleted>(),
        ),
    );
    g::seq(
        request,
        g::seq(sections, g::seq(review, closed::<VM, OWNER>())),
    )
}

pub mod roles2e {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const INTRO: u8 = 7;
    pub const INTRO_RESOURCE: u8 = 28;
    pub const METHODS: u8 = 8;
    pub const METHODS_RESOURCE: u8 = 29;
    pub const RESULTS: u8 = 9;
    pub const RESULTS_RESOURCE: u8 = 30;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
}
pub struct Programs2E {
    pub a: RoleProgram<{ roles2e::A }>,
    pub b: RoleProgram<{ roles2e::B }>,
    pub c: RoleProgram<{ roles2e::C }>,
    pub intro: RoleProgram<{ roles2e::INTRO }>,
    pub intro_resource: RoleProgram<{ roles2e::INTRO_RESOURCE }>,
    pub methods: RoleProgram<{ roles2e::METHODS }>,
    pub methods_resource: RoleProgram<{ roles2e::METHODS_RESOURCE }>,
    pub results: RoleProgram<{ roles2e::RESULTS }>,
    pub results_resource: RoleProgram<{ roles2e::RESULTS_RESOURCE }>,
    pub a_wasi: RoleProgram<{ roles2e::A_WASI }>,
    pub b_wasi: RoleProgram<{ roles2e::B_WASI }>,
    pub c_wasi: RoleProgram<{ roles2e::C_WASI }>,
    pub a_io: RoleProgram<{ roles2e::A_IO }>,
    pub b_io: RoleProgram<{ roles2e::B_IO }>,
    pub c_io: RoleProgram<{ roles2e::C_IO }>,
}
pub fn project_2e() -> Programs2E {
    use roles2e::*;
    let writes_0 = write::<A_WASI, A, INTRO, METHODS, INTRO_RESOURCE, METHODS_RESOURCE, CHECK_A>();
    let writes_1 =
        write::<B_WASI, B, METHODS, RESULTS, METHODS_RESOURCE, RESULTS_RESOURCE, CHECK_B>();
    let writes_2 = write::<C_WASI, C, INTRO, RESULTS, INTRO_RESOURCE, RESULTS_RESOURCE, CHECK_C>();
    let task = g::seq(writes_0, g::seq(writes_1, writes_2));
    let private = g::par(
        crate::shell::global::private_io::<A_WASI, A_IO>(),
        g::par(
            crate::shell::global::private_io::<B_WASI, B_IO>(),
            crate::shell::global::private_io::<C_WASI, C_IO>(),
        ),
    );
    let protocol = g::par(task, private);
    Programs2E {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        intro: project(&protocol),
        intro_resource: project(&protocol),
        methods: project(&protocol),
        methods_resource: project(&protocol),
        results: project(&protocol),
        results_resource: project(&protocol),
        a_wasi: project(&protocol),
        b_wasi: project(&protocol),
        c_wasi: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        c_io: project(&protocol),
    }
}

pub mod roles2m {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const BACKGROUND: u8 = 7;
    pub const BACKGROUND_RESOURCE: u8 = 28;
    pub const INTRO: u8 = 8;
    pub const INTRO_RESOURCE: u8 = 29;
    pub const METHODS: u8 = 9;
    pub const METHODS_RESOURCE: u8 = 30;
    pub const EXPERIMENTS: u8 = 10;
    pub const EXPERIMENTS_RESOURCE: u8 = 31;
    pub const RESULTS: u8 = 11;
    pub const RESULTS_RESOURCE: u8 = 32;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
}
pub struct Programs2M {
    pub a: RoleProgram<{ roles2m::A }>,
    pub b: RoleProgram<{ roles2m::B }>,
    pub c: RoleProgram<{ roles2m::C }>,
    pub d: RoleProgram<{ roles2m::D }>,
    pub e: RoleProgram<{ roles2m::E }>,
    pub background: RoleProgram<{ roles2m::BACKGROUND }>,
    pub background_resource: RoleProgram<{ roles2m::BACKGROUND_RESOURCE }>,
    pub intro: RoleProgram<{ roles2m::INTRO }>,
    pub intro_resource: RoleProgram<{ roles2m::INTRO_RESOURCE }>,
    pub methods: RoleProgram<{ roles2m::METHODS }>,
    pub methods_resource: RoleProgram<{ roles2m::METHODS_RESOURCE }>,
    pub experiments: RoleProgram<{ roles2m::EXPERIMENTS }>,
    pub experiments_resource: RoleProgram<{ roles2m::EXPERIMENTS_RESOURCE }>,
    pub results: RoleProgram<{ roles2m::RESULTS }>,
    pub results_resource: RoleProgram<{ roles2m::RESULTS_RESOURCE }>,
    pub a_wasi: RoleProgram<{ roles2m::A_WASI }>,
    pub b_wasi: RoleProgram<{ roles2m::B_WASI }>,
    pub c_wasi: RoleProgram<{ roles2m::C_WASI }>,
    pub d_wasi: RoleProgram<{ roles2m::D_WASI }>,
    pub e_wasi: RoleProgram<{ roles2m::E_WASI }>,
    pub a_io: RoleProgram<{ roles2m::A_IO }>,
    pub b_io: RoleProgram<{ roles2m::B_IO }>,
    pub c_io: RoleProgram<{ roles2m::C_IO }>,
    pub d_io: RoleProgram<{ roles2m::D_IO }>,
    pub e_io: RoleProgram<{ roles2m::E_IO }>,
}
pub fn project_2m() -> Programs2M {
    use roles2m::*;
    let writes_0 = g::par(
        write::<A_WASI, A, BACKGROUND, INTRO, BACKGROUND_RESOURCE, INTRO_RESOURCE, CHECK_A>(),
        write::<C_WASI, C, EXPERIMENTS, METHODS, EXPERIMENTS_RESOURCE, METHODS_RESOURCE, CHECK_C>(),
    );
    let writes_1 = g::par(
        write::<B_WASI, B, BACKGROUND, METHODS, BACKGROUND_RESOURCE, METHODS_RESOURCE, CHECK_B>(),
        write::<D_WASI, D, EXPERIMENTS, RESULTS, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE, CHECK_D>(),
    );
    let writes_2 = write::<E_WASI, E, INTRO, RESULTS, INTRO_RESOURCE, RESULTS_RESOURCE, CHECK_E>();
    let task = g::seq(writes_0, g::seq(writes_1, writes_2));
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
    Programs2M {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        background: project(&protocol),
        background_resource: project(&protocol),
        intro: project(&protocol),
        intro_resource: project(&protocol),
        methods: project(&protocol),
        methods_resource: project(&protocol),
        experiments: project(&protocol),
        experiments_resource: project(&protocol),
        results: project(&protocol),
        results_resource: project(&protocol),
        a_wasi: project(&protocol),
        b_wasi: project(&protocol),
        c_wasi: project(&protocol),
        d_wasi: project(&protocol),
        e_wasi: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        c_io: project(&protocol),
        d_io: project(&protocol),
        e_io: project(&protocol),
    }
}

pub mod roles2h {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const F: u8 = 5;
    pub const G: u8 = 6;
    pub const BACKGROUND: u8 = 7;
    pub const BACKGROUND_RESOURCE: u8 = 28;
    pub const INTRO: u8 = 8;
    pub const INTRO_RESOURCE: u8 = 29;
    pub const RELATED: u8 = 9;
    pub const RELATED_RESOURCE: u8 = 30;
    pub const METHODS: u8 = 10;
    pub const METHODS_RESOURCE: u8 = 31;
    pub const EXPERIMENTS: u8 = 11;
    pub const EXPERIMENTS_RESOURCE: u8 = 32;
    pub const RESULTS: u8 = 12;
    pub const RESULTS_RESOURCE: u8 = 33;
    pub const DISCUSSION: u8 = 13;
    pub const DISCUSSION_RESOURCE: u8 = 34;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
    pub const F_WASI: u8 = 19;
    pub const F_IO: u8 = 26;
    pub const G_WASI: u8 = 20;
    pub const G_IO: u8 = 27;
}
pub struct Programs2H {
    pub a: RoleProgram<{ roles2h::A }>,
    pub b: RoleProgram<{ roles2h::B }>,
    pub c: RoleProgram<{ roles2h::C }>,
    pub d: RoleProgram<{ roles2h::D }>,
    pub e: RoleProgram<{ roles2h::E }>,
    pub f: RoleProgram<{ roles2h::F }>,
    pub g: RoleProgram<{ roles2h::G }>,
    pub background: RoleProgram<{ roles2h::BACKGROUND }>,
    pub background_resource: RoleProgram<{ roles2h::BACKGROUND_RESOURCE }>,
    pub intro: RoleProgram<{ roles2h::INTRO }>,
    pub intro_resource: RoleProgram<{ roles2h::INTRO_RESOURCE }>,
    pub related: RoleProgram<{ roles2h::RELATED }>,
    pub related_resource: RoleProgram<{ roles2h::RELATED_RESOURCE }>,
    pub methods: RoleProgram<{ roles2h::METHODS }>,
    pub methods_resource: RoleProgram<{ roles2h::METHODS_RESOURCE }>,
    pub experiments: RoleProgram<{ roles2h::EXPERIMENTS }>,
    pub experiments_resource: RoleProgram<{ roles2h::EXPERIMENTS_RESOURCE }>,
    pub results: RoleProgram<{ roles2h::RESULTS }>,
    pub results_resource: RoleProgram<{ roles2h::RESULTS_RESOURCE }>,
    pub discussion: RoleProgram<{ roles2h::DISCUSSION }>,
    pub discussion_resource: RoleProgram<{ roles2h::DISCUSSION_RESOURCE }>,
    pub a_wasi: RoleProgram<{ roles2h::A_WASI }>,
    pub b_wasi: RoleProgram<{ roles2h::B_WASI }>,
    pub c_wasi: RoleProgram<{ roles2h::C_WASI }>,
    pub d_wasi: RoleProgram<{ roles2h::D_WASI }>,
    pub e_wasi: RoleProgram<{ roles2h::E_WASI }>,
    pub f_wasi: RoleProgram<{ roles2h::F_WASI }>,
    pub g_wasi: RoleProgram<{ roles2h::G_WASI }>,
    pub a_io: RoleProgram<{ roles2h::A_IO }>,
    pub b_io: RoleProgram<{ roles2h::B_IO }>,
    pub c_io: RoleProgram<{ roles2h::C_IO }>,
    pub d_io: RoleProgram<{ roles2h::D_IO }>,
    pub e_io: RoleProgram<{ roles2h::E_IO }>,
    pub f_io: RoleProgram<{ roles2h::F_IO }>,
    pub g_io: RoleProgram<{ roles2h::G_IO }>,
}
pub fn project_2h() -> Programs2H {
    use roles2h::*;
    let writes_0 = g::par(
        write::<A_WASI, A, BACKGROUND, INTRO, BACKGROUND_RESOURCE, INTRO_RESOURCE, CHECK_A>(),
        g::par(
            write::<C_WASI, C, METHODS, RELATED, METHODS_RESOURCE, RELATED_RESOURCE, CHECK_C>(),
            write::<E_WASI, E, EXPERIMENTS, RESULTS, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE, CHECK_E>(
            ),
        ),
    );
    let writes_1 = g::par(
        write::<B_WASI, B, BACKGROUND, RELATED, BACKGROUND_RESOURCE, RELATED_RESOURCE, CHECK_B>(),
        g::par(
            write::<D_WASI, D, EXPERIMENTS, METHODS, EXPERIMENTS_RESOURCE, METHODS_RESOURCE, CHECK_D>(
            ),
            write::<F_WASI, F, DISCUSSION, RESULTS, DISCUSSION_RESOURCE, RESULTS_RESOURCE, CHECK_F>(
            ),
        ),
    );
    let writes_2 =
        write::<G_WASI, G, DISCUSSION, INTRO, DISCUSSION_RESOURCE, INTRO_RESOURCE, CHECK_G>();
    let task = g::seq(writes_0, g::seq(writes_1, writes_2));
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
    Programs2H {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        f: project(&protocol),
        g: project(&protocol),
        background: project(&protocol),
        background_resource: project(&protocol),
        intro: project(&protocol),
        intro_resource: project(&protocol),
        related: project(&protocol),
        related_resource: project(&protocol),
        methods: project(&protocol),
        methods_resource: project(&protocol),
        experiments: project(&protocol),
        experiments_resource: project(&protocol),
        results: project(&protocol),
        results_resource: project(&protocol),
        discussion: project(&protocol),
        discussion_resource: project(&protocol),
        a_wasi: project(&protocol),
        b_wasi: project(&protocol),
        c_wasi: project(&protocol),
        d_wasi: project(&protocol),
        e_wasi: project(&protocol),
        f_wasi: project(&protocol),
        g_wasi: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        c_io: project(&protocol),
        d_io: project(&protocol),
        e_io: project(&protocol),
        f_io: project(&protocol),
        g_io: project(&protocol),
    }
}
