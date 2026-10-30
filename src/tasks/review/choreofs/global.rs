//! Repository operations require the same delegated Endpoint throughout each
//! actual request, result and return. Review results choose the next work.
use super::imp::RepositoryArguments;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const A_WASI: u8 = 0;
pub const B_WASI: u8 = 1;
pub const REVIEW_WASI: u8 = 2;
pub const REVIEW_A: u8 = 5;
pub const REVIEW_B: u8 = 6;
pub const REPO: u8 = 7;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const REVIEW_IO: u8 = 10;
pub const REPO_ENDPOINT: u8 = 13;
pub const SELECT_REVIEW: u16 = 260;
pub const DECIDE_A: u16 = 264;
pub const DECIDE_B: u16 = 265;
pub const FEATURE_A: &str = "user_authentication";
pub const FEATURE_B: &str = "notification_feature";
pub const MERGE_BRANCH_A: &str = "dev_a_feature";
pub const MERGE_BRANCH_B: &str = "dev_b_feature";

pub type RepoLoan<'r> = g::Msg<10, Endpoint<'r, REPO_ENDPOINT>>;
pub type ImplementA = g::Msg<69, RepositoryArguments<1>>;
pub type ReviseA = g::Msg<76, RepositoryArguments<1>>;
pub type ImplementB = g::Msg<169, RepositoryArguments<1>>;
pub type ReviseB = g::Msg<176, RepositoryArguments<1>>;
pub type MergeA = g::Msg<70, RepositoryArguments<2>>;
pub type MergeB = g::Msg<170, RepositoryArguments<2>>;
pub type ImplementOnRepo = g::Msg<190, RepositoryArguments<1>>;
pub type MergeOnRepo = g::Msg<191, RepositoryArguments<2>>;
pub type RepoResult<'a> = g::Msg<192, &'a [u8]>;
pub type ReadyA<'a> = g::Msg<90, &'a [u8]>;
pub type ReadyB<'a> = g::Msg<91, &'a [u8]>;
pub type ReviewA = g::Msg<71, RepositoryArguments<3>>;
pub type ReReviewA = g::Msg<77, RepositoryArguments<3>>;
pub type ReviewB = g::Msg<171, RepositoryArguments<3>>;
pub type ReReviewB = g::Msg<177, RepositoryArguments<3>>;
pub type ApprovedA<'a> = g::Msg<30, &'a [u8]>;
pub type RejectedA<'a> = g::Msg<31, &'a [u8]>;
pub type ApprovedB<'a> = g::Msg<40, &'a [u8]>;
pub type RejectedB<'a> = g::Msg<41, &'a [u8]>;

// These messages carry the review's actual arguments. Distinct labels identify
// its projected occurrence; filesystem operations do not choose a task phase.
pub type FirstA = g::Msg<80, RepositoryArguments<3>>;
pub type FirstB = g::Msg<81, RepositoryArguments<3>>;
pub type AgainABeforeB = g::Msg<82, RepositoryArguments<3>>;
pub type BDuringARevision = g::Msg<83, RepositoryArguments<3>>;
pub type BAfterAMerge = g::Msg<84, RepositoryArguments<3>>;
pub type AgainAAfterB = g::Msg<85, RepositoryArguments<3>>;
pub type AgainBBeforeA = g::Msg<86, RepositoryArguments<3>>;
pub type ADuringBRevision = g::Msg<87, RepositoryArguments<3>>;
pub type AAfterBMerge = g::Msg<88, RepositoryArguments<3>>;
pub type AgainBAfterA = g::Msg<89, RepositoryArguments<3>>;
pub type AgainADuringBRevision = g::Msg<92, RepositoryArguments<3>>;
pub type AgainBDuringARevision = g::Msg<93, RepositoryArguments<3>>;
pub type AgainBBeforeAReview = g::Msg<94, RepositoryArguments<3>>;
pub type AgainABeforeBReview = g::Msg<95, RepositoryArguments<3>>;
pub type ExitA = g::Msg<230, i32>;
pub type ExitB = g::Msg<231, i32>;
pub type ReviewExitA = g::Msg<232, i32>;
pub type ReviewExitB = g::Msg<233, i32>;
pub type RepoExit = g::Msg<234, i32>;

