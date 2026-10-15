//! Developer workflows and all resource handoffs. Domain calls are local steps.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub type Design = g::Msg<1, ()>;
pub type Implement = g::Msg<2, ()>;
pub type Commit = g::Msg<3, ()>;
pub type Test = g::Msg<4, ()>;
pub type Finished = g::Msg<5, ()>;
pub type AcquireFirst = g::Msg<10, ()>;
pub type GrantedFirst = g::Msg<11, ()>;
pub type ReturnFirst = g::Msg<12, ()>;
pub type ReturnedFirst = g::Msg<13, ()>;
pub type AcquireSecond = g::Msg<20, ()>;
pub type GrantedSecond = g::Msg<21, ()>;
pub type ReturnSecond = g::Msg<22, ()>;
pub type ReturnedSecond = g::Msg<23, ()>;
pub type ClosedFirst = g::Msg<30, ()>;
pub type ClosedSecond = g::Msg<31, ()>;
pub type Passed = g::Msg<150, ()>;
pub type Attempt = g::Msg<151, ()>;

// Every developer acquires modules in this common order. These are static
// resource addresses, not a runtime ownership table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Resource {
    Auth,
    Database,
    Api,
    Payment,
    Notify,
    Analytics,
    Search,
}
pub const MODULES_1H: [[Resource; 2]; 7] = [
    [Resource::Auth, Resource::Database],
    [Resource::Database, Resource::Api],
    [Resource::Api, Resource::Payment],
    [Resource::Payment, Resource::Notify],
    [Resource::Notify, Resource::Analytics],
    [Resource::Analytics, Resource::Search],
    [Resource::Auth, Resource::Search],
];
pub const MODULES_1E: [[Resource; 2]; 3] = [
    [Resource::Auth, Resource::Database],
    [Resource::Database, Resource::Api],
    [Resource::Auth, Resource::Api],
];
pub const MODULES_1M: [[Resource; 2]; 5] = [
    [Resource::Auth, Resource::Database],
    [Resource::Database, Resource::Api],
    [Resource::Api, Resource::Payment],
    [Resource::Payment, Resource::Notify],
    [Resource::Auth, Resource::Notify],
];
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
    let acquire = g::seq(
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

fn terminal_developer<const DEV: u8, const FIRST: u8, const SECOND: u8>() -> impl Projectable {
    let acquire = g::seq(
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
        g::send::<DEV, DEV, Design>(),
        g::seq(
            g::send::<DEV, DEV, Implement>(),
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
    pub a_first: RoleProgram<A_FIRST>,
    pub a_second: RoleProgram<A_SECOND>,
    pub b: RoleProgram<B>,
    pub b_first: RoleProgram<B_FIRST>,
    pub b_second: RoleProgram<B_SECOND>,
    pub c: RoleProgram<C>,
    pub c_first: RoleProgram<C_FIRST>,
    pub c_second: RoleProgram<C_SECOND>,
}

pub fn project_1e() -> Programs1E {
    let a = terminal_developer::<A, A_FIRST, A_SECOND>();
    let b = terminal_developer::<B, B_FIRST, B_SECOND>();
    let c = terminal_developer::<C, C_FIRST, C_SECOND>();
    let protocol = g::par(a, g::par(b, c));
    Programs1E {
        a: project(&protocol),
        a_first: project(&protocol),
        a_second: project(&protocol),
        b: project(&protocol),
        b_first: project(&protocol),
        b_second: project(&protocol),
        c: project(&protocol),
        c_first: project(&protocol),
        c_second: project(&protocol),
    }
}

pub struct Programs1M {
    pub a: RoleProgram<A>,
    pub a_first: RoleProgram<A_FIRST>,
    pub a_second: RoleProgram<A_SECOND>,
    pub b: RoleProgram<B>,
    pub b_first: RoleProgram<B_FIRST>,
    pub b_second: RoleProgram<B_SECOND>,
    pub c: RoleProgram<C>,
    pub c_first: RoleProgram<C_FIRST>,
    pub c_second: RoleProgram<C_SECOND>,
    pub d: RoleProgram<D>,
    pub d_first: RoleProgram<D_FIRST>,
    pub d_second: RoleProgram<D_SECOND>,
    pub e: RoleProgram<E>,
    pub e_first: RoleProgram<E_FIRST>,
    pub e_second: RoleProgram<E_SECOND>,
}

pub fn project_1m() -> Programs1M {
    let a = terminal_developer::<A, A_FIRST, A_SECOND>();
    let b = terminal_developer::<B, B_FIRST, B_SECOND>();
    let c = terminal_developer::<C, C_FIRST, C_SECOND>();
    let d = terminal_developer::<D, D_FIRST, D_SECOND>();
    let e = terminal_developer::<E, E_FIRST, E_SECOND>();
    let protocol = g::par(g::par(a, b), g::par(c, g::par(d, e)));
    Programs1M {
        a: project(&protocol),
        a_first: project(&protocol),
        a_second: project(&protocol),
        b: project(&protocol),
        b_first: project(&protocol),
        b_second: project(&protocol),
        c: project(&protocol),
        c_first: project(&protocol),
        c_second: project(&protocol),
        d: project(&protocol),
        d_first: project(&protocol),
        d_second: project(&protocol),
        e: project(&protocol),
        e_first: project(&protocol),
        e_second: project(&protocol),
    }
}
