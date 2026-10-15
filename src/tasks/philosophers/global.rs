use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const A: u8 = 0;
pub const B: u8 = 1;
pub const C: u8 = 2;
pub const FORK_A: u8 = 3;
pub const FORK_B: u8 = 4;
pub const FORK_C: u8 = 5;

pub mod roles9m {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const FORK_A: u8 = 5;
    pub const FORK_B: u8 = 6;
    pub const FORK_C: u8 = 7;
    pub const FORK_D: u8 = 8;
    pub const FORK_E: u8 = 9;
}

pub mod roles9h {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const F: u8 = 5;
    pub const G: u8 = 6;
    pub const FORK_A: u8 = 7;
    pub const FORK_B: u8 = 8;
    pub const FORK_C: u8 = 9;
    pub const FORK_D: u8 = 10;
    pub const FORK_E: u8 = 11;
    pub const FORK_F: u8 = 12;
    pub const FORK_G: u8 = 13;
}

pub type Think = g::Msg<1, ()>;
pub type Eat = g::Msg<2, ()>;
pub type Finished = g::Msg<3, ()>;

pub trait ForkMessages {
    type Available: g::Message<Payload = ()>;
    type Acquire: g::Message<Payload = ()>;
    type Granted: g::Message<Payload = ()>;
    type Return: g::Message<Payload = ()>;
    type Returned: g::Message<Payload = ()>;
}

pub struct ForkA;
pub struct ForkB;
pub struct ForkC;
pub struct ForkD;
pub struct ForkE;
pub struct ForkF;
pub struct ForkG;

impl ForkMessages for ForkA {
    type Available = g::Msg<10, ()>;
    type Acquire = g::Msg<11, ()>;
    type Granted = g::Msg<12, ()>;
    type Return = g::Msg<13, ()>;
    type Returned = g::Msg<14, ()>;
}
impl ForkMessages for ForkB {
    type Available = g::Msg<20, ()>;
    type Acquire = g::Msg<21, ()>;
    type Granted = g::Msg<22, ()>;
    type Return = g::Msg<23, ()>;
    type Returned = g::Msg<24, ()>;
}
impl ForkMessages for ForkC {
    type Available = g::Msg<30, ()>;
    type Acquire = g::Msg<31, ()>;
    type Granted = g::Msg<32, ()>;
    type Return = g::Msg<33, ()>;
    type Returned = g::Msg<34, ()>;
}
impl ForkMessages for ForkD {
    type Available = g::Msg<40, ()>;
    type Acquire = g::Msg<41, ()>;
    type Granted = g::Msg<42, ()>;
    type Return = g::Msg<43, ()>;
    type Returned = g::Msg<44, ()>;
}
impl ForkMessages for ForkE {
    type Available = g::Msg<50, ()>;
    type Acquire = g::Msg<51, ()>;
    type Granted = g::Msg<52, ()>;
    type Return = g::Msg<53, ()>;
    type Returned = g::Msg<54, ()>;
}
impl ForkMessages for ForkF {
    type Available = g::Msg<60, ()>;
    type Acquire = g::Msg<61, ()>;
    type Granted = g::Msg<62, ()>;
    type Return = g::Msg<63, ()>;
    type Returned = g::Msg<64, ()>;
}
impl ForkMessages for ForkG {
    type Available = g::Msg<70, ()>;
    type Acquire = g::Msg<71, ()>;
    type Granted = g::Msg<72, ()>;
    type Return = g::Msg<73, ()>;
    type Returned = g::Msg<74, ()>;
}

fn acquire<const AGENT: u8, const FORK: u8, M: ForkMessages>() -> impl Projectable {
    g::seq(
        g::send::<FORK, AGENT, M::Available>(),
        g::seq(
            g::send::<AGENT, FORK, M::Acquire>(),
            g::send::<FORK, AGENT, M::Granted>(),
        ),
    )
}

fn return_fork<const AGENT: u8, const FORK: u8, M: ForkMessages>() -> impl Projectable {
    g::seq(
        g::send::<AGENT, FORK, M::Return>(),
        g::send::<FORK, AGENT, M::Returned>(),
    )
}

fn eating<const AGENT: u8, const LEFT: u8, const RIGHT: u8, L: ForkMessages, R: ForkMessages>()
-> impl Projectable {
    g::seq(
        g::par(acquire::<AGENT, LEFT, L>(), acquire::<AGENT, RIGHT, R>()),
        g::seq(
            g::send::<AGENT, AGENT, Eat>(),
            g::seq(
                g::par(
                    return_fork::<AGENT, RIGHT, R>(),
                    return_fork::<AGENT, LEFT, L>(),
                ),
                g::send::<AGENT, AGENT, Finished>(),
            ),
        ),
    )
}

pub struct Programs9M {
    pub a: RoleProgram<{ roles9m::A }>,
    pub b: RoleProgram<{ roles9m::B }>,
    pub c: RoleProgram<{ roles9m::C }>,
    pub d: RoleProgram<{ roles9m::D }>,
    pub e: RoleProgram<{ roles9m::E }>,
    pub fork_a: RoleProgram<{ roles9m::FORK_A }>,
    pub fork_b: RoleProgram<{ roles9m::FORK_B }>,
    pub fork_c: RoleProgram<{ roles9m::FORK_C }>,
    pub fork_d: RoleProgram<{ roles9m::FORK_D }>,
    pub fork_e: RoleProgram<{ roles9m::FORK_E }>,
}

