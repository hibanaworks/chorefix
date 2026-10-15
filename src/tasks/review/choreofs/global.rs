//! The WASI roles, feature owners, reviewer services and sole REPO owner
//! communicate through this one choreography.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::protocol;

pub const A_WASI: u8 = 0;
pub const B_WASI: u8 = 1;
pub const REVIEW_WASI: u8 = 2;
pub const A: u8 = 3;
pub const B: u8 = 4;
pub const REVIEW_A: u8 = 5;
pub const REVIEW_B: u8 = 6;
pub const REPO: u8 = 7;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const REVIEW_IO: u8 = 10;
// One guest owns independent projected service roles. An offer on one
// service must not observe the other service's still-live rejection loop.
pub const REVIEW_A_WASI: u8 = 11;
pub const REVIEW_B_WASI: u8 = 12;
// Domain identities for the two projected feature branches.
pub const FEATURE_A: &str = "user_authentication";
pub const FEATURE_B: &str = "notification_feature";
pub const MERGE_BRANCH_A: &str = "dev_a_feature";
pub const MERGE_BRANCH_B: &str = "dev_b_feature";
pub const SERVICE_FD: u8 = 4;
pub const SERVICE_PREFIX: &[u8] = b"service/";
pub const RESULT_PATH: &[u8] = b"service/result";
pub const REVIEW_B_FD: u8 = 16;
pub const CHECK_A: u16 = 260;
pub const CHECK_B: u16 = 261;
pub const CHECK_REVIEW_A: u16 = 262;
pub const CHECK_REVIEW_B: u16 = 263;
pub const DECIDE_A: u16 = 264;
pub const DECIDE_B: u16 = 265;

pub type Open = g::Msg<3, protocol::PathOpenReq>;
pub type Opened = g::Msg<4, protocol::PathOpenedRet>;
pub type Write = g::Msg<5, protocol::FdWriteReq>;
pub type Written = g::Msg<6, protocol::FdWriteDoneRet>;
pub type Commit = g::Msg<7, protocol::FdCloseReq>;
pub type Admitted = g::Msg<8, ()>;
pub type Rejected<'a> = g::Msg<9, &'a [u8]>;
pub type Validate = g::Msg<12, ()>;
pub type Result<'a> = g::Msg<24, &'a [u8]>;
pub type ServiceDone = g::Msg<25, ()>;
pub type Unavailable = g::Msg<26, protocol::FdClosedRet>;
pub type OpenB = g::Msg<43, protocol::PathOpenReq>;
pub type OpenedB = g::Msg<44, protocol::PathOpenedRet>;
pub type WriteB = g::Msg<45, protocol::FdWriteReq>;
pub type WrittenB = g::Msg<46, protocol::FdWriteDoneRet>;
pub type CommitB = g::Msg<47, protocol::FdCloseReq>;
pub type AdmittedB = g::Msg<48, ()>;
pub type RejectedB<'a> = g::Msg<49, &'a [u8]>;
pub type ValidateB = g::Msg<52, ()>;
pub type ResultB<'a> = g::Msg<64, &'a [u8]>;
pub type ServiceDoneB = g::Msg<65, ()>;
pub type UnavailableB = g::Msg<66, protocol::FdClosedRet>;
pub type Exit = g::Msg<241, i32>;
pub type ExitB = g::Msg<242, ()>;
pub type Ready = g::Msg<20, ()>;
pub type BeginFeature = g::Msg<67, ()>;
pub type BeginReviewVisit = g::Msg<68, ()>;
pub type ImplementRequest = g::Msg<69, ()>;
pub type MergeRequest = g::Msg<70, ()>;
pub type ReviewRequest = g::Msg<71, ()>;
pub type RejectRequest = g::Msg<72, ()>;
pub type RequestReady = g::Msg<73, ()>;
pub type ReviewRequestReady = g::Msg<74, ()>;
pub type ReviewRequestReadyB = g::Msg<75, ()>;
pub type ReviewRequested = g::Msg<38, ()>;
pub type ReviewRequestedB = g::Msg<39, ()>;
pub type Approved<'a> = g::Msg<30, &'a [u8]>;
pub type Revise<'a> = g::Msg<31, &'a [u8]>;
pub type Review = g::Msg<32, ()>;
pub type Merged = g::Msg<33, ()>;
pub type BothMerged = g::Msg<34, ()>;
pub type ReviewerClosed = g::Msg<37, ()>;
pub type ReviewExitAck = g::Msg<243, ()>;
pub type ReviewExitAckB = g::Msg<244, ()>;
pub type RepoExit = g::Msg<35, ()>;
pub type RepoFailed = g::Msg<36, i32>;
pub type ApproveRepoA = g::Msg<180, ()>;
pub type ReworkRepoA = g::Msg<181, ()>;
pub type ApproveRepoB = g::Msg<182, ()>;
pub type ReworkRepoB = g::Msg<183, ()>;
pub type DecisionRecordedA = g::Msg<184, ()>;
pub type DecisionRecordedB = g::Msg<185, ()>;
pub const CHECK_MERGE_A: u16 = 266;
pub const CHECK_MERGE_B: u16 = 267;

