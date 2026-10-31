//! Every service request originates at the WASI Endpoint in this global.
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;

pub const DEV_WASI: u8 = 0;
pub const TEST_WASI: u8 = 1;
pub const DEPLOY_WASI: u8 = 2;
pub const DEVELOPER: u8 = 3;
pub const TESTER: u8 = 4;
pub const DEPLOYER: u8 = 5;
pub const BUILD_OWNER: u8 = 6;
pub const BUILD_SERVER: u8 = 11;
pub const DEPLOY_OWNER: u8 = 7;
pub const DEPLOY_ENV: u8 = 12;
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
pub type Write = g::Msg<5, protocol::FdWriteObjectReq>;
pub type Written = g::Msg<6, protocol::FdWriteDoneRet>;
pub type Commit = g::Msg<7, protocol::FdCloseReq>;
pub type Admitted<'a> = g::Msg<8, &'a [u8]>;
pub type Rejected<'a> = g::Msg<9, &'a [u8]>;
pub type Loan<'r, const RESOURCE: u8> = g::Msg<10, Endpoint<'r, RESOURCE>>;
pub type Validate = g::Msg<12, ()>;
pub type CompileRequest = g::Msg<30, super::imp::Arguments<1>>;
pub type CompileResult<'a> = g::Msg<31, &'a [u8]>;
pub type TestRequest = g::Msg<32, super::imp::Arguments<2>>;
pub type TestResult<'a> = g::Msg<33, &'a [u8]>;
pub type DeployRequest = g::Msg<34, super::imp::Arguments<3>>;
pub type DeployResult<'a> = g::Msg<35, &'a [u8]>;
pub type Unavailable = g::Msg<16, protocol::FdClosedRet>;
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
    pub build_owner: RoleProgram<BUILD_OWNER>,
    pub build_server: RoleProgram<BUILD_SERVER>,
    pub deploy_owner: RoleProgram<DEPLOY_OWNER>,
    pub deploy_env: RoleProgram<DEPLOY_ENV>,
    pub dev_io: RoleProgram<DEV_IO>,
    pub test_io: RoleProgram<TEST_IO>,
    pub deploy_io: RoleProgram<DEPLOY_IO>,
}

// The owner validates the received WASI request before handing it back to the guest.
fn service<const VM: u8, const OWNER: u8, const CHECK: u16>() -> impl Projectable {
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
                                g::send::<OWNER, OWNER, Validate>(),
                                g::send::<OWNER, VM, Admitted>(),
                            ),
                            g::send::<OWNER, VM, Rejected>(),
                        )
                        .resolve::<CHECK>(),
                    ),
                ),
            ),
        ),
    )
    .roll();
    g::seq(g::send::<OWNER, VM, Opened>(), requests)
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
                g::seq(
                    service::<DEV_WASI, DEVELOPER, COMPILE_CLOSE>(),
                    g::seq(
                        g::send::<BUILD_OWNER, DEV_WASI, Loan<BUILD_SERVER>>(),
                        g::seq(
                            g::send::<BUILD_SERVER, DEVELOPER, CompileRequest>(),
                            g::seq(
                                g::send::<DEVELOPER, BUILD_SERVER, CompileResult>(),
                                g::send::<DEV_WASI, BUILD_OWNER, Loan<BUILD_SERVER>>(),
                            ),
                        ),
                    ),
                ),
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
        g::seq(
            service::<TEST_WASI, TESTER, TEST_CLOSE>(),
            g::seq(
                g::send::<BUILD_OWNER, TEST_WASI, Loan<BUILD_SERVER>>(),
                g::seq(
                    g::send::<BUILD_SERVER, TESTER, TestRequest>(),
                    g::seq(
                        g::send::<TESTER, BUILD_SERVER, TestResult>(),
                        g::send::<TEST_WASI, BUILD_OWNER, Loan<BUILD_SERVER>>(),
                    ),
                ),
            ),
        ),
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
                        g::seq(
                            service::<DEPLOY_WASI, DEPLOYER, DEPLOY_CLOSE>(),
                            g::seq(
                                g::send::<DEPLOY_OWNER, DEPLOY_WASI, Loan<DEPLOY_ENV>>(),
                                g::seq(
                                    g::send::<DEPLOY_ENV, DEPLOYER, DeployRequest>(),
                                    g::seq(
                                        g::send::<DEPLOYER, DEPLOY_ENV, DeployResult>(),
                                        g::send::<DEPLOY_WASI, DEPLOY_OWNER, Loan<DEPLOY_ENV>>(),
                                    ),
                                ),
                            ),
                        ),
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
        build_owner: project(&protocol),
        build_server: project(&protocol),
        deploy_owner: project(&protocol),
        deploy_env: project(&protocol),
        dev_io: project(&protocol),
        test_io: project(&protocol),
        deploy_io: project(&protocol),
    }
}