fn intent<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_WASI, REPO, g::Msg<LABEL, RepositoryArguments<3>>>(),
        g::seq(
            g::send::<REVIEW_WASI, REVIEW_A, g::Msg<LABEL, RepositoryArguments<3>>>(),
            g::seq(
                g::send::<REVIEW_WASI, REVIEW_B, g::Msg<LABEL, RepositoryArguments<3>>>(),
                g::seq(
                    g::send::<REVIEW_WASI, A_WASI, g::Msg<LABEL, RepositoryArguments<3>>>(),
                    g::send::<REVIEW_WASI, B_WASI, g::Msg<LABEL, RepositoryArguments<3>>>(),
                ),
            ),
        ),
    )
}

fn implement_a<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<A_WASI, REPO, g::Msg<LABEL, RepositoryArguments<1>>>(),
        g::seq(
            g::send::<REPO, A_WASI, RepoLoan<'_>>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, ImplementOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult<'static>>(),
                    g::seq(
                        g::send::<A_WASI, REPO, RepoLoan<'_>>(),
                        g::send::<A_WASI, REVIEW_WASI, ReadyA<'static>>(),
                    ),
                ),
            ),
        ),
    )
}
fn implement_b<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<B_WASI, REPO, g::Msg<LABEL, RepositoryArguments<1>>>(),
        g::seq(
            g::send::<REPO, B_WASI, RepoLoan<'_>>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, ImplementOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult<'static>>(),
                    g::seq(
                        g::send::<B_WASI, REPO, RepoLoan<'_>>(),
                        g::send::<B_WASI, REVIEW_WASI, ReadyB<'static>>(),
                    ),
                ),
            ),
        ),
    )
}
fn merge_a() -> impl Projectable {
    g::seq(
        g::send::<A_WASI, REPO, MergeA>(),
        g::seq(
            g::send::<REPO, A_WASI, RepoLoan<'_>>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, MergeOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult<'static>>(),
                    g::send::<A_WASI, REPO, RepoLoan<'_>>(),
                ),
            ),
        ),
    )
}
fn merge_b() -> impl Projectable {
    g::seq(
        g::send::<B_WASI, REPO, MergeB>(),
        g::seq(
            g::send::<REPO, B_WASI, RepoLoan<'_>>(),
            g::seq(
                g::send::<REPO_ENDPOINT, REPO, MergeOnRepo>(),
                g::seq(
                    g::send::<REPO, REPO_ENDPOINT, RepoResult<'static>>(),
                    g::send::<B_WASI, REPO, RepoLoan<'_>>(),
                ),
            ),
        ),
    )
}
fn outcome_a<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, A_WASI, g::Msg<LABEL, &'static [u8]>>(),
        g::seq(
            g::send::<REVIEW_A, B_WASI, g::Msg<LABEL, &'static [u8]>>(),
            g::seq(
                g::send::<REVIEW_A, REVIEW_WASI, g::Msg<LABEL, &'static [u8]>>(),
                g::seq(
                    g::send::<REVIEW_A, REVIEW_B, g::Msg<LABEL, &'static [u8]>>(),
                    g::send::<REVIEW_A, REPO, g::Msg<LABEL, &'static [u8]>>(),
                ),
            ),
        ),
    )
}
fn outcome_b<const LABEL: u8>() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, A_WASI, g::Msg<LABEL, &'static [u8]>>(),
        g::seq(
            g::send::<REVIEW_B, B_WASI, g::Msg<LABEL, &'static [u8]>>(),
            g::seq(
                g::send::<REVIEW_B, REVIEW_WASI, g::Msg<LABEL, &'static [u8]>>(),
                g::seq(
                    g::send::<REVIEW_B, REVIEW_A, g::Msg<LABEL, &'static [u8]>>(),
                    g::send::<REVIEW_B, REPO, g::Msg<LABEL, &'static [u8]>>(),
                ),
            ),
        ),
    )
}
fn finish() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<A_WASI, REVIEW_WASI, ExitA>(),
            g::send::<B_WASI, REVIEW_WASI, ExitB>(),
        ),
        g::seq(
            g::par(
                g::send::<REVIEW_WASI, REVIEW_A, ReviewExitA>(),
                g::send::<REVIEW_WASI, REVIEW_B, ReviewExitB>(),
            ),
            g::send::<REVIEW_WASI, REPO, RepoExit>(),
        ),
    )
}

