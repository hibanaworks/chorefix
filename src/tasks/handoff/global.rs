use hibana::{
    Endpoint, g,
    runtime::program::{RoleProgram, project},
};

pub const A: u8 = 0;
pub const B: u8 = 1;
pub const C: u8 = 2;
pub const STORE: u8 = 3;
pub const AUX: u8 = 4;
pub const S_OWNER: u8 = 5;
pub const R: u8 = 6;
pub const S: u8 = 7;
pub const VERIFY: u16 = 301;
pub const COMMIT_DECISION: u16 = 302;

pub type Prepare = g::Msg<1, [u8; 24]>;
pub type Prepared = g::Msg<2, [u8; 24]>;
pub type Inspect = g::Msg<3, [u8; 24]>;
pub type Inspected = g::Msg<4, [u8; 25]>;
pub type Revise = g::Msg<5, [u8; 24]>;
pub type Revised = g::Msg<6, [u8; 24]>;
pub type Reinspect = g::Msg<7, [u8; 24]>;
pub type Passed = g::Msg<8, [u8; 25]>;
pub type Failed = g::Msg<9, [u8; 25]>;
pub type Context = g::Msg<10, [u8; 25]>;
pub type LoanR<'r> = g::Msg<11, Endpoint<'r, R>>;
pub type LoanS<'r> = g::Msg<12, Endpoint<'r, S>>;
pub type Work = g::Msg<13, u64>;
pub type Worked = g::Msg<14, u64>;
pub type CommitR = g::Msg<15, [u8; 25]>;
pub type CommitS = g::Msg<16, [u8; 25]>;
pub type CommittedR = g::Msg<17, [u8; 24]>;
pub type CommittedS = g::Msg<18, [u8; 24]>;
pub type Report = g::Msg<22, [u8; 24]>;
pub type Audit = g::Msg<23, u64>;
pub type Audited = g::Msg<24, [u8; 24]>;
pub type CandidateReady = g::Msg<25, [u8; 25]>;
pub type CommitReady = g::Msg<26, [u8; 25]>;
pub type CommitDenied = g::Msg<27, [u8; 25]>;
pub type WorkReport = g::Msg<28, u64>;

pub struct Programs {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub store: RoleProgram<STORE>,
    pub aux: RoleProgram<AUX>,
    pub s_owner: RoleProgram<S_OWNER>,
    pub r: RoleProgram<R>,
    pub s: RoleProgram<S>,
}

pub fn programs() -> Programs {
    let accepted = g::seq(
        g::send::<STORE, R, CommitReady>(),
        g::seq(
            g::send::<STORE, S_OWNER, CommitReady>(),
            g::seq(
                g::send::<STORE, A, CommitReady>(),
                g::seq(
                    g::send::<S_OWNER, B, LoanS<'_>>(),
                    g::seq(
                        g::par(
                            g::send::<R, STORE, CommitR>(),
                            g::send::<S, STORE, CommitS>(),
                        ),
                        g::seq(
                            g::par(
                                g::send::<STORE, R, CommittedR>(),
                                g::send::<STORE, S, CommittedS>(),
                            ),
                            g::seq(
                                g::send::<B, S_OWNER, LoanS<'_>>(),
                                g::seq(
                                    g::send::<B, A, LoanR<'_>>(),
                                    g::seq(
                                        g::send::<B, A, Report>(),
                                        g::seq(
                                            g::send::<R, STORE, Audit>(),
                                            g::send::<STORE, R, Audited>(),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let rejected = g::seq(
        g::send::<STORE, R, CommitDenied>(),
        g::seq(
            g::send::<STORE, S_OWNER, CommitDenied>(),
            g::seq(
                g::send::<STORE, A, CommitDenied>(),
                g::send::<B, A, LoanR<'_>>(),
            ),
        ),
    );
    let revise = g::seq(
        g::send::<R, STORE, Revise>(),
        g::seq(
            g::send::<STORE, R, Revised>(),
            g::seq(
                g::send::<R, STORE, Reinspect>(),
                g::route(g::send::<STORE, R, Passed>(), g::send::<STORE, R, Failed>())
                    .resolve::<VERIFY>(),
            ),
        ),
    );
    let reconsider = g::route(revise, g::send::<R, STORE, CandidateReady>()).roll();
    let handoff = g::seq(
        g::send::<R, STORE, Prepare>(),
        g::seq(
            g::send::<STORE, R, Prepared>(),
            g::seq(
                g::send::<R, STORE, Inspect>(),
                g::seq(
                    g::send::<STORE, R, Inspected>(),
                    g::seq(
                        g::send::<A, B, Context>(),
                        g::seq(g::send::<A, B, LoanR<'_>>(), reconsider),
                    ),
                ),
            ),
        ),
    );
    let independent = g::seq(
        g::send::<S_OWNER, C, LoanS<'_>>(),
        g::seq(
            g::send::<S, AUX, Work>(),
            g::seq(
                g::send::<AUX, S, Worked>(),
                g::seq(
                    g::send::<C, S_OWNER, LoanS<'_>>(),
                    g::send::<C, STORE, WorkReport>(),
                ),
            ),
        ),
    );
    let protocol = g::seq(
        g::par(handoff, independent),
        g::route(accepted, rejected).resolve::<COMMIT_DECISION>(),
    );
    Programs {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        store: project(&protocol),
        aux: project(&protocol),
        s_owner: project(&protocol),
        r: project(&protocol),
        s: project(&protocol),
    }
}
