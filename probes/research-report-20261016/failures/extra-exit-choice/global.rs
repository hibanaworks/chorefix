//! Research is independent. DOCUMENT and DATABASE visits have explicit order.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub const A: u8 = 0;
pub const B: u8 = 1;
pub const EDITOR: u8 = 2;
pub const DOCUMENT: u8 = 3;
pub const DATABASE: u8 = 4;
pub const REVIEW_A: u16 = 810;
pub const REVIEW_B: u16 = 811;
pub type Research = g::Msg<1, ()>;
pub type Write = g::Msg<2, ()>;
pub type References = g::Msg<3, ()>;
pub type Revise = g::Msg<4, ()>;
pub type ReviewA = g::Msg<8, ()>;
pub type ReviewB = g::Msg<9, ()>;
pub type Combine = g::Msg<10, ()>;

pub trait Messages {
    type DocAvailable: g::Message<Payload = ()>;
    type DocRequest: g::Message<Payload = ()>;
    type DocGranted: g::Message<Payload = ()>;
    type DocReturn: g::Message<Payload = ()>;
    type DocReturned: g::Message<Payload = ()>;
    type RefsRequest: g::Message<Payload = ()>;
    type RefsGranted: g::Message<Payload = ()>;
    type RefsReturn: g::Message<Payload = ()>;
    type RefsReturned: g::Message<Payload = ()>;
    type Ready: g::Message<Payload = ()>;
    type BeginReview: g::Message<Payload = ()>;
    type BeginDocumentReview: g::Message<Payload = ()>;
    type EndIteration: g::Message<Payload = ()>;
    type EndDocumentIteration: g::Message<Payload = ()>;
    type NeedsRevision: g::Message<Payload = ()>;
    type RevisionReady: g::Message<Payload = ()>;
    type NeedsDocumentRevision: g::Message<Payload = ()>;
    type RevisionAvailable: g::Message<Payload = ()>;
    type RevisionRequest: g::Message<Payload = ()>;
    type RevisionGranted: g::Message<Payload = ()>;
    type RevisionReturn: g::Message<Payload = ()>;
    type RevisionReturned: g::Message<Payload = ()>;
    type Resubmitted: g::Message<Payload = ()>;
    type Accepted: g::Message<Payload = ()>;
    type Aborted: g::Message<Payload = ()>;
    type Review: g::Message<Payload = ()>;
}

