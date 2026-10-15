//! Developer workflows and physical module ownership in one global.
use crate::resource::global::*;
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
pub mod reconsider;
pub use reconsider::project_1h;
pub type Design = g::Msg<1, ()>;
pub type Implement = g::Msg<2, ()>;
pub type Commit = g::Msg<3, ()>;
pub type Test = g::Msg<4, ()>;
pub type Finished = g::Msg<5, ()>;
pub const A: u8 = 0;
pub const TEST_A: u16 = 100;
pub const B: u8 = 1;
pub const TEST_B: u16 = 101;
pub const C: u8 = 2;
pub const TEST_C: u16 = 102;
pub const D: u8 = 3;
pub const TEST_D: u16 = 103;
pub const E: u8 = 4;
pub const TEST_E: u16 = 104;
pub const F: u8 = 5;
pub const TEST_F: u16 = 105;
pub const G: u8 = 6;
pub const TEST_G: u16 = 106;
pub const ANALYTICS_MODULE: u8 = 35;
pub const SEARCH_MODULE: u8 = 36;
pub type AvailableFirst = g::Msg<152, ()>;
pub type AvailableSecond = g::Msg<153, ()>;
pub const AUTH_MODULE: u8 = 30;
pub const DATABASE_MODULE: u8 = 31;
pub const API_MODULE: u8 = 32;
pub const PAYMENT_MODULE: u8 = 33;
pub const NOTIFY_MODULE: u8 = 34;

fn terminal_developer<const DEV: u8, const FIRST: u8, const SECOND: u8>() -> impl Projectable {
    let preparation = g::seq(
        g::send::<DEV, DEV, Design>(),
        g::send::<DEV, DEV, Implement>(),
    );
    let availability = g::par(
        g::send::<FIRST, DEV, AvailableFirst>(),
        g::send::<SECOND, DEV, AvailableSecond>(),
    );
    let acquire = g::par(
        g::seq(
            g::send::<DEV, FIRST, AcquireFirst>(),
            g::send::<FIRST, DEV, GrantedFirst>(),
        ),
        g::seq(
            g::send::<DEV, SECOND, AcquireSecond>(),
            g::send::<SECOND, DEV, GrantedSecond>(),
        ),
    );
    let release = g::par(
        g::seq(
            g::send::<DEV, SECOND, ReturnSecond>(),
            g::send::<SECOND, DEV, ReturnedSecond>(),
        ),
        g::seq(
            g::send::<DEV, FIRST, ReturnFirst>(),
            g::send::<FIRST, DEV, ReturnedFirst>(),
        ),
    );
    g::seq(
        g::par(preparation, availability),
        g::seq(
            acquire,
            g::seq(
                g::send::<DEV, DEV, Commit>(),
                g::seq(
                    release,
                    g::seq(g::send::<DEV, DEV, Test>(), g::send::<DEV, DEV, Finished>()),
                ),
            ),
        ),
    )
}

pub struct Programs1E {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub auth: RoleProgram<AUTH_MODULE>,
    pub database: RoleProgram<DATABASE_MODULE>,
    pub api: RoleProgram<API_MODULE>,
}
pub fn project_1e() -> Programs1E {
    let a = terminal_developer::<A, AUTH_MODULE, DATABASE_MODULE>();
    let b = terminal_developer::<B, DATABASE_MODULE, API_MODULE>();
    let c = terminal_developer::<C, AUTH_MODULE, API_MODULE>();
    let protocol = g::seq(a, g::seq(b, c));
    Programs1E {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        auth: project(&protocol),
        database: project(&protocol),
        api: project(&protocol),
    }
}
pub struct Programs1M {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub d: RoleProgram<D>,
    pub e: RoleProgram<E>,
    pub auth: RoleProgram<AUTH_MODULE>,
    pub database: RoleProgram<DATABASE_MODULE>,
    pub api: RoleProgram<API_MODULE>,
    pub payment: RoleProgram<PAYMENT_MODULE>,
    pub notify: RoleProgram<NOTIFY_MODULE>,
}
pub fn project_1m() -> Programs1M {
    let a = terminal_developer::<A, AUTH_MODULE, DATABASE_MODULE>();
    let b = terminal_developer::<B, DATABASE_MODULE, API_MODULE>();
    let c = terminal_developer::<C, API_MODULE, PAYMENT_MODULE>();
    let d = terminal_developer::<D, PAYMENT_MODULE, NOTIFY_MODULE>();
    let e = terminal_developer::<E, AUTH_MODULE, NOTIFY_MODULE>();
    let protocol = g::seq(g::par(a, c), g::seq(g::par(b, d), e));
    Programs1M {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        auth: project(&protocol),
        database: project(&protocol),
        api: project(&protocol),
        payment: project(&protocol),
        notify: project(&protocol),
    }
}