pub fn project_9m() -> Programs9M {
    use roles9m::*;
    let thinking = g::par(
        g::par(g::send::<A, A, Think>(), g::send::<B, B, Think>()),
        g::par(
            g::send::<C, C, Think>(),
            g::par(g::send::<D, D, Think>(), g::send::<E, E, Think>()),
        ),
    );
    let first = g::par(
        eating::<A, FORK_A, FORK_E, ForkA, ForkE>(),
        eating::<C, FORK_C, FORK_B, ForkC, ForkB>(),
    );
    let next = g::par(
        eating::<B, FORK_B, FORK_A, ForkB, ForkA>(),
        eating::<D, FORK_D, FORK_C, ForkD, ForkC>(),
    );
    let last = eating::<E, FORK_E, FORK_D, ForkE, ForkD>();
    // These compositions order each fork's two leases. Projection introduces
    // no all-role barrier: D can eat while A still holds its disjoint forks.
    let protocol = g::seq(thinking, g::seq(first, g::seq(next, last)));
    Programs9M {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        fork_a: project(&protocol),
        fork_b: project(&protocol),
        fork_c: project(&protocol),
        fork_d: project(&protocol),
        fork_e: project(&protocol),
    }
}

pub struct Programs9H {
    pub a: RoleProgram<{ roles9h::A }>,
    pub b: RoleProgram<{ roles9h::B }>,
    pub c: RoleProgram<{ roles9h::C }>,
    pub d: RoleProgram<{ roles9h::D }>,
    pub e: RoleProgram<{ roles9h::E }>,
    pub f: RoleProgram<{ roles9h::F }>,
    pub g: RoleProgram<{ roles9h::G }>,
    pub fork_a: RoleProgram<{ roles9h::FORK_A }>,
    pub fork_b: RoleProgram<{ roles9h::FORK_B }>,
    pub fork_c: RoleProgram<{ roles9h::FORK_C }>,
    pub fork_d: RoleProgram<{ roles9h::FORK_D }>,
    pub fork_e: RoleProgram<{ roles9h::FORK_E }>,
    pub fork_f: RoleProgram<{ roles9h::FORK_F }>,
    pub fork_g: RoleProgram<{ roles9h::FORK_G }>,
}

pub fn project_9h() -> Programs9H {
    use roles9h::*;
    let thinking = g::par(
        g::par(
            g::send::<A, A, Think>(),
            g::par(g::send::<B, B, Think>(), g::send::<C, C, Think>()),
        ),
        g::par(
            g::par(g::send::<D, D, Think>(), g::send::<E, E, Think>()),
            g::par(g::send::<F, F, Think>(), g::send::<G, G, Think>()),
        ),
    );
    let first = g::par(
        eating::<A, FORK_A, FORK_G, ForkA, ForkG>(),
        g::par(
            eating::<C, FORK_C, FORK_B, ForkC, ForkB>(),
            eating::<E, FORK_E, FORK_D, ForkE, ForkD>(),
        ),
    );
    let next = g::par(
        eating::<B, FORK_B, FORK_A, ForkB, ForkA>(),
        g::par(
            eating::<D, FORK_D, FORK_C, ForkD, ForkC>(),
            eating::<F, FORK_F, FORK_E, ForkF, ForkE>(),
        ),
    );
    let last = eating::<G, FORK_G, FORK_F, ForkG, ForkF>();
    let protocol = g::seq(thinking, g::seq(first, g::seq(next, last)));
    Programs9H {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        f: project(&protocol),
        g: project(&protocol),
        fork_a: project(&protocol),
        fork_b: project(&protocol),
        fork_c: project(&protocol),
        fork_d: project(&protocol),
        fork_e: project(&protocol),
        fork_f: project(&protocol),
        fork_g: project(&protocol),
    }
}

pub struct Programs9E {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub fork_a: RoleProgram<FORK_A>,
    pub fork_b: RoleProgram<FORK_B>,
    pub fork_c: RoleProgram<FORK_C>,
}

pub fn project_9e() -> Programs9E {
    // All three thinking operations are independent. Every pair of eaters
    // shares a fork in 9E, so their eating intervals cannot overlap.
    let thinking = g::par(
        g::send::<A, A, Think>(),
        g::par(g::send::<B, B, Think>(), g::send::<C, C, Think>()),
    );
    let a = eating::<A, FORK_A, FORK_C, ForkA, ForkC>();
    let b = eating::<B, FORK_B, FORK_A, ForkB, ForkA>();
    let c = eating::<C, FORK_C, FORK_B, ForkC, ForkB>();
    let protocol = g::seq(thinking, g::seq(a, g::seq(b, c)));
    Programs9E {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        fork_a: project(&protocol),
        fork_b: project(&protocol),
        fork_c: project(&protocol),
    }
}
