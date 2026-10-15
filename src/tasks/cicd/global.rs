//! 16E is one sequential build/test/deploy pipeline with explicit resource roles.
use hibana::{
    g,
    runtime::program::{RoleProgram, project},
};

pub const DEVELOPER: u8 = 0;
pub const TESTER: u8 = 1;
pub const DEPLOYER: u8 = 2;
pub const BUILD_SERVER: u8 = 3;
pub const DEPLOY_ENV: u8 = 4;

pub type Compile = g::Msg<1, ()>;
pub type Test = g::Msg<2, ()>;
pub type Deploy = g::Msg<3, ()>;
pub type Acquire = g::Msg<10, ()>;
pub type Granted = g::Msg<11, ()>;
pub type Return = g::Msg<12, ()>;
pub type Returned = g::Msg<13, ()>;
pub type BuildComplete = g::Msg<20, ()>;
pub type TestsPassed = g::Msg<21, ()>;
pub type DeployedToDeveloper = g::Msg<22, ()>;
pub type DeployedToTester = g::Msg<23, ()>;

pub struct Programs16E {
    pub developer: RoleProgram<DEVELOPER>,
    pub tester: RoleProgram<TESTER>,
    pub deployer: RoleProgram<DEPLOYER>,
    pub build_server: RoleProgram<BUILD_SERVER>,
    pub deploy_env: RoleProgram<DEPLOY_ENV>,
}

pub fn project_16e() -> Programs16E {
    let build = g::seq(
        g::send::<DEVELOPER, BUILD_SERVER, Acquire>(),
        g::seq(
            g::send::<BUILD_SERVER, DEVELOPER, Granted>(),
            g::seq(
                g::send::<DEVELOPER, DEVELOPER, Compile>(),
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
    let test = g::seq(
        g::send::<TESTER, BUILD_SERVER, Acquire>(),
        g::seq(
            g::send::<BUILD_SERVER, TESTER, Granted>(),
            g::seq(
                g::send::<TESTER, TESTER, Test>(),
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
    let deploy = g::seq(
        g::send::<DEPLOYER, DEPLOY_ENV, Acquire>(),
        g::seq(
            g::send::<DEPLOY_ENV, DEPLOYER, Granted>(),
            g::seq(
                g::send::<DEPLOYER, DEPLOYER, Deploy>(),
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
    let protocol = g::seq(build, g::seq(test, deploy));
    Programs16E {
        developer: project(&protocol),
        tester: project(&protocol),
        deployer: project(&protocol),
        build_server: project(&protocol),
        deploy_env: project(&protocol),
    }
}
