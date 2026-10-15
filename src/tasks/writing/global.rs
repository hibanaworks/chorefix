//! Literature and drafting are local. A shared write needs both section grants.
//! Review is terminal for all three original difficulty levels.
use crate::resource::global::*;
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub type Literature = g::Msg<1, ()>;
pub type Draft = g::Msg<2, ()>;
pub type Write = g::Msg<3, ()>;
pub type Review = g::Msg<4, ()>;
pub type Finished = g::Msg<5, ()>;
pub const A: u8 = 0;
pub const A_FIRST: u8 = 7;
pub const A_SECOND: u8 = 8;
pub const B: u8 = 1;
pub const B_FIRST: u8 = 9;
pub const B_SECOND: u8 = 10;
pub const C: u8 = 2;
pub const C_FIRST: u8 = 11;
pub const C_SECOND: u8 = 12;
pub const D: u8 = 3;
pub const D_FIRST: u8 = 13;
pub const D_SECOND: u8 = 14;
pub const E: u8 = 4;
pub const E_FIRST: u8 = 15;
pub const E_SECOND: u8 = 16;
pub const F: u8 = 5;
pub const F_FIRST: u8 = 17;
pub const F_SECOND: u8 = 18;
pub const G: u8 = 6;
pub const G_FIRST: u8 = 19;
pub const G_SECOND: u8 = 20;

fn researcher<const RESEARCHER: u8, const FIRST: u8, const SECOND: u8>() -> impl Projectable {
    let acquire = g::seq(
        g::seq(
            g::send::<RESEARCHER, FIRST, AcquireFirst>(),
            g::send::<FIRST, RESEARCHER, GrantedFirst>(),
        ),
        g::seq(
            g::send::<RESEARCHER, SECOND, AcquireSecond>(),
            g::send::<SECOND, RESEARCHER, GrantedSecond>(),
        ),
    );
    let release = g::par(
        g::seq(
            g::send::<RESEARCHER, SECOND, ReturnSecond>(),
            g::send::<SECOND, RESEARCHER, ReturnedSecond>(),
        ),
        g::seq(
            g::send::<RESEARCHER, FIRST, ReturnFirst>(),
            g::send::<FIRST, RESEARCHER, ReturnedFirst>(),
        ),
    );
    g::seq(
        g::send::<RESEARCHER, RESEARCHER, Literature>(),
        g::seq(
            g::send::<RESEARCHER, RESEARCHER, Draft>(),
            g::seq(
                acquire,
                g::seq(
                    g::send::<RESEARCHER, RESEARCHER, Write>(),
                    g::seq(
                        release,
                        g::seq(
                            g::send::<RESEARCHER, RESEARCHER, Review>(),
                            g::send::<RESEARCHER, RESEARCHER, Finished>(),
                        ),
                    ),
                ),
            ),
        ),
    )
}

pub struct Programs2E {
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

pub fn project_2e() -> Programs2E {
    let a = researcher::<A, A_FIRST, A_SECOND>();
    let b = researcher::<B, B_FIRST, B_SECOND>();
    let c = researcher::<C, C_FIRST, C_SECOND>();
    let protocol = g::par(a, g::par(b, c));
    Programs2E {
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

pub struct Programs2M {
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

pub fn project_2m() -> Programs2M {
    let a = researcher::<A, A_FIRST, A_SECOND>();
    let b = researcher::<B, B_FIRST, B_SECOND>();
    let c = researcher::<C, C_FIRST, C_SECOND>();
    let d = researcher::<D, D_FIRST, D_SECOND>();
    let e = researcher::<E, E_FIRST, E_SECOND>();
    let protocol = g::par(g::par(a, b), g::par(c, g::par(d, e)));
    Programs2M {
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

pub struct Programs2H {
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
    pub f: RoleProgram<F>,
    pub f_first: RoleProgram<F_FIRST>,
    pub f_second: RoleProgram<F_SECOND>,
    pub g: RoleProgram<G>,
    pub g_first: RoleProgram<G_FIRST>,
    pub g_second: RoleProgram<G_SECOND>,
}

pub fn project_2h() -> Programs2H {
    let a = researcher::<A, A_FIRST, A_SECOND>();
    let b = researcher::<B, B_FIRST, B_SECOND>();
    let c = researcher::<C, C_FIRST, C_SECOND>();
    let d = researcher::<D, D_FIRST, D_SECOND>();
    let e = researcher::<E, E_FIRST, E_SECOND>();
    let f = researcher::<F, F_FIRST, F_SECOND>();
    let g = researcher::<G, G_FIRST, G_SECOND>();
    let protocol = g::par(g::par(a, g::par(b, c)), g::par(g::par(d, e), g::par(f, g)));
    Programs2H {
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
        f: project(&protocol),
        f_first: project(&protocol),
        f_second: project(&protocol),
        g: project(&protocol),
        g_first: project(&protocol),
        g_second: project(&protocol),
    }
}
