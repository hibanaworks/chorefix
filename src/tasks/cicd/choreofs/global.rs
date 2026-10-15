//! Every service request originates at the WASI Endpoint in this global.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;

pub const DEV_WASI: u8 = 0;
pub const TEST_WASI: u8 = 1;
pub const DEPLOY_WASI: u8 = 2;
pub const DEVELOPER: u8 = 3;
pub const TESTER: u8 = 4;
pub const DEPLOYER: u8 = 5;
pub const BUILD_SERVER: u8 = 6;
pub const DEPLOY_ENV: u8 = 7;
pub const DEV_IO: u8 = 8;
pub const TEST_IO: u8 = 9;
pub const DEPLOY_IO: u8 = 10;
pub const COMPILE_CLOSE: u16 = 160;
pub const TEST_CLOSE: u16 = 161;
pub const DEPLOY_CLOSE: u16 = 162;
pub const SERVICE_FD: u8 = 4;
pub const SERVICE_PREFIX: &[u8] = b"service/";
pub const RESULT_PATH: &[u8] = b"service/result";

pub type Open = g::Msg<3, protocol::PathOpenReq>;
pub type Opened = g::Msg<4, protocol::PathOpenedRet>;
pub type Write = g::Msg<5, protocol::FdWriteReq>;
pub type Written = g::Msg<6, protocol::FdWriteDoneRet>;
pub type Commit = g::Msg<7, protocol::FdCloseReq>;
pub type Committed<'a> = g::Msg<8, &'a [u8]>;
pub type Rejected<'a> = g::Msg<9, &'a [u8]>;
pub type Acquire = g::Msg<10, ()>;
pub type Granted = g::Msg<11, ()>;
pub type Materialize = g::Msg<12, ()>;
pub type Return = g::Msg<13, ()>;
pub type Returned = g::Msg<14, ()>;
pub type Unavailable = g::Msg<16, protocol::FdClosedRet>;
pub type ServiceDone = g::Msg<15, ()>;
pub type ServeRequest = g::Msg<80, ()>;
pub type RejectRequest = g::Msg<81, ()>;
pub type RequestReady = g::Msg<82, ()>;
pub type BuildComplete = g::Msg<20, ()>;
pub type TestsPassed = g::Msg<21, ()>;
pub type DeployedToDeveloper = g::Msg<22, ()>;
pub type DeployedToTester = g::Msg<23, ()>;
pub type ServiceExit = g::Msg<241, i32>;

pub struct Programs {
    pub dev_wasi: RoleProgram<DEV_WASI>,
    pub test_wasi: RoleProgram<TEST_WASI>,
    pub deploy_wasi: RoleProgram<DEPLOY_WASI>,
    pub developer: RoleProgram<DEVELOPER>,
    pub tester: RoleProgram<TESTER>,
    pub deployer: RoleProgram<DEPLOYER>,
    pub build_server: RoleProgram<BUILD_SERVER>,
    pub deploy_env: RoleProgram<DEPLOY_ENV>,
    pub dev_io: RoleProgram<DEV_IO>,
    pub test_io: RoleProgram<TEST_IO>,
    pub deploy_io: RoleProgram<DEPLOY_IO>,
}

// The entire request, resource loan, materialization and reply are protocol steps.
fn service<const VM: u8, const OWNER: u8, const RESOURCE: u8, const CHECK: u16>() -> impl Projectable
{
    let requests = g::seq(
        g::send::<OWNER, OWNER, ServeRequest>(),
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
                                g::send::<OWNER, OWNER, Materialize>(),
                                g::send::<OWNER, VM, Committed<'static>>(),
                            ),
                            g::send::<OWNER, VM, Rejected<'static>>(),
                        )
                        .resolve::<CHECK>(),
                    ),
                ),
            ),
        ),
    )
    .roll();
    g::seq(
        g::send::<OWNER, RESOURCE, Acquire>(),
        g::seq(
            g::send::<RESOURCE, OWNER, Granted>(),
            g::seq(
                g::send::<OWNER, VM, Opened>(),
                g::seq(
                    requests,
                    g::seq(
                        g::send::<OWNER, RESOURCE, Return>(),
                        g::seq(
                            g::send::<RESOURCE, OWNER, Returned>(),
                            g::send::<OWNER, VM, ServiceDone>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

// After the admitted operation, further service requests get an explicit error.
fn closed_service<const VM: u8, const OWNER: u8>() -> impl Projectable {
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

pub fn programs() -> Programs {
    let build = g::seq(
        g::send::<DEVELOPER, DEV_WASI, RequestReady>(),
        g::seq(
            g::send::<DEV_WASI, DEVELOPER, Open>(),
            g::seq(
                service::<DEV_WASI, DEVELOPER, BUILD_SERVER, COMPILE_CLOSE>(),
                g::send::<DEVELOPER, TESTER, BuildComplete>(),
            ),
        ),
    );
    let build_and_test_request = g::par(
        build,
        g::seq(
            g::send::<TESTER, TEST_WASI, RequestReady>(),
            g::send::<TEST_WASI, TESTER, Open>(),
        ),
    );
    let test = g::seq(
        service::<TEST_WASI, TESTER, BUILD_SERVER, TEST_CLOSE>(),
        g::send::<TESTER, DEPLOYER, TestsPassed>(),
    );
    let test_and_deploy_request = g::par(
        test,
        g::seq(
            g::send::<DEPLOYER, DEPLOY_WASI, RequestReady>(),
            g::send::<DEPLOY_WASI, DEPLOYER, Open>(),
        ),
    );
    // Completed service lifetimes continue independently of downstream work.
    // A guest can inspect its result, get duplicate-operation errors, and exit.
    let task = g::seq(
        build_and_test_request,
        g::par(
            closed_service::<DEV_WASI, DEVELOPER>(),
            g::seq(
                test_and_deploy_request,
                g::par(
                    closed_service::<TEST_WASI, TESTER>(),
                    g::seq(
                        service::<DEPLOY_WASI, DEPLOYER, DEPLOY_ENV, DEPLOY_CLOSE>(),
                        g::par(
                            closed_service::<DEPLOY_WASI, DEPLOYER>(),
                            g::par(
                                g::send::<DEPLOYER, DEVELOPER, DeployedToDeveloper>(),
                                g::send::<DEPLOYER, TESTER, DeployedToTester>(),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let private = g::par(
        crate::shell::global::private_io::<DEV_WASI, DEV_IO>(),
        g::par(
            crate::shell::global::private_io::<TEST_WASI, TEST_IO>(),
            crate::shell::global::private_io::<DEPLOY_WASI, DEPLOY_IO>(),
        ),
    );
    let protocol = g::par(task, private);
    Programs {
        dev_wasi: project(&protocol),
        test_wasi: project(&protocol),
        deploy_wasi: project(&protocol),
        developer: project(&protocol),
        tester: project(&protocol),
        deployer: project(&protocol),
        build_server: project(&protocol),
        deploy_env: project(&protocol),
        dev_io: project(&protocol),
        test_io: project(&protocol),
        deploy_io: project(&protocol),
    }
}
