//! 8E: schema migration, backend work, then one terminal integration test.
use hibana::{
    g,
    runtime::program::{RoleProgram, project},
};

pub const DB_ADMIN: u8 = 0;
pub const BACKEND_DEV: u8 = 1;
pub const TESTER: u8 = 2;
pub const DATABASE: u8 = 3;
pub const TEST_ENV: u8 = 4;
pub const TEST_RESULT: u16 = 80;

pub type Migrate = g::Msg<1, ()>;
pub type Implement = g::Msg<2, ()>;
pub type Test = g::Msg<3, ()>;
pub type Acquire = g::Msg<10, ()>;
pub type Granted = g::Msg<11, ()>;
pub type Return = g::Msg<12, ()>;
pub type Returned = g::Msg<13, ()>;
pub type MigrationDone = g::Msg<40, ()>;
pub type BackendReady = g::Msg<41, ()>;
pub type PassedToBackend = g::Msg<50, ()>;
pub type PassedToAdmin = g::Msg<51, ()>;
pub type FailedToBackend = g::Msg<60, ()>;
pub type FailedToAdmin = g::Msg<61, ()>;

pub struct Programs8E {
    pub admin: RoleProgram<DB_ADMIN>,
    pub backend: RoleProgram<BACKEND_DEV>,
    pub tester: RoleProgram<TESTER>,
    pub database: RoleProgram<DATABASE>,
    pub test_env: RoleProgram<TEST_ENV>,
}

pub fn project_8e() -> Programs8E {
    let migration = g::seq(
        g::send::<DB_ADMIN, DATABASE, Acquire>(),
        g::seq(
            g::send::<DATABASE, DB_ADMIN, Granted>(),
            g::seq(
                g::send::<DB_ADMIN, DB_ADMIN, Migrate>(),
                g::seq(
                    g::send::<DB_ADMIN, DATABASE, Return>(),
                    g::seq(
                        g::send::<DATABASE, DB_ADMIN, Returned>(),
                        g::send::<DB_ADMIN, BACKEND_DEV, MigrationDone>(),
                    ),
                ),
            ),
        ),
    );
    let backend = g::seq(
        g::send::<BACKEND_DEV, BACKEND_DEV, Implement>(),
        g::send::<BACKEND_DEV, TESTER, BackendReady>(),
    );
    let test = g::seq(
        g::send::<TESTER, TEST_ENV, Acquire>(),
        g::seq(
            g::send::<TEST_ENV, TESTER, Granted>(),
            g::seq(
                g::send::<TESTER, TESTER, Test>(),
                g::seq(
                    g::send::<TESTER, TEST_ENV, Return>(),
                    g::send::<TEST_ENV, TESTER, Returned>(),
                ),
            ),
        ),
    );
    let result = g::route(
        g::par(
            g::send::<TESTER, BACKEND_DEV, PassedToBackend>(),
            g::send::<TESTER, DB_ADMIN, PassedToAdmin>(),
        ),
        g::par(
            g::send::<TESTER, BACKEND_DEV, FailedToBackend>(),
            g::send::<TESTER, DB_ADMIN, FailedToAdmin>(),
        ),
    )
    .resolve::<TEST_RESULT>();
    let protocol = g::seq(migration, g::seq(backend, g::seq(test, result)));
    Programs8E {
        admin: project(&protocol),
        backend: project(&protocol),
        tester: project(&protocol),
        database: project(&protocol),
        test_env: project(&protocol),
    }
}