fn repeat_b_last() -> impl Projectable {
    g::seq(
        implement_b::<176>(),
        g::seq(
            intent::<89>(),
            g::seq(
                g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
                g::route(
                    g::seq(outcome_b::<40>(), g::seq(merge_b(), finish())),
                    outcome_b::<41>(),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
    .roll()
}
fn repeat_a_last() -> impl Projectable {
    g::seq(
        implement_a::<76>(),
        g::seq(
            intent::<85>(),
            g::seq(
                g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
                g::route(
                    g::seq(outcome_a::<30>(), g::seq(merge_a(), finish())),
                    outcome_a::<31>(),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
    )
    .roll()
}

fn merge_a_and_review_b(review: impl Projectable) -> impl Projectable {
    g::seq(
        g::par(merge_a(), review),
        g::route(
            g::seq(outcome_b::<40>(), g::seq(merge_b(), finish())),
            g::seq(outcome_b::<41>(), repeat_b_last()),
        )
        .resolve::<DECIDE_B>(),
    )
}
fn merge_b_and_review_a(review: impl Projectable) -> impl Projectable {
    g::seq(
        g::par(merge_b(), review),
        g::route(
            g::seq(outcome_a::<30>(), g::seq(merge_a(), finish())),
            g::seq(outcome_a::<31>(), repeat_a_last()),
        )
        .resolve::<DECIDE_A>(),
    )
}

fn review_a_until_approved(approved: impl Projectable) -> impl Projectable {
    g::seq(
        intent::<85>(),
        g::seq(
            g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
            g::route(
                g::seq(outcome_a::<30>(), approved),
                g::seq(outcome_a::<31>(), implement_a::<76>()),
            )
            .resolve::<DECIDE_A>(),
        ),
    )
    .roll()
}
fn review_b_until_approved(approved: impl Projectable) -> impl Projectable {
    g::seq(
        intent::<89>(),
        g::seq(
            g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
            g::route(
                g::seq(outcome_b::<40>(), approved),
                g::seq(outcome_b::<41>(), implement_b::<176>()),
            )
            .resolve::<DECIDE_B>(),
        ),
    )
    .roll()
}

fn both_rejected_a_ready() -> impl Projectable {
    g::route(
        g::seq(
            intent::<92>(),
            g::seq(
                g::par(
                    implement_b::<176>(),
                    g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
                ),
                g::route(
                    g::seq(
                        outcome_a::<30>(),
                        g::seq(
                            intent::<89>(),
                            merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        ),
                    ),
                    g::seq(
                        outcome_a::<31>(),
                        review_a_until_approved(g::seq(
                            intent::<89>(),
                            merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        )),
                    ),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
        g::seq(
            intent::<94>(),
            g::seq(
                implement_b::<176>(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
                    g::route(
                        g::seq(
                            outcome_b::<40>(),
                            g::seq(
                                intent::<85>(),
                                merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                            ),
                        ),
                        g::seq(
                            outcome_b::<41>(),
                            review_b_until_approved(g::seq(
                                intent::<85>(),
                                merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                            )),
                        ),
                    )
                    .resolve::<DECIDE_B>(),
                ),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
}
fn both_rejected_b_ready() -> impl Projectable {
    g::route(
        g::seq(
            intent::<95>(),
            g::seq(
                implement_a::<76>(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
                    g::route(
                        g::seq(
                            outcome_a::<30>(),
                            g::seq(
                                intent::<89>(),
                                merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                            ),
                        ),
                        g::seq(
                            outcome_a::<31>(),
                            review_a_until_approved(g::seq(
                                intent::<89>(),
                                merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                            )),
                        ),
                    )
                    .resolve::<DECIDE_A>(),
                ),
            ),
        ),
        g::seq(
            intent::<93>(),
            g::seq(
                g::par(
                    implement_a::<76>(),
                    g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
                ),
                g::route(
                    g::seq(
                        outcome_b::<40>(),
                        g::seq(
                            intent::<85>(),
                            merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        ),
                    ),
                    g::seq(
                        outcome_b::<41>(),
                        review_b_until_approved(g::seq(
                            intent::<85>(),
                            merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        )),
                    ),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
}

fn repeat_a_before_b() -> impl Projectable {
    g::route(
        g::seq(
            intent::<82>(),
            g::seq(
                implement_a::<76>(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>(),
                    g::route(
                        g::seq(
                            outcome_a::<30>(),
                            g::seq(
                                intent::<84>(),
                                merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReviewB>()),
                            ),
                        ),
                        outcome_a::<31>(),
                    )
                    .resolve::<DECIDE_A>(),
                ),
            ),
        ),
        g::seq(
            intent::<83>(),
            g::seq(
                g::par(
                    implement_a::<76>(),
                    g::send::<REVIEW_WASI, REVIEW_B, ReviewB>(),
                ),
                g::route(
                    g::seq(
                        outcome_b::<40>(),
                        g::seq(
                            intent::<85>(),
                            merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReReviewA>()),
                        ),
                    ),
                    g::seq(outcome_b::<41>(), both_rejected_a_ready()),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
    .roll()
}
fn repeat_b_before_a() -> impl Projectable {
    g::route(
        g::seq(
            intent::<87>(),
            g::seq(
                g::par(
                    implement_b::<176>(),
                    g::send::<REVIEW_WASI, REVIEW_A, ReviewA>(),
                ),
                g::route(
                    g::seq(
                        outcome_a::<30>(),
                        g::seq(
                            intent::<89>(),
                            merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>()),
                        ),
                    ),
                    g::seq(outcome_a::<31>(), both_rejected_b_ready()),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
        g::seq(
            intent::<86>(),
            g::seq(
                implement_b::<176>(),
                g::seq(
                    g::send::<REVIEW_WASI, REVIEW_B, ReReviewB>(),
                    g::route(
                        g::seq(
                            outcome_b::<40>(),
                            g::seq(
                                intent::<88>(),
                                merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReviewA>()),
                            ),
                        ),
                        outcome_b::<41>(),
                    )
                    .resolve::<DECIDE_B>(),
                ),
            ),
        ),
    )
    .resolve::<SELECT_REVIEW>()
    .roll()
}
fn first_a() -> impl Projectable {
    g::seq(
        intent::<80>(),
        g::seq(
            implement_a::<69>(),
            g::seq(
                g::par(
                    g::send::<REVIEW_WASI, REVIEW_A, ReviewA>(),
                    implement_b::<169>(),
                ),
                g::route(
                    g::seq(
                        outcome_a::<30>(),
                        g::seq(
                            intent::<84>(),
                            merge_a_and_review_b(g::send::<REVIEW_WASI, REVIEW_B, ReviewB>()),
                        ),
                    ),
                    g::seq(outcome_a::<31>(), repeat_a_before_b()),
                )
                .resolve::<DECIDE_A>(),
            ),
        ),
    )
}
fn first_b() -> impl Projectable {
    g::seq(
        intent::<81>(),
        g::seq(
            implement_b::<169>(),
            g::seq(
                g::par(
                    g::send::<REVIEW_WASI, REVIEW_B, ReviewB>(),
                    implement_a::<69>(),
                ),
                g::route(
                    g::seq(
                        outcome_b::<40>(),
                        g::seq(
                            intent::<88>(),
                            merge_b_and_review_a(g::send::<REVIEW_WASI, REVIEW_A, ReviewA>()),
                        ),
                    ),
                    g::seq(outcome_b::<41>(), repeat_b_before_a()),
                )
                .resolve::<DECIDE_B>(),
            ),
        ),
    )
}

pub struct Programs {
    pub a_wasi: RoleProgram<A_WASI>,
    pub b_wasi: RoleProgram<B_WASI>,
    pub review_wasi: RoleProgram<REVIEW_WASI>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub repo: RoleProgram<REPO>,
    pub repo_endpoint: RoleProgram<REPO_ENDPOINT>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
}

pub fn programs() -> Programs {
    let task = g::route(first_a(), first_b()).resolve::<SELECT_REVIEW>();
    let private = g::par(
        crate::shell::global::private_io::<A_WASI, A_IO>(),
        g::par(
            crate::shell::global::private_io::<B_WASI, B_IO>(),
            crate::shell::global::private_io::<REVIEW_WASI, REVIEW_IO>(),
        ),
    );
    let protocol = g::par(task, private);
    Programs {
        a_wasi: project(&protocol),
        b_wasi: project(&protocol),
        review_wasi: project(&protocol),
        review_a: project(&protocol),
        review_b: project(&protocol),
        repo: project(&protocol),
        repo_endpoint: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        review_io: project(&protocol),
    }
}
