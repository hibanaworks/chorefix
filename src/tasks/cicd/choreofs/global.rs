//! 16E file operations and resource ownership share one global protocol.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;

pub const DEVELOPER: u8 = 0;
pub const TESTER: u8 = 1;
pub const DEPLOYER: u8 = 2;
pub const BUILD_SERVER: u8 = 3;
pub const DEPLOY_ENV: u8 = 4;
pub const DEV_VM: u8 = 5;
pub const DEV_IO: u8 = 6;
pub const TEST_VM: u8 = 7;
pub const TEST_IO: u8 = 8;
pub const DEPLOY_VM: u8 = 9;
pub const DEPLOY_IO: u8 = 10;
pub const COMPILE_CLOSE: u16 = 160;
pub const TEST_CLOSE: u16 = 161;
pub const DEPLOY_CLOSE: u16 = 162;

pub type Acquire = g::Msg<1, ()>;
pub type Granted = g::Msg<2, ()>;
pub type Open = g::Msg<3, protocol::PathOpenReq>;
pub type Opened = g::Msg<4, protocol::PathOpenedRet>;
pub type Write = g::Msg<5, protocol::FdWriteReq>;
pub type Written = g::Msg<6, protocol::FdWriteDoneRet>;
pub type Commit = g::Msg<7, protocol::FdCloseReq>;
pub type Committed<'a> = g::Msg<8, &'a [u8]>;
pub type Rejected<'a> = g::Msg<9, &'a [u8]>;
pub type Return = g::Msg<10, ()>;
pub type Returned = g::Msg<11, ()>;
pub type Materialize = g::Msg<12, ()>;
pub type BuildComplete = g::Msg<20, ()>;
pub type TestsPassed = g::Msg<21, ()>;
pub type DeployedToDeveloper = g::Msg<22, ()>;
pub type DeployedToTester = g::Msg<23, ()>;

pub struct Programs {
    pub developer: RoleProgram<DEVELOPER>,
    pub tester: RoleProgram<TESTER>,
    pub deployer: RoleProgram<DEPLOYER>,
    pub build_server: RoleProgram<BUILD_SERVER>,
    pub deploy_env: RoleProgram<DEPLOY_ENV>,
    pub dev_vm: RoleProgram<DEV_VM>,
    pub dev_io: RoleProgram<DEV_IO>,
    pub test_vm: RoleProgram<TEST_VM>,
    pub test_io: RoleProgram<TEST_IO>,
    pub deploy_vm: RoleProgram<DEPLOY_VM>,
    pub deploy_io: RoleProgram<DEPLOY_IO>,
}

