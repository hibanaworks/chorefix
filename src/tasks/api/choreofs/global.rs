//! Task 8E: actual WASI requests, physical resources and result-file reads.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;
pub const ADMIN: u8 = 0;
pub const BACKEND: u8 = 1;
pub const TESTER: u8 = 2;
pub const DATABASE: u8 = 3;
pub const TEST_ENV: u8 = 4;
pub const ADMIN_VM: u8 = 5;
pub const BACKEND_VM: u8 = 6;
pub const TESTER_VM: u8 = 7;
pub const ADMIN_IO: u8 = 8;
pub const BACKEND_IO: u8 = 9;
pub const TESTER_IO: u8 = 10;
pub const ADMIN_OUTCOME: u8 = 11;
pub const BACKEND_OUTCOME: u8 = 12;
pub const ADMIN_OUTCOME_VM: u8 = 13;
pub const BACKEND_OUTCOME_VM: u8 = 14;
pub const CHECK_ADMIN: u16 = 680;
pub const CHECK_BACKEND: u16 = 681;
pub const CHECK_TESTER: u16 = 682;
pub const TEST_RESULT: u16 = 683;
pub const SERVICE_FD: u8 = 4;
pub const SERVICE_PREFIX: &[u8] = b"service/";
pub const RESULT_PATH: &[u8] = b"service/result";
pub const OUTCOME_PATH: &[u8] = b"service/outcome";
pub const OUTCOME_FD: u8 = 16;
pub type Open = g::Msg<100, protocol::PathOpenReq>;
pub type Opened = g::Msg<101, protocol::PathOpenedRet>;
pub type Write = g::Msg<102, protocol::FdWriteReq>;
pub type Written = g::Msg<103, protocol::FdWriteDoneRet>;
pub type Commit = g::Msg<104, protocol::FdCloseReq>;
pub type Admitted = g::Msg<105, ()>;
pub type Rejected<'a> = g::Msg<106, &'a [u8]>;
pub type ResultBytes<'a> = g::Msg<107, &'a [u8]>;
pub type Unavailable = g::Msg<109, protocol::FdClosedRet>;
pub type RequestReady = g::Msg<110, ()>;
pub type Validate = g::Msg<111, ()>;
pub type Request = g::Msg<112, ()>;
pub type RejectRequest = g::Msg<113, ()>;
pub type Effect = g::Msg<115, ()>;
pub type ServiceExit = g::Msg<241, i32>;
pub type Acquire = g::Msg<10, ()>;
pub type Granted = g::Msg<11, ()>;
pub type Return = g::Msg<12, ()>;
pub type Returned = g::Msg<13, ()>;
pub type MigrationDone = g::Msg<40, ()>;
pub type BackendReady = g::Msg<41, ()>;
pub type PassedAdmin<'a> = g::Msg<50, &'a [u8]>;
pub type PassedBackend<'a> = g::Msg<51, &'a [u8]>;
pub type FailedAdmin<'a> = g::Msg<60, &'a [u8]>;
pub type FailedBackend<'a> = g::Msg<61, &'a [u8]>;
pub type OutcomeOpen = g::Msg<150, protocol::PathOpenReq>;
pub type OutcomeOpened = g::Msg<151, protocol::PathOpenedRet>;
pub type OutcomeRead = g::Msg<152, protocol::FdReadReq>;
pub type OutcomeReadDone = g::Msg<153, protocol::FdReadDoneRet>;
pub type OutcomeStat = g::Msg<154, protocol::FdFdstatGetReq>;
pub type OutcomeStatDone = g::Msg<155, protocol::FdStatRet>;
pub type OutcomeFileStat = g::Msg<156, protocol::FdFilestatGetReq>;
pub type OutcomeFileStatDone = g::Msg<157, protocol::FdFilestatRet>;
pub type OutcomeClose = g::Msg<158, protocol::FdCloseReq>;
pub type OutcomeClosed = g::Msg<159, protocol::FdClosedRet>;
pub type OutcomeExit = g::Msg<242, i32>;
fn requests<const VM: u8, const OWNER: u8, const CHECK: u16>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, OWNER, Request>(),
        g::seq(
            g::send::<OWNER, VM, RequestReady>(),
            g::route(
                g::seq(g::send::<VM, OWNER, Open>(), g::send::<OWNER, VM, Opened>()),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, Write>(),
                        g::send::<OWNER, VM, Written>(),
                    ),
                    g::seq(
                        g::send::<VM, OWNER, Commit>(),
                        g::route(
                            g::seq(
                                g::send::<OWNER, OWNER, Validate>(),
                                g::send::<OWNER, VM, Admitted>(),
                            ),
                            g::send::<OWNER, VM, Rejected<'static>>(),
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
            g::send::<OWNER, VM, RequestReady>(),
            g::route(
                g::seq(g::send::<VM, OWNER, Open>(), g::send::<OWNER, VM, Opened>()),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, Write>(),
                        g::send::<OWNER, VM, Written>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<VM, OWNER, Commit>(),
                            g::send::<OWNER, VM, Unavailable>(),
                        ),
                        g::send::<VM, OWNER, ServiceExit>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn outcome_io<const VM: u8, const OWNER: u8>() -> impl Projectable {
    // Opening the file is a required result-receipt step. Its owner must receive
    // the test notification before entering this fragment or answering any read.
    let first_open = g::send::<OWNER, VM, OutcomeOpened>();
    let io = g::route(
        g::seq(
            g::send::<VM, OWNER, OutcomeOpen>(),
            g::send::<OWNER, VM, OutcomeOpened>(),
        ),
        g::route(
            g::seq(
                g::send::<VM, OWNER, OutcomeRead>(),
                g::send::<OWNER, VM, OutcomeReadDone>(),
            ),
            g::route(
                g::seq(
                    g::send::<VM, OWNER, OutcomeStat>(),
                    g::send::<OWNER, VM, OutcomeStatDone>(),
                ),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, OutcomeFileStat>(),
                        g::send::<OWNER, VM, OutcomeFileStatDone>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<VM, OWNER, OutcomeClose>(),
                            g::send::<OWNER, VM, OutcomeClosed>(),
                        ),
                        g::send::<VM, OWNER, OutcomeExit>(),
                    ),
                ),
            ),
        ),
    )
    .roll();
    g::seq(first_open, io)
}
pub struct Programs {
    pub admin: RoleProgram<ADMIN>,
    pub backend: RoleProgram<BACKEND>,
    pub tester: RoleProgram<TESTER>,
    pub database: RoleProgram<DATABASE>,
    pub test_env: RoleProgram<TEST_ENV>,
    pub admin_vm: RoleProgram<ADMIN_VM>,
    pub backend_vm: RoleProgram<BACKEND_VM>,
    pub tester_vm: RoleProgram<TESTER_VM>,
    pub admin_io: RoleProgram<ADMIN_IO>,
    pub backend_io: RoleProgram<BACKEND_IO>,
    pub tester_io: RoleProgram<TESTER_IO>,
    pub admin_outcome: RoleProgram<ADMIN_OUTCOME>,
    pub backend_outcome: RoleProgram<BACKEND_OUTCOME>,
    pub admin_outcome_vm: RoleProgram<ADMIN_OUTCOME_VM>,
    pub backend_outcome_vm: RoleProgram<BACKEND_OUTCOME_VM>,
}
pub fn programs() -> Programs {
    let migration = g::seq(
        requests::<ADMIN_VM, ADMIN, CHECK_ADMIN>(),
        g::seq(
            g::send::<ADMIN, DATABASE, Acquire>(),
            g::seq(
                g::send::<DATABASE, ADMIN, Granted>(),
                g::seq(
                    g::send::<ADMIN, ADMIN, Effect>(),
                    g::seq(
                        g::send::<ADMIN, DATABASE, Return>(),
                        g::seq(
                            g::send::<DATABASE, ADMIN, Returned>(),
                            g::seq(
                                g::send::<ADMIN, ADMIN_VM, ResultBytes<'static>>(),
                                g::send::<ADMIN, BACKEND, MigrationDone>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let backend = g::seq(
        requests::<BACKEND_VM, BACKEND, CHECK_BACKEND>(),
        g::seq(
            g::send::<BACKEND, BACKEND, Effect>(),
            g::seq(
                g::send::<BACKEND, BACKEND_VM, ResultBytes<'static>>(),
                g::send::<BACKEND, TESTER, BackendReady>(),
            ),
        ),
    );
    let test = g::seq(
        requests::<TESTER_VM, TESTER, CHECK_TESTER>(),
        g::seq(
            g::send::<TESTER, TEST_ENV, Acquire>(),
            g::seq(
                g::send::<TEST_ENV, TESTER, Granted>(),
                g::seq(
                    g::send::<TESTER, TESTER, Effect>(),
                    g::seq(
                        g::send::<TESTER, TEST_ENV, Return>(),
                        g::seq(
                            g::send::<TEST_ENV, TESTER, Returned>(),
                            g::send::<TESTER, TESTER_VM, ResultBytes<'static>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let outcome = g::route(
        g::par(
            g::send::<TESTER, ADMIN_OUTCOME, PassedAdmin<'static>>(),
            g::send::<TESTER, BACKEND_OUTCOME, PassedBackend<'static>>(),
        ),
        g::par(
            g::send::<TESTER, ADMIN_OUTCOME, FailedAdmin<'static>>(),
            g::send::<TESTER, BACKEND_OUTCOME, FailedBackend<'static>>(),
        ),
    )
    .resolve::<TEST_RESULT>();
    let task = g::seq(
        migration,
        g::par(
            closed::<ADMIN_VM, ADMIN>(),
            g::seq(
                backend,
                g::par(
                    closed::<BACKEND_VM, BACKEND>(),
                    g::seq(test, g::seq(outcome, closed::<TESTER_VM, TESTER>())),
                ),
            ),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<ADMIN_VM, ADMIN_IO>(),
        g::par(
            crate::shell::global::private_io::<BACKEND_VM, BACKEND_IO>(),
            crate::shell::global::private_io::<TESTER_VM, TESTER_IO>(),
        ),
    );
    let results = g::seq(
        g::par(
            task,
            g::par(
                g::send::<ADMIN_OUTCOME_VM, ADMIN_OUTCOME, OutcomeOpen>(),
                g::send::<BACKEND_OUTCOME_VM, BACKEND_OUTCOME, OutcomeOpen>(),
            ),
        ),
        g::par(
            outcome_io::<ADMIN_OUTCOME_VM, ADMIN_OUTCOME>(),
            outcome_io::<BACKEND_OUTCOME_VM, BACKEND_OUTCOME>(),
        ),
    );
    let global = g::par(results, private);
    Programs {
        admin: project(&global),
        backend: project(&global),
        tester: project(&global),
        database: project(&global),
        test_env: project(&global),
        admin_vm: project(&global),
        backend_vm: project(&global),
        tester_vm: project(&global),
        admin_io: project(&global),
        backend_io: project(&global),
        tester_io: project(&global),
        admin_outcome: project(&global),
        backend_outcome: project(&global),
        admin_outcome_vm: project(&global),
        backend_outcome_vm: project(&global),
    }
}