pub type OpenImplementA = g::Msg<110, protocol::PathOpenReq>;
pub type OpenedImplementA = g::Msg<111, protocol::PathOpenedRet>;
pub type WriteImplementA = g::Msg<112, protocol::FdWriteReq>;
pub type WrittenImplementA = g::Msg<113, protocol::FdWriteDoneRet>;
pub type CommitImplementA = g::Msg<114, protocol::FdCloseReq>;
pub type MaterializeImplementA = g::Msg<115, ()>;
pub type CompletedImplementA<'a> = g::Msg<116, &'a [u8]>;
pub type RejectedImplementA<'a> = g::Msg<117, &'a [u8]>;
fn implement_a() -> impl Projectable {
    g::seq(
        g::send::<A, A, ImplementRequest>(),
        g::seq(
            g::send::<A, A_WASI, RequestReady>(),
            g::route(
                g::seq(
                    g::send::<A_WASI, A, Open>(),
                    g::seq(
                        g::send::<A, REPO, OpenImplementA>(),
                        g::seq(
                            g::send::<REPO, A, OpenedImplementA>(),
                            g::send::<A, A_WASI, Opened>(),
                        ),
                    ),
                ),
                g::route(
                    g::seq(
                        g::send::<A_WASI, A, Write>(),
                        g::seq(
                            g::send::<A, REPO, WriteImplementA>(),
                            g::seq(
                                g::send::<REPO, A, WrittenImplementA>(),
                                g::send::<A, A_WASI, Written>(),
                            ),
                        ),
                    ),
                    g::seq(
                        g::send::<A_WASI, A, Commit>(),
                        g::seq(
                            g::send::<A, REPO, CommitImplementA>(),
                            g::route(
                                g::seq(
                                    g::send::<REPO, REPO, MaterializeImplementA>(),
                                    g::seq(
                                        g::send::<REPO, A, CompletedImplementA<'static>>(),
                                        g::send::<A, A_WASI, Admitted>(),
                                    ),
                                ),
                                g::seq(
                                    g::send::<REPO, A, RejectedImplementA<'static>>(),
                                    g::send::<A, A_WASI, Rejected<'static>>(),
                                ),
                            )
                            .resolve::<CHECK_A>(),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
pub type OpenMergeA = g::Msg<130, protocol::PathOpenReq>;
pub type OpenedMergeA = g::Msg<131, protocol::PathOpenedRet>;
pub type WriteMergeA = g::Msg<132, protocol::FdWriteReq>;
pub type WrittenMergeA = g::Msg<133, protocol::FdWriteDoneRet>;
pub type CommitMergeA = g::Msg<134, protocol::FdCloseReq>;
pub type MaterializeMergeA = g::Msg<135, ()>;
pub type CompletedMergeA<'a> = g::Msg<136, &'a [u8]>;
pub type RejectedMergeA<'a> = g::Msg<137, &'a [u8]>;
fn merge_a() -> impl Projectable {
    g::seq(
        g::send::<A, A, MergeRequest>(),
        g::seq(
            g::send::<A, A_WASI, RequestReady>(),
            g::route(
                g::seq(
                    g::send::<A_WASI, A, Open>(),
                    g::seq(
                        g::send::<A, REPO, OpenMergeA>(),
                        g::seq(
                            g::send::<REPO, A, OpenedMergeA>(),
                            g::send::<A, A_WASI, Opened>(),
                        ),
                    ),
                ),
                g::route(
                    g::seq(
                        g::send::<A_WASI, A, Write>(),
                        g::seq(
                            g::send::<A, REPO, WriteMergeA>(),
                            g::seq(
                                g::send::<REPO, A, WrittenMergeA>(),
                                g::send::<A, A_WASI, Written>(),
                            ),
                        ),
                    ),
                    g::seq(
                        g::send::<A_WASI, A, Commit>(),
                        g::seq(
                            g::send::<A, REPO, CommitMergeA>(),
                            g::route(
                                g::seq(
                                    g::send::<REPO, REPO, MaterializeMergeA>(),
                                    g::seq(
                                        g::send::<REPO, A, CompletedMergeA<'static>>(),
                                        g::send::<A, A_WASI, Admitted>(),
                                    ),
                                ),
                                g::seq(
                                    g::send::<REPO, A, RejectedMergeA<'static>>(),
                                    g::send::<A, A_WASI, Rejected<'static>>(),
                                ),
                            )
                            .resolve::<CHECK_MERGE_A>(),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
pub type OpenImplementB = g::Msg<150, protocol::PathOpenReq>;
pub type OpenedImplementB = g::Msg<151, protocol::PathOpenedRet>;
pub type WriteImplementB = g::Msg<152, protocol::FdWriteReq>;
pub type WrittenImplementB = g::Msg<153, protocol::FdWriteDoneRet>;
pub type CommitImplementB = g::Msg<154, protocol::FdCloseReq>;
pub type MaterializeImplementB = g::Msg<155, ()>;
pub type CompletedImplementB<'a> = g::Msg<156, &'a [u8]>;
pub type RejectedImplementB<'a> = g::Msg<157, &'a [u8]>;
fn implement_b() -> impl Projectable {
    g::seq(
        g::send::<B, B, ImplementRequest>(),
        g::seq(
            g::send::<B, B_WASI, RequestReady>(),
            g::route(
                g::seq(
                    g::send::<B_WASI, B, Open>(),
                    g::seq(
                        g::send::<B, REPO, OpenImplementB>(),
                        g::seq(
                            g::send::<REPO, B, OpenedImplementB>(),
                            g::send::<B, B_WASI, Opened>(),
                        ),
                    ),
                ),
                g::route(
                    g::seq(
                        g::send::<B_WASI, B, Write>(),
                        g::seq(
                            g::send::<B, REPO, WriteImplementB>(),
                            g::seq(
                                g::send::<REPO, B, WrittenImplementB>(),
                                g::send::<B, B_WASI, Written>(),
                            ),
                        ),
                    ),
                    g::seq(
                        g::send::<B_WASI, B, Commit>(),
                        g::seq(
                            g::send::<B, REPO, CommitImplementB>(),
                            g::route(
                                g::seq(
                                    g::send::<REPO, REPO, MaterializeImplementB>(),
                                    g::seq(
                                        g::send::<REPO, B, CompletedImplementB<'static>>(),
                                        g::send::<B, B_WASI, Admitted>(),
                                    ),
                                ),
                                g::seq(
                                    g::send::<REPO, B, RejectedImplementB<'static>>(),
                                    g::send::<B, B_WASI, Rejected<'static>>(),
                                ),
                            )
                            .resolve::<CHECK_B>(),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
pub type OpenMergeB = g::Msg<170, protocol::PathOpenReq>;
pub type OpenedMergeB = g::Msg<171, protocol::PathOpenedRet>;
pub type WriteMergeB = g::Msg<172, protocol::FdWriteReq>;
pub type WrittenMergeB = g::Msg<173, protocol::FdWriteDoneRet>;
pub type CommitMergeB = g::Msg<174, protocol::FdCloseReq>;
pub type MaterializeMergeB = g::Msg<175, ()>;
pub type CompletedMergeB<'a> = g::Msg<176, &'a [u8]>;
pub type RejectedMergeB<'a> = g::Msg<177, &'a [u8]>;
fn merge_b() -> impl Projectable {
    g::seq(
        g::send::<B, B, MergeRequest>(),
        g::seq(
            g::send::<B, B_WASI, RequestReady>(),
            g::route(
                g::seq(
                    g::send::<B_WASI, B, Open>(),
                    g::seq(
                        g::send::<B, REPO, OpenMergeB>(),
                        g::seq(
                            g::send::<REPO, B, OpenedMergeB>(),
                            g::send::<B, B_WASI, Opened>(),
                        ),
                    ),
                ),
                g::route(
                    g::seq(
                        g::send::<B_WASI, B, Write>(),
                        g::seq(
                            g::send::<B, REPO, WriteMergeB>(),
                            g::seq(
                                g::send::<REPO, B, WrittenMergeB>(),
                                g::send::<B, B_WASI, Written>(),
                            ),
                        ),
                    ),
                    g::seq(
                        g::send::<B_WASI, B, Commit>(),
                        g::seq(
                            g::send::<B, REPO, CommitMergeB>(),
                            g::route(
                                g::seq(
                                    g::send::<REPO, REPO, MaterializeMergeB>(),
                                    g::seq(
                                        g::send::<REPO, B, CompletedMergeB<'static>>(),
                                        g::send::<B, B_WASI, Admitted>(),
                                    ),
                                ),
                                g::seq(
                                    g::send::<REPO, B, RejectedMergeB<'static>>(),
                                    g::send::<B, B_WASI, Rejected<'static>>(),
                                ),
                            )
                            .resolve::<CHECK_MERGE_B>(),
                        ),
                    ),
                ),
            ),
        ),
    )
    .roll()
}

pub struct Programs {
    pub a_wasi: RoleProgram<A_WASI>,
    pub b_wasi: RoleProgram<B_WASI>,
    pub review_wasi: RoleProgram<REVIEW_WASI>,
    pub review_a_wasi: RoleProgram<REVIEW_A_WASI>,
    pub review_b_wasi: RoleProgram<REVIEW_B_WASI>,
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub review_a: RoleProgram<REVIEW_A>,
    pub review_b: RoleProgram<REVIEW_B>,
    pub repo: RoleProgram<REPO>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub review_io: RoleProgram<REVIEW_IO>,
}

// This fragment only admits a close. Materialization and its reply follow it
// explicitly in the task protocol; invalid argument data can be submitted again.
fn requests<const VM: u8, const OWNER: u8, const CHECK: u16>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, OWNER, ReviewRequest>(),
        g::seq(
            g::send::<OWNER, VM, ReviewRequestReady>(),
            g::route(
                g::seq(g::send::<VM, OWNER, Open>(), g::send::<OWNER, VM, Opened>()),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, Write>(),
                        g::send::<OWNER, VM, Written>(),
                    ),
                    g::seq(
                        g::send::<VM, OWNER, Commit>(),
                        g::route(
                            g::seq(
                                g::send::<OWNER, OWNER, Validate>(),
                                g::send::<OWNER, VM, Admitted>(),
                            ),
                            g::send::<OWNER, VM, Rejected<'static>>(),
                        )
                        .resolve::<CHECK>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn review_b_requests() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, REVIEW_B, ReviewRequest>(),
        g::seq(
            g::send::<REVIEW_B, REVIEW_B_WASI, ReviewRequestReadyB>(),
            g::route(
                g::seq(
                    g::send::<REVIEW_B_WASI, REVIEW_B, OpenB>(),
                    g::send::<REVIEW_B, REVIEW_B_WASI, OpenedB>(),
                ),
                g::route(
                    g::seq(
                        g::send::<REVIEW_B_WASI, REVIEW_B, WriteB>(),
                        g::send::<REVIEW_B, REVIEW_B_WASI, WrittenB>(),
                    ),
                    g::seq(
                        g::send::<REVIEW_B_WASI, REVIEW_B, CommitB>(),
                        g::route(
                            g::seq(
                                g::send::<REVIEW_B, REVIEW_B, ValidateB>(),
                                g::send::<REVIEW_B, REVIEW_B_WASI, AdmittedB>(),
                            ),
                            g::send::<REVIEW_B, REVIEW_B_WASI, RejectedB<'static>>(),
                        )
                        .resolve::<CHECK_REVIEW_B>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn closed<const VM: u8, const OWNER: u8>() -> impl Projectable {
    g::seq(
        g::send::<OWNER, OWNER, RejectRequest>(),
        g::seq(
            g::send::<OWNER, VM, RequestReady>(),
            g::route(
                g::seq(g::send::<VM, OWNER, Open>(), g::send::<OWNER, VM, Opened>()),
                g::route(
                    g::seq(
                        g::send::<VM, OWNER, Write>(),
                        g::send::<OWNER, VM, Written>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<VM, OWNER, Commit>(),
                            g::send::<OWNER, VM, Unavailable>(),
                        ),
                        g::send::<VM, OWNER, Exit>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn closed_review_a() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_A, REVIEW_A, RejectRequest>(),
        g::seq(
            g::send::<REVIEW_A, REVIEW_A_WASI, ReviewRequestReady>(),
            g::route(
                g::seq(
                    g::send::<REVIEW_A_WASI, REVIEW_A, Open>(),
                    g::send::<REVIEW_A, REVIEW_A_WASI, Opened>(),
                ),
                g::route(
                    g::seq(
                        g::send::<REVIEW_A_WASI, REVIEW_A, Write>(),
                        g::send::<REVIEW_A, REVIEW_A_WASI, Written>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<REVIEW_A_WASI, REVIEW_A, Commit>(),
                            g::send::<REVIEW_A, REVIEW_A_WASI, Unavailable>(),
                        ),
                        g::send::<REVIEW_A_WASI, REVIEW_A, Exit>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}
fn closed_review_b() -> impl Projectable {
    g::seq(
        g::send::<REVIEW_B, REVIEW_B, RejectRequest>(),
        g::seq(
            g::send::<REVIEW_B, REVIEW_B_WASI, ReviewRequestReadyB>(),
            g::route(
                g::seq(
                    g::send::<REVIEW_B_WASI, REVIEW_B, OpenB>(),
                    g::send::<REVIEW_B, REVIEW_B_WASI, OpenedB>(),
                ),
                g::route(
                    g::seq(
                        g::send::<REVIEW_B_WASI, REVIEW_B, WriteB>(),
                        g::send::<REVIEW_B, REVIEW_B_WASI, WrittenB>(),
                    ),
                    g::route(
                        g::seq(
                            g::send::<REVIEW_B_WASI, REVIEW_B, CommitB>(),
                            g::send::<REVIEW_B, REVIEW_B_WASI, UnavailableB>(),
                        ),
                        g::send::<REVIEW_B_WASI, REVIEW_B, ExitB>(),
                    ),
                ),
            ),
        ),
    )
    .roll()
}

pub fn programs() -> Programs {
    let implement_a = g::seq(
        g::par(
            g::seq(g::send::<A, A, BeginFeature>(), implement_a()),
            g::seq(
                g::send::<REVIEW_A, REVIEW_A, BeginReviewVisit>(),
                g::seq(
                    g::send::<REVIEW_A, REVIEW_A_WASI, ReviewRequestReady>(),
                    g::seq(
                        g::send::<REVIEW_A_WASI, REVIEW_A, Open>(),
                        g::send::<REVIEW_A, A, ReviewRequested>(),
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<A, REVIEW_A, Ready>(),
            g::seq(
                g::send::<REVIEW_A, REVIEW_A_WASI, Opened>(),
                g::seq(
                    requests::<REVIEW_A_WASI, REVIEW_A, CHECK_REVIEW_A>(),
                    g::send::<REVIEW_A, REVIEW_A, Review>(),
                ),
            ),
        ),
    );
    let accepted_a = g::seq(
        g::send::<REVIEW_A, REPO, ApproveRepoA>(),
        g::seq(
            g::send::<REPO, REVIEW_A, DecisionRecordedA>(),
            g::seq(
                g::send::<REVIEW_A, A, Approved<'static>>(),
                g::seq(
                    g::send::<REVIEW_A, REVIEW_A_WASI, Result<'static>>(),
                    g::seq(
                        g::send::<REVIEW_A, REVIEW_A_WASI, ServiceDone>(),
                        g::seq(
                            g::send::<A, A_WASI, Result<'static>>(),
                            g::seq(
                                g::send::<A, A_WASI, ServiceDone>(),
                                g::seq(
                                    g::par(
                                        g::seq(
                                            merge_a(),
                                            g::seq(
                                                g::send::<A, A_WASI, Result<'static>>(),
                                                g::seq(
                                                    g::send::<A, A_WASI, ServiceDone>(),
                                                    closed::<A_WASI, A>(),
                                                ),
                                            ),
                                        ),
                                        g::seq(
                                            closed_review_a(),
                                            g::send::<REVIEW_A, A, ReviewerClosed>(),
                                        ),
                                    ),
                                    g::seq(
                                        g::send::<A, REVIEW_A, Merged>(),
                                        g::send::<REVIEW_A, REVIEW_A_WASI, ReviewExitAck>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let revise_a = g::seq(
        g::send::<REVIEW_A, REPO, ReworkRepoA>(),
        g::seq(
            g::send::<REPO, REVIEW_A, DecisionRecordedA>(),
            g::seq(
                g::send::<REVIEW_A, A, Revise<'static>>(),
                g::seq(
                    g::send::<REVIEW_A, REVIEW_A_WASI, Result<'static>>(),
                    g::seq(
                        g::send::<REVIEW_A, REVIEW_A_WASI, ServiceDone>(),
                        g::seq(
                            g::send::<A, A_WASI, Result<'static>>(),
                            g::send::<A, A_WASI, ServiceDone>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let feature_a = g::seq(
        implement_a,
        g::route(accepted_a, revise_a).resolve::<DECIDE_A>(),
    )
    .roll();
    let implement_b = g::seq(
        g::par(
            g::seq(g::send::<B, B, BeginFeature>(), implement_b()),
            g::seq(
                g::send::<REVIEW_B, REVIEW_B, BeginReviewVisit>(),
                g::seq(
                    g::send::<REVIEW_B, REVIEW_B_WASI, ReviewRequestReadyB>(),
                    g::seq(
                        g::send::<REVIEW_B_WASI, REVIEW_B, OpenB>(),
                        g::send::<REVIEW_B, B, ReviewRequestedB>(),
                    ),
                ),
            ),
        ),
        g::seq(
            g::send::<B, REVIEW_B, Ready>(),
            g::seq(
                g::send::<REVIEW_B, REVIEW_B_WASI, OpenedB>(),
                g::seq(review_b_requests(), g::send::<REVIEW_B, REVIEW_B, Review>()),
            ),
        ),
    );
    let accepted_b = g::seq(
        g::send::<REVIEW_B, REPO, ApproveRepoB>(),
        g::seq(
            g::send::<REPO, REVIEW_B, DecisionRecordedB>(),
            g::seq(
                g::send::<REVIEW_B, B, Approved<'static>>(),
                g::seq(
                    g::send::<REVIEW_B, REVIEW_B_WASI, ResultB<'static>>(),
                    g::seq(
                        g::send::<REVIEW_B, REVIEW_B_WASI, ServiceDoneB>(),
                        g::seq(
                            g::send::<B, B_WASI, Result<'static>>(),
                            g::seq(
                                g::send::<B, B_WASI, ServiceDone>(),
                                g::seq(
                                    g::par(
                                        g::seq(
                                            merge_b(),
                                            g::seq(
                                                g::send::<B, B_WASI, Result<'static>>(),
                                                g::seq(
                                                    g::send::<B, B_WASI, ServiceDone>(),
                                                    closed::<B_WASI, B>(),
                                                ),
                                            ),
                                        ),
                                        g::seq(
                                            closed_review_b(),
                                            g::send::<REVIEW_B, B, ReviewerClosed>(),
                                        ),
                                    ),
                                    g::seq(
                                        g::send::<B, REVIEW_B, Merged>(),
                                        g::send::<REVIEW_B, REVIEW_B_WASI, ReviewExitAckB>(),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let revise_b = g::seq(
        g::send::<REVIEW_B, REPO, ReworkRepoB>(),
        g::seq(
            g::send::<REPO, REVIEW_B, DecisionRecordedB>(),
            g::seq(
                g::send::<REVIEW_B, B, Revise<'static>>(),
                g::seq(
                    g::send::<REVIEW_B, REVIEW_B_WASI, ResultB<'static>>(),
                    g::seq(
                        g::send::<REVIEW_B, REVIEW_B_WASI, ServiceDoneB>(),
                        g::seq(
                            g::send::<B, B_WASI, Result<'static>>(),
                            g::send::<B, B_WASI, ServiceDone>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    let feature_b = g::seq(
        implement_b,
        g::route(accepted_b, revise_b).resolve::<DECIDE_B>(),
    )
    .roll();
    let task = g::seq(
        g::par(
            feature_a,
            g::seq(feature_b, g::send::<REVIEW_B, REVIEW_A, BothMerged>()),
        ),
        g::route(
            g::send::<REVIEW_A, REPO, RepoExit>(),
            g::send::<REVIEW_A, REPO, RepoFailed>(),
        ),
    );
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
        review_a_wasi: project(&protocol),
        review_b_wasi: project(&protocol),
        a: project(&protocol),
        b: project(&protocol),
        review_a: project(&protocol),
        review_b: project(&protocol),
        repo: project(&protocol),
        a_io: project(&protocol),
        b_io: project(&protocol),
        review_io: project(&protocol),
    }
}