pub mod author_a {
    use hibana::g;
    pub type DocRequest = g::Msg<40, ()>;
    pub type DocGranted = g::Msg<41, ()>;
    pub type DocReturn = g::Msg<42, ()>;
    pub type DocReturned = g::Msg<43, ()>;
    pub type RefsRequest = g::Msg<44, ()>;
    pub type RefsGranted = g::Msg<45, ()>;
    pub type RefsReturn = g::Msg<46, ()>;
    pub type RefsReturned = g::Msg<47, ()>;
    pub type Ready = g::Msg<48, ()>;
    pub type BeginReview = g::Msg<49, ()>;
    pub type BeginDocumentReview = g::Msg<50, ()>;
    pub type EndIteration = g::Msg<51, ()>;
    pub type EndDocumentIteration = g::Msg<52, ()>;
    pub type NeedsRevision = g::Msg<53, ()>;
    pub type NeedsDocumentRevision = g::Msg<54, ()>;
    pub type RevisionAvailable = g::Msg<55, ()>;
    pub type RevisionRequest = g::Msg<56, ()>;
    pub type RevisionGranted = g::Msg<57, ()>;
    pub type RevisionReturn = g::Msg<58, ()>;
    pub type RevisionReturned = g::Msg<59, ()>;
    pub type Resubmitted = g::Msg<60, ()>;
    pub type Accepted = g::Msg<61, ()>;
    pub type Aborted = g::Msg<62, ()>;
    pub type DocAvailable = g::Msg<63, ()>;
    pub type RevisionReady = g::Msg<64, ()>;
}
pub struct AuthorA;
impl Messages for AuthorA {
    type DocAvailable = author_a::DocAvailable;
    type DocRequest = author_a::DocRequest;
    type DocGranted = author_a::DocGranted;
    type DocReturn = author_a::DocReturn;
    type DocReturned = author_a::DocReturned;
    type RefsRequest = author_a::RefsRequest;
    type RefsGranted = author_a::RefsGranted;
    type RefsReturn = author_a::RefsReturn;
    type RefsReturned = author_a::RefsReturned;
    type Ready = author_a::Ready;
    type BeginReview = author_a::BeginReview;
    type BeginDocumentReview = author_a::BeginDocumentReview;
    type EndIteration = author_a::EndIteration;
    type EndDocumentIteration = author_a::EndDocumentIteration;
    type NeedsRevision = author_a::NeedsRevision;
    type RevisionReady = author_a::RevisionReady;
    type NeedsDocumentRevision = author_a::NeedsDocumentRevision;
    type RevisionAvailable = author_a::RevisionAvailable;
    type RevisionRequest = author_a::RevisionRequest;
    type RevisionGranted = author_a::RevisionGranted;
    type RevisionReturn = author_a::RevisionReturn;
    type RevisionReturned = author_a::RevisionReturned;
    type Resubmitted = author_a::Resubmitted;
    type Accepted = author_a::Accepted;
    type Aborted = author_a::Aborted;
    type Review = ReviewA;
}
pub mod author_b {
    use hibana::g;
    pub type DocRequest = g::Msg<80, ()>;
    pub type DocGranted = g::Msg<81, ()>;
    pub type DocReturn = g::Msg<82, ()>;
    pub type DocReturned = g::Msg<83, ()>;
    pub type RefsRequest = g::Msg<84, ()>;
    pub type RefsGranted = g::Msg<85, ()>;
    pub type RefsReturn = g::Msg<86, ()>;
    pub type RefsReturned = g::Msg<87, ()>;
    pub type Ready = g::Msg<88, ()>;
    pub type BeginReview = g::Msg<89, ()>;
    pub type BeginDocumentReview = g::Msg<90, ()>;
    pub type EndIteration = g::Msg<91, ()>;
    pub type EndDocumentIteration = g::Msg<92, ()>;
    pub type NeedsRevision = g::Msg<93, ()>;
    pub type NeedsDocumentRevision = g::Msg<94, ()>;
    pub type RevisionAvailable = g::Msg<95, ()>;
    pub type RevisionRequest = g::Msg<96, ()>;
    pub type RevisionGranted = g::Msg<97, ()>;
    pub type RevisionReturn = g::Msg<98, ()>;
    pub type RevisionReturned = g::Msg<99, ()>;
    pub type Resubmitted = g::Msg<100, ()>;
    pub type Accepted = g::Msg<101, ()>;
    pub type Aborted = g::Msg<102, ()>;
    pub type RefsAvailable = g::Msg<103, ()>;
    pub type DocAvailable = g::Msg<104, ()>;
    pub type RevisionReady = g::Msg<105, ()>;
}
pub struct AuthorB;
impl Messages for AuthorB {
    type DocAvailable = author_b::DocAvailable;
    type DocRequest = author_b::DocRequest;
    type DocGranted = author_b::DocGranted;
    type DocReturn = author_b::DocReturn;
    type DocReturned = author_b::DocReturned;
    type RefsRequest = author_b::RefsRequest;
    type RefsGranted = author_b::RefsGranted;
    type RefsReturn = author_b::RefsReturn;
    type RefsReturned = author_b::RefsReturned;
    type Ready = author_b::Ready;
    type BeginReview = author_b::BeginReview;
    type BeginDocumentReview = author_b::BeginDocumentReview;
    type EndIteration = author_b::EndIteration;
    type EndDocumentIteration = author_b::EndDocumentIteration;
    type NeedsRevision = author_b::NeedsRevision;
    type RevisionReady = author_b::RevisionReady;
    type NeedsDocumentRevision = author_b::NeedsDocumentRevision;
    type RevisionAvailable = author_b::RevisionAvailable;
    type RevisionRequest = author_b::RevisionRequest;
    type RevisionGranted = author_b::RevisionGranted;
    type RevisionReturn = author_b::RevisionReturn;
    type RevisionReturned = author_b::RevisionReturned;
    type Resubmitted = author_b::Resubmitted;
    type Accepted = author_b::Accepted;
    type Aborted = author_b::Aborted;
    type Review = ReviewB;
}
pub type CombineRequest = g::Msg<120, ()>;
pub type CombineGranted = g::Msg<121, ()>;
pub type CombineReturn = g::Msg<122, ()>;
pub type CombineReturned = g::Msg<123, ()>;
pub type AbortDocumentA = g::Msg<124, ()>;
pub type AbortDocumentB = g::Msg<125, ()>;
pub type BeginCombine = g::Msg<127, ()>;