pub fn choreography() -> impl Projectable {
    let compile_file = g::route(
        g::seq(
            g::send::<DEVELOPER, BUILD_SERVER, Open>(),
            g::send::<BUILD_SERVER, DEVELOPER, Opened>(),
        ),
        g::route(
            g::seq(
                g::send::<DEVELOPER, BUILD_SERVER, Write>(),
                g::send::<BUILD_SERVER, DEVELOPER, Written>(),
            ),
            g::seq(
                g::send::<DEVELOPER, BUILD_SERVER, Commit>(),
                g::route(
                    g::seq(
                        g::send::<BUILD_SERVER, BUILD_SERVER, Materialize>(),
                        g::send::<BUILD_SERVER, DEVELOPER, Committed<'static>>(),
                    ),
                    g::send::<BUILD_SERVER, DEVELOPER, Rejected<'static>>(),
                )
                .resolve::<COMPILE_CLOSE>(),
            ),
        ),
    )
    .roll();
    let build = g::seq(
        g::send::<DEVELOPER, BUILD_SERVER, Acquire>(),
        g::seq(
            g::send::<BUILD_SERVER, DEVELOPER, Granted>(),
            g::seq(
                compile_file,
                g::seq(
                    g::send::<DEVELOPER, BUILD_SERVER, Return>(),
                    g::seq(
                        g::send::<BUILD_SERVER, DEVELOPER, Returned>(),
                        g::send::<DEVELOPER, TESTER, BuildComplete>(),
                    ),
                ),
            ),
        ),
    );

    let test_file = g::route(
        g::seq(
            g::send::<TESTER, BUILD_SERVER, Open>(),
            g::send::<BUILD_SERVER, TESTER, Opened>(),
        ),
        g::route(
            g::seq(
                g::send::<TESTER, BUILD_SERVER, Write>(),
                g::send::<BUILD_SERVER, TESTER, Written>(),
            ),
            g::seq(
                g::send::<TESTER, BUILD_SERVER, Commit>(),
                g::route(
                    g::seq(
                        g::send::<BUILD_SERVER, BUILD_SERVER, Materialize>(),
                        g::send::<BUILD_SERVER, TESTER, Committed<'static>>(),
                    ),
                    g::send::<BUILD_SERVER, TESTER, Rejected<'static>>(),
                )
                .resolve::<TEST_CLOSE>(),
            ),
        ),
    )
    .roll();
    let test = g::seq(
        g::send::<TESTER, BUILD_SERVER, Acquire>(),
        g::seq(
            g::send::<BUILD_SERVER, TESTER, Granted>(),
            g::seq(
                test_file,
                g::seq(
                    g::send::<TESTER, BUILD_SERVER, Return>(),
                    g::seq(
                        g::send::<BUILD_SERVER, TESTER, Returned>(),
                        g::send::<TESTER, DEPLOYER, TestsPassed>(),
                    ),
                ),
            ),
        ),
    );

    let deploy_file = g::route(
        g::seq(
            g::send::<DEPLOYER, DEPLOY_ENV, Open>(),
            g::send::<DEPLOY_ENV, DEPLOYER, Opened>(),
        ),
        g::route(
            g::seq(
                g::send::<DEPLOYER, DEPLOY_ENV, Write>(),
                g::send::<DEPLOY_ENV, DEPLOYER, Written>(),
            ),
            g::seq(
                g::send::<DEPLOYER, DEPLOY_ENV, Commit>(),
                g::route(
                    g::seq(
                        g::send::<DEPLOY_ENV, DEPLOY_ENV, Materialize>(),
                        g::send::<DEPLOY_ENV, DEPLOYER, Committed<'static>>(),
                    ),
                    g::send::<DEPLOY_ENV, DEPLOYER, Rejected<'static>>(),
                )
                .resolve::<DEPLOY_CLOSE>(),
            ),
        ),
    )
    .roll();
    let deploy = g::seq(
        g::send::<DEPLOYER, DEPLOY_ENV, Acquire>(),
        g::seq(
            g::send::<DEPLOY_ENV, DEPLOYER, Granted>(),
            g::seq(
                deploy_file,
                g::seq(
                    g::send::<DEPLOYER, DEPLOY_ENV, Return>(),
                    g::seq(
                        g::send::<DEPLOY_ENV, DEPLOYER, Returned>(),
                        g::par(
                            g::send::<DEPLOYER, DEVELOPER, DeployedToDeveloper>(),
                            g::send::<DEPLOYER, TESTER, DeployedToTester>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(build, g::seq(test, deploy))
}

pub fn programs() -> Programs {
    let task = choreography();
    let dev_io = crate::shell::global::private_io::<DEV_VM, DEV_IO>();
    let test_io = crate::shell::global::private_io::<TEST_VM, TEST_IO>();
    let deploy_io = crate::shell::global::private_io::<DEPLOY_VM, DEPLOY_IO>();
    let protocol = g::par(task, g::par(dev_io, g::par(test_io, deploy_io)));
    Programs {
        developer: project(&protocol),
        tester: project(&protocol),
        deployer: project(&protocol),
        build_server: project(&protocol),
        deploy_env: project(&protocol),
        dev_vm: project(&protocol),
        dev_io: project(&protocol),
        test_vm: project(&protocol),
        test_io: project(&protocol),
        deploy_vm: project(&protocol),
        deploy_io: project(&protocol),
    }
}
