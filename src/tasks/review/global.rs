//! 6E: independent features share a repository and one reviewer.
use hibana::{
    g,
    runtime::program::{RoleProgram, project},
};

pub const A: u8 = 0;
pub const B: u8 = 1;
pub const REVIEWER: u8 = 2;
pub const PORT_A: u8 = 3;
pub const PORT_B: u8 = 4;
pub const REVIEW_A: u16 = 60;
pub const REVIEW_B: u16 = 61;

pub type Implement = g::Msg<1, ()>;
pub type Merge = g::Msg<2, ()>;
pub type ReviewA = g::Msg<3, ()>;
pub type ReviewB = g::Msg<4, ()>;
pub type Acquire = g::Msg<10, ()>;
pub type Granted = g::Msg<11, ()>;
pub type Return = g::Msg<12, ()>;
pub type Returned = g::Msg<13, ()>;
pub type ReadyA = g::Msg<20, ()>;
pub type MergedA = g::Msg<21, ()>;
pub type ReadyB = g::Msg<22, ()>;
pub type MergedB = g::Msg<23, ()>;
pub type ApprovedA<'a> = g::Msg<30, &'a [u8]>;
pub type ReviseA<'a> = g::Msg<31, &'a [u8]>;
pub type ApprovedB<'a> = g::Msg<32, &'a [u8]>;
pub type ReviseB<'a> = g::Msg<33, &'a [u8]>;
pub type ApprovedPortA = g::Msg<40, ()>;
pub type RevisePortA = g::Msg<41, ()>;
pub type ApprovedPortB = g::Msg<42, ()>;
pub type RevisePortB = g::Msg<43, ()>;

pub struct Programs6E {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub reviewer: RoleProgram<REVIEWER>,
    pub port_a: RoleProgram<PORT_A>,
    pub port_b: RoleProgram<PORT_B>,
}

pub fn project_6e() -> Programs6E {
    let acquire_a = g::seq(
        g::send::<A, PORT_A, Acquire>(),
        g::send::<PORT_A, A, Granted>(),
    );
    let return_a = g::seq(
        g::send::<A, PORT_A, Return>(),
        g::send::<PORT_A, A, Returned>(),
    );
    let implement_a = g::seq(acquire_a, g::seq(g::send::<A, A, Implement>(), return_a));
    let review_a = g::seq(
        g::send::<A, REVIEWER, ReadyA>(),
        g::send::<REVIEWER, REVIEWER, ReviewA>(),
    );
    let approve_a = g::seq(
        g::par(
            g::send::<REVIEWER, A, ApprovedA<'static>>(),
            g::send::<REVIEWER, PORT_A, ApprovedPortA>(),
        ),
        g::seq(
            acquire_a,
            g::seq(
                g::send::<A, A, Merge>(),
                g::seq(return_a, g::send::<A, REVIEWER, MergedA>()),
            ),
        ),
    );
    let revise_a = g::seq(
        g::par(
            g::send::<REVIEWER, A, ReviseA<'static>>(),
            g::send::<REVIEWER, PORT_A, RevisePortA>(),
        ),
        g::seq(implement_a, review_a),
    );
    let a = g::seq(
        implement_a,
        g::seq(
            review_a,
            g::route(approve_a, revise_a).resolve::<REVIEW_A>().roll(),
        ),
    );

    let acquire_b = g::seq(
        g::send::<B, PORT_B, Acquire>(),
        g::send::<PORT_B, B, Granted>(),
    );
    let return_b = g::seq(
        g::send::<B, PORT_B, Return>(),
        g::send::<PORT_B, B, Returned>(),
    );
    let implement_b = g::seq(acquire_b, g::seq(g::send::<B, B, Implement>(), return_b));
    let review_b = g::seq(
        g::send::<B, REVIEWER, ReadyB>(),
        g::send::<REVIEWER, REVIEWER, ReviewB>(),
    );
    let approve_b = g::seq(
        g::par(
            g::send::<REVIEWER, B, ApprovedB<'static>>(),
            g::send::<REVIEWER, PORT_B, ApprovedPortB>(),
        ),
        g::seq(
            acquire_b,
            g::seq(
                g::send::<B, B, Merge>(),
                g::seq(return_b, g::send::<B, REVIEWER, MergedB>()),
            ),
        ),
    );
    let revise_b = g::seq(
        g::par(
            g::send::<REVIEWER, B, ReviseB<'static>>(),
            g::send::<REVIEWER, PORT_B, RevisePortB>(),
        ),
        g::seq(implement_b, review_b),
    );
    let b = g::seq(
        implement_b,
        g::seq(
            review_b,
            g::route(approve_b, revise_b).resolve::<REVIEW_B>().roll(),
        ),
    );
    let protocol = g::par(a, b);
    Programs6E {
        a: project(&protocol),
        b: project(&protocol),
        reviewer: project(&protocol),
        port_a: project(&protocol),
        port_b: project(&protocol),
    }
}