fn write<const AUTHOR: u8, M: Messages>() -> impl Projectable {
    g::seq(
        g::send::<DOCUMENT, AUTHOR, M::DocAvailable>(),
        g::seq(
            g::send::<AUTHOR, DOCUMENT, M::DocRequest>(),
            g::seq(
                g::send::<DOCUMENT, AUTHOR, M::DocGranted>(),
                g::seq(
                    g::send::<AUTHOR, AUTHOR, Write>(),
                    g::seq(
                        g::send::<AUTHOR, DOCUMENT, M::DocReturn>(),
                        g::send::<DOCUMENT, AUTHOR, M::DocReturned>(),
                    ),
                ),
            ),
        ),
    )
}
fn references<const AUTHOR: u8, M: Messages>() -> impl Projectable {
    g::seq(
        g::send::<AUTHOR, DATABASE, M::RefsRequest>(),
        g::seq(
            g::send::<DATABASE, AUTHOR, M::RefsGranted>(),
            g::seq(
                g::send::<AUTHOR, AUTHOR, References>(),
                g::seq(
                    g::send::<AUTHOR, DATABASE, M::RefsReturn>(),
                    g::send::<DATABASE, AUTHOR, M::RefsReturned>(),
                ),
            ),
        ),
    )
}
fn revisions<const AUTHOR: u8, const ID: u16, M: Messages>() -> impl Projectable {
    let done = g::par(
        g::send::<EDITOR, AUTHOR, M::EndIteration>(),
        g::send::<EDITOR, DOCUMENT, M::EndDocumentIteration>(),
    );
    let revision = g::seq(
        g::send::<EDITOR, AUTHOR, M::NeedsRevision>(),
        g::seq(
            g::send::<AUTHOR, EDITOR, M::RevisionReady>(),
            g::seq(
                g::send::<EDITOR, DOCUMENT, M::NeedsDocumentRevision>(),
                g::seq(
                    g::send::<DOCUMENT, AUTHOR, M::RevisionAvailable>(),
                    g::seq(
                        g::send::<AUTHOR, DOCUMENT, M::RevisionRequest>(),
                        g::seq(
                            g::send::<DOCUMENT, AUTHOR, M::RevisionGranted>(),
                            g::seq(
                                g::send::<AUTHOR, AUTHOR, Revise>(),
                                g::seq(
                                    g::send::<AUTHOR, DOCUMENT, M::RevisionReturn>(),
                                    g::seq(
                                        g::send::<DOCUMENT, AUTHOR, M::RevisionReturned>(),
                                        g::seq(
                                            g::send::<AUTHOR, EDITOR, M::Resubmitted>(),
                                            g::send::<EDITOR, EDITOR, M::Review>(),
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
    g::route(done, revision).resolve::<ID>().roll()
}
pub struct Programs3E {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub editor: RoleProgram<EDITOR>,
    pub document: RoleProgram<DOCUMENT>,
    pub database: RoleProgram<DATABASE>,
}
pub fn project_3e() -> Programs3E {
    let research = g::par(g::send::<A, A, Research>(), g::send::<B, B, Research>());
    // A's DATABASE update and B's DOCUMENT write can execute together.
    // Each physical owner's own visits remain sequential in the projection.
    let initial = g::seq(
        research,
        g::seq(
            write::<A, AuthorA>(),
            g::seq(
                g::par(
                    g::seq(
                        references::<A, AuthorA>(),
                        g::send::<DATABASE, B, author_b::RefsAvailable>(),
                    ),
                    write::<B, AuthorB>(),
                ),
                references::<B, AuthorB>(),
            ),
        ),
    );
    let submitted = g::par(
        g::send::<A, EDITOR, author_a::Ready>(),
        g::send::<B, EDITOR, author_b::Ready>(),
    );
    let start_a = g::par(
        g::send::<EDITOR, A, author_a::BeginReview>(),
        g::send::<EDITOR, DOCUMENT, author_a::BeginDocumentReview>(),
    );
    let start_b = g::par(
        g::send::<EDITOR, B, author_b::BeginReview>(),
        g::send::<EDITOR, DOCUMENT, author_b::BeginDocumentReview>(),
    );
    let combine = g::seq(
        g::send::<EDITOR, DOCUMENT, BeginCombine>(),
        g::seq(
            g::send::<EDITOR, DOCUMENT, CombineRequest>(),
            g::seq(
                g::send::<DOCUMENT, EDITOR, CombineGranted>(),
                g::seq(
                    g::send::<EDITOR, EDITOR, Combine>(),
                    g::seq(
                        g::send::<EDITOR, DOCUMENT, CombineReturn>(),
                        g::send::<DOCUMENT, EDITOR, CombineReturned>(),
                    ),
                ),
            ),
        ),
    );
    let passed_b = g::seq(g::send::<EDITOR, B, author_b::Accepted>(), combine);
    let abort_b = g::par(
        g::send::<EDITOR, B, author_b::Aborted>(),
        g::send::<EDITOR, DOCUMENT, AbortDocumentB>(),
    );
    let review_b = g::seq(
        start_b,
        g::seq(
            g::send::<EDITOR, EDITOR, ReviewB>(),
            g::seq(
                revisions::<B, REVIEW_B, AuthorB>(),
                g::route(passed_b, abort_b).resolve::<REVIEW_B>(),
            ),
        ),
    );
    let passed_a = g::seq(g::send::<EDITOR, A, author_a::Accepted>(), review_b);
    let abort_a = g::par(
        g::send::<EDITOR, A, author_a::Aborted>(),
        g::par(
            g::send::<EDITOR, B, author_b::Aborted>(),
            g::send::<EDITOR, DOCUMENT, AbortDocumentA>(),
        ),
    );
    let review_a = g::seq(
        start_a,
        g::seq(
            g::send::<EDITOR, EDITOR, ReviewA>(),
            g::seq(
                revisions::<A, REVIEW_A, AuthorA>(),
                g::route(passed_a, abort_a).resolve::<REVIEW_A>(),
            ),
        ),
    );
    let global = g::seq(initial, g::seq(submitted, review_a));
    Programs3E {
        a: project(&global),
        b: project(&global),
        editor: project(&global),
        document: project(&global),
        database: project(&global),
    }
}
