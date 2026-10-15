//! Developer workflows and all resource handoffs. Domain calls are local steps.
use crate::resource::global::*;
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub type Design = g::Msg<1, ()>;
pub type Implement = g::Msg<2, ()>;
pub type Commit = g::Msg<3, ()>;
pub type Test = g::Msg<4, ()>;
pub type Finished = g::Msg<5, ()>;
pub type Passed = g::Msg<150, ()>;
pub type Attempt = g::Msg<151, ()>;

pub const A: u8 = 0;
pub const A_FIRST: u8 = 7;
pub const A_SECOND: u8 = 8;
pub const RESULT_A: u8 = 21;
pub const TEST_A: u16 = 100;
pub const B: u8 = 1;
pub const B_FIRST: u8 = 9;
pub const B_SECOND: u8 = 10;
pub const RESULT_B: u8 = 22;
pub const TEST_B: u16 = 101;
pub const C: u8 = 2;
pub const C_FIRST: u8 = 11;
pub const C_SECOND: u8 = 12;
pub const RESULT_C: u8 = 23;
pub const TEST_C: u16 = 102;
pub const D: u8 = 3;
pub const D_FIRST: u8 = 13;
pub const D_SECOND: u8 = 14;
pub const RESULT_D: u8 = 24;
pub const TEST_D: u16 = 103;
pub const E: u8 = 4;
pub const E_FIRST: u8 = 15;
pub const E_SECOND: u8 = 16;
pub const RESULT_E: u8 = 25;
pub const TEST_E: u16 = 104;
pub const F: u8 = 5;
pub const F_FIRST: u8 = 17;
pub const F_SECOND: u8 = 18;
pub const RESULT_F: u8 = 26;
pub const TEST_F: u16 = 105;
pub const G: u8 = 6;
pub const G_FIRST: u8 = 19;
pub const G_SECOND: u8 = 20;
pub const RESULT_G: u8 = 27;
pub const TEST_G: u16 = 106;

fn developer<
    const DEV: u8,
    const FIRST: u8,
    const SECOND: u8,
    const RESULT: u8,
    const RESOLVER: u16,
>() -> impl Projectable {
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
    let attempt = g::seq(
        g::send::<DEV, RESULT, Attempt>(),
        g::seq(
            g::send::<DEV, DEV, Implement>(),
            g::seq(
                acquire,
                g::seq(
                    g::send::<DEV, DEV, Commit>(),
                    g::seq(release, g::send::<DEV, DEV, Test>()),
                ),
            ),
        ),
    );
    let finish = g::seq(
        g::send::<DEV, RESULT, Passed>(),
        g::seq(
            g::par(
                g::send::<DEV, FIRST, ClosedFirst>(),
                g::send::<DEV, SECOND, ClosedSecond>(),
            ),
            g::send::<DEV, DEV, Finished>(),
        ),
    );
    let attempts = g::route(finish, attempt).resolve::<RESOLVER>().roll();
    g::seq(g::send::<DEV, DEV, Design>(), attempts)
}

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

pub struct Programs1H {
    pub a: RoleProgram<A>,
    pub a_first: RoleProgram<A_FIRST>,
    pub a_second: RoleProgram<A_SECOND>,
    pub result_a: RoleProgram<RESULT_A>,
    pub b: RoleProgram<B>,
    pub b_first: RoleProgram<B_FIRST>,
    pub b_second: RoleProgram<B_SECOND>,
    pub result_b: RoleProgram<RESULT_B>,
    pub c: RoleProgram<C>,
    pub c_first: RoleProgram<C_FIRST>,
    pub c_second: RoleProgram<C_SECOND>,
    pub result_c: RoleProgram<RESULT_C>,
    pub d: RoleProgram<D>,
    pub d_first: RoleProgram<D_FIRST>,
    pub d_second: RoleProgram<D_SECOND>,
    pub result_d: RoleProgram<RESULT_D>,
    pub e: RoleProgram<E>,
    pub e_first: RoleProgram<E_FIRST>,
    pub e_second: RoleProgram<E_SECOND>,
    pub result_e: RoleProgram<RESULT_E>,
    pub f: RoleProgram<F>,
    pub f_first: RoleProgram<F_FIRST>,
    pub f_second: RoleProgram<F_SECOND>,
    pub result_f: RoleProgram<RESULT_F>,
    pub g: RoleProgram<G>,
    pub g_first: RoleProgram<G_FIRST>,
    pub g_second: RoleProgram<G_SECOND>,
    pub result_g: RoleProgram<RESULT_G>,
}

pub fn project_1h() -> Programs1H {
    let a = developer::<A, A_FIRST, A_SECOND, RESULT_A, TEST_A>();
    let b = developer::<B, B_FIRST, B_SECOND, RESULT_B, TEST_B>();
    let c = developer::<C, C_FIRST, C_SECOND, RESULT_C, TEST_C>();
    let d = developer::<D, D_FIRST, D_SECOND, RESULT_D, TEST_D>();
    let e = developer::<E, E_FIRST, E_SECOND, RESULT_E, TEST_E>();
    let f = developer::<F, F_FIRST, F_SECOND, RESULT_F, TEST_F>();
    let g = developer::<G, G_FIRST, G_SECOND, RESULT_G, TEST_G>();
    let protocol = g::par(g::par(a, g::par(b, c)), g::par(g::par(d, e), g::par(f, g)));
    Programs1H {
        a: project(&protocol),
        a_first: project(&protocol),
        a_second: project(&protocol),
        result_a: project(&protocol),
        b: project(&protocol),
        b_first: project(&protocol),
        b_second: project(&protocol),
        result_b: project(&protocol),
        c: project(&protocol),
        c_first: project(&protocol),
        c_second: project(&protocol),
        result_c: project(&protocol),
        d: project(&protocol),
        d_first: project(&protocol),
        d_second: project(&protocol),
        result_d: project(&protocol),
        e: project(&protocol),
        e_first: project(&protocol),
        e_second: project(&protocol),
        result_e: project(&protocol),
        f: project(&protocol),
        f_first: project(&protocol),
        f_second: project(&protocol),
        result_f: project(&protocol),
        g: project(&protocol),
        g_first: project(&protocol),
        g_second: project(&protocol),
        result_g: project(&protocol),
    }
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
