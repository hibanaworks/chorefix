//! One repeated round orders competing loans while independent work overlaps.
use super::*;
pub const DISPATCH: u8 = 200;
pub const READY_A: u16 = 300;
pub const READY_B: u16 = 301;
pub const READY_C: u16 = 302;
pub const READY_D: u16 = 303;
pub const READY_E: u16 = 304;
pub const READY_F: u16 = 305;
pub const READY_G: u16 = 306;
pub const ALL_TESTS: u16 = 206;
pub type ChooseA = g::Msg<200, ()>;
pub type ChooseB = g::Msg<201, ()>;
pub type ChooseC = g::Msg<202, ()>;
pub type ChooseD = g::Msg<203, ()>;
pub type ChooseE = g::Msg<204, ()>;
pub type ChooseF = g::Msg<205, ()>;
pub type ChooseG = g::Msg<206, ()>;
pub type LocalLane = g::Msg<208, ()>;
pub type Complete = g::Msg<209, ()>;
pub type Aborted = g::Msg<210, ()>;
pub type SkipA = g::Msg<220, ()>;
pub type SkipB = g::Msg<221, ()>;
pub type SkipC = g::Msg<222, ()>;
pub type SkipD = g::Msg<223, ()>;
pub type SkipE = g::Msg<224, ()>;
pub type SkipF = g::Msg<225, ()>;
pub type SkipG = g::Msg<226, ()>;
pub type SubmittedA = g::Msg<230, ()>;
pub type SubmittedB = g::Msg<231, ()>;
pub type SubmittedC = g::Msg<232, ()>;
pub type SubmittedD = g::Msg<233, ()>;
pub type SubmittedE = g::Msg<234, ()>;
pub type SubmittedF = g::Msg<235, ()>;
pub type SubmittedG = g::Msg<236, ()>;

pub type FinishA = g::Msg<40, ()>;
pub type DoneA = g::Msg<50, ()>;
pub type AbortA = g::Msg<60, ()>;
pub type FinishB = g::Msg<41, ()>;
pub type DoneB = g::Msg<51, ()>;
pub type AbortB = g::Msg<61, ()>;
pub type FinishC = g::Msg<42, ()>;
pub type DoneC = g::Msg<52, ()>;
pub type AbortC = g::Msg<62, ()>;
pub type FinishD = g::Msg<43, ()>;
pub type DoneD = g::Msg<53, ()>;
pub type AbortD = g::Msg<63, ()>;
pub type FinishE = g::Msg<44, ()>;
pub type DoneE = g::Msg<54, ()>;
pub type AbortE = g::Msg<64, ()>;
pub type FinishF = g::Msg<45, ()>;
pub type DoneF = g::Msg<55, ()>;
pub type AbortF = g::Msg<65, ()>;
pub type FinishG = g::Msg<46, ()>;
pub type DoneG = g::Msg<56, ()>;
pub type AbortG = g::Msg<66, ()>;
// Every potentially competing operation has one message identity.
pub trait Loan: g::Message<Payload = ()> {
    type Local: g::Message<Payload = ()>;
    type AvailableFirst: g::Message<Payload = ()>;
    type AvailableSecond: g::Message<Payload = ()>;
    type AcquireFirst: g::Message<Payload = ()>;
    type AcquireSecond: g::Message<Payload = ()>;
    type GrantedFirst: g::Message<Payload = ()>;
    type GrantedSecond: g::Message<Payload = ()>;
    type ReturnFirst: g::Message<Payload = ()>;
    type ReturnSecond: g::Message<Payload = ()>;
    type ReturnedFirst: g::Message<Payload = ()>;
    type ReturnedSecond: g::Message<Payload = ()>;
}
impl Loan for ChooseA {
    type Local = g::Msg<70, ()>;
    type AvailableFirst = g::Msg<71, ()>;
    type AvailableSecond = g::Msg<72, ()>;
    type AcquireFirst = g::Msg<73, ()>;
    type AcquireSecond = g::Msg<74, ()>;
    type GrantedFirst = g::Msg<75, ()>;
    type GrantedSecond = g::Msg<76, ()>;
    type ReturnFirst = g::Msg<77, ()>;
    type ReturnSecond = g::Msg<78, ()>;
    type ReturnedFirst = g::Msg<79, ()>;
    type ReturnedSecond = g::Msg<80, ()>;
}
impl Loan for ChooseB {
    type Local = g::Msg<81, ()>;
    type AvailableFirst = g::Msg<82, ()>;
    type AvailableSecond = g::Msg<83, ()>;
    type AcquireFirst = g::Msg<84, ()>;
    type AcquireSecond = g::Msg<85, ()>;
    type GrantedFirst = g::Msg<86, ()>;
    type GrantedSecond = g::Msg<87, ()>;
    type ReturnFirst = g::Msg<88, ()>;
    type ReturnSecond = g::Msg<89, ()>;
    type ReturnedFirst = g::Msg<90, ()>;
    type ReturnedSecond = g::Msg<91, ()>;
}
impl Loan for ChooseC {
    type Local = g::Msg<92, ()>;
    type AvailableFirst = g::Msg<93, ()>;
    type AvailableSecond = g::Msg<94, ()>;
    type AcquireFirst = g::Msg<95, ()>;
    type AcquireSecond = g::Msg<96, ()>;
    type GrantedFirst = g::Msg<97, ()>;
    type GrantedSecond = g::Msg<98, ()>;
    type ReturnFirst = g::Msg<99, ()>;
    type ReturnSecond = g::Msg<100, ()>;
    type ReturnedFirst = g::Msg<101, ()>;
    type ReturnedSecond = g::Msg<102, ()>;
}
impl Loan for ChooseD {
    type Local = g::Msg<103, ()>;
    type AvailableFirst = g::Msg<104, ()>;
    type AvailableSecond = g::Msg<105, ()>;
    type AcquireFirst = g::Msg<106, ()>;
    type AcquireSecond = g::Msg<107, ()>;
    type GrantedFirst = g::Msg<108, ()>;
    type GrantedSecond = g::Msg<109, ()>;
    type ReturnFirst = g::Msg<110, ()>;
    type ReturnSecond = g::Msg<111, ()>;
    type ReturnedFirst = g::Msg<112, ()>;
    type ReturnedSecond = g::Msg<113, ()>;
}
impl Loan for ChooseE {
    type Local = g::Msg<114, ()>;
    type AvailableFirst = g::Msg<115, ()>;
    type AvailableSecond = g::Msg<116, ()>;
    type AcquireFirst = g::Msg<117, ()>;
    type AcquireSecond = g::Msg<118, ()>;
    type GrantedFirst = g::Msg<119, ()>;
    type GrantedSecond = g::Msg<120, ()>;
    type ReturnFirst = g::Msg<121, ()>;
    type ReturnSecond = g::Msg<122, ()>;
    type ReturnedFirst = g::Msg<123, ()>;
    type ReturnedSecond = g::Msg<124, ()>;
}
impl Loan for ChooseF {
    type Local = g::Msg<125, ()>;
    type AvailableFirst = g::Msg<126, ()>;
    type AvailableSecond = g::Msg<127, ()>;
    type AcquireFirst = g::Msg<128, ()>;
    type AcquireSecond = g::Msg<129, ()>;
    type GrantedFirst = g::Msg<130, ()>;
    type GrantedSecond = g::Msg<131, ()>;
    type ReturnFirst = g::Msg<132, ()>;
    type ReturnSecond = g::Msg<133, ()>;
    type ReturnedFirst = g::Msg<134, ()>;
    type ReturnedSecond = g::Msg<135, ()>;
}
impl Loan for ChooseG {
    type Local = g::Msg<136, ()>;
    type AvailableFirst = g::Msg<137, ()>;
    type AvailableSecond = g::Msg<138, ()>;
    type AcquireFirst = g::Msg<139, ()>;
    type AcquireSecond = g::Msg<140, ()>;
    type GrantedFirst = g::Msg<141, ()>;
    type GrantedSecond = g::Msg<142, ()>;
    type ReturnFirst = g::Msg<143, ()>;
    type ReturnSecond = g::Msg<144, ()>;
    type ReturnedFirst = g::Msg<145, ()>;
    type ReturnedSecond = g::Msg<146, ()>;
}
fn visit<
    const DEV: u8,
    const FIRST: u8,
    const SECOND: u8,
    const READY: u16,
    const TEST: u16,
    Work: Loan,
    Skip: g::Message<Payload = ()>,
    Submitted: g::Message<Payload = ()>,
>() -> impl Projectable {
    let acquire = g::par(
        g::seq(
            g::send::<FIRST, DEV, Work::AvailableFirst>(),
            g::seq(
                g::send::<DEV, FIRST, Work::AcquireFirst>(),
                g::send::<FIRST, DEV, Work::GrantedFirst>(),
            ),
        ),
        g::seq(
            g::send::<SECOND, DEV, Work::AvailableSecond>(),
            g::seq(
                g::send::<DEV, SECOND, Work::AcquireSecond>(),
                g::send::<SECOND, DEV, Work::GrantedSecond>(),
            ),
        ),
    );
    let release = g::par(
        g::seq(
            g::send::<DEV, FIRST, Work::ReturnFirst>(),
            g::send::<FIRST, DEV, Work::ReturnedFirst>(),
        ),
        g::seq(
            g::send::<DEV, SECOND, Work::ReturnSecond>(),
            g::send::<SECOND, DEV, Work::ReturnedSecond>(),
        ),
    );
    let tested = g::seq(
        g::send::<DEV, DEV, Test>(),
        g::route(
            g::send::<DEV, DEV, Finished>(),
            g::send::<DEV, DEV, Implement>(),
        )
        .resolve::<TEST>(),
    );
    let work = g::seq(
        g::send::<DISPATCH, DEV, Work>(),
        g::seq(
            g::send::<DISPATCH, FIRST, Work>(),
            g::seq(
                g::send::<DISPATCH, SECOND, Work>(),
                g::seq(
                    g::send::<DEV, DISPATCH, Submitted>(),
                    g::par(
                        g::par(
                            g::send::<DEV, DEV, Work::Local>(),
                            g::par(
                                g::send::<FIRST, FIRST, Work::Local>(),
                                g::send::<SECOND, SECOND, Work::Local>(),
                            ),
                        ),
                        g::seq(
                            acquire,
                            g::seq(g::send::<DEV, DEV, Commit>(), g::seq(release, tested)),
                        ),
                    ),
                ),
            ),
        ),
    );
    let skip = g::seq(
        g::send::<DISPATCH, DEV, Skip>(),
        g::seq(
            g::send::<DISPATCH, FIRST, Skip>(),
            g::send::<DISPATCH, SECOND, Skip>(),
        ),
    );
    g::route(work, skip).resolve::<READY>()
}
// The final handshake closes every loan-to-dispatch causal path.
fn finish<
    const DEV: u8,
    const TEST: u16,
    Finish: g::Message<Payload = ()>,
    Done: g::Message<Payload = ()>,
    Abort: g::Message<Payload = ()>,
>() -> impl Projectable {
    g::seq(
        g::send::<DISPATCH, DEV, Finish>(),
        g::route(
            g::send::<DEV, DISPATCH, Done>(),
            g::send::<DEV, DISPATCH, Abort>(),
        )
        .resolve::<TEST>(),
    )
}
fn notify<M: g::Message<Payload = ()>>() -> impl Projectable {
    g::seq(
        g::seq(
            g::seq(g::send::<DISPATCH, A, M>(), g::send::<DISPATCH, B, M>()),
            g::seq(g::send::<DISPATCH, C, M>(), g::send::<DISPATCH, D, M>()),
        ),
        g::seq(
            g::seq(
                g::send::<DISPATCH, E, M>(),
                g::seq(g::send::<DISPATCH, F, M>(), g::send::<DISPATCH, G, M>()),
            ),
            g::seq(
                g::seq(
                    g::seq(
                        g::send::<DISPATCH, AUTH_MODULE, M>(),
                        g::send::<DISPATCH, DATABASE_MODULE, M>(),
                    ),
                    g::seq(
                        g::send::<DISPATCH, API_MODULE, M>(),
                        g::send::<DISPATCH, PAYMENT_MODULE, M>(),
                    ),
                ),
                g::seq(
                    g::send::<DISPATCH, NOTIFY_MODULE, M>(),
                    g::seq(
                        g::send::<DISPATCH, ANALYTICS_MODULE, M>(),
                        g::send::<DISPATCH, SEARCH_MODULE, M>(),
                    ),
                ),
            ),
        ),
    )
}
pub struct Programs1H {
    pub dispatch: RoleProgram<DISPATCH>,
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub d: RoleProgram<D>,
    pub e: RoleProgram<E>,
    pub f: RoleProgram<F>,
    pub g: RoleProgram<G>,
    pub auth: RoleProgram<AUTH_MODULE>,
    pub database: RoleProgram<DATABASE_MODULE>,
    pub api: RoleProgram<API_MODULE>,
    pub payment: RoleProgram<PAYMENT_MODULE>,
    pub notify: RoleProgram<NOTIFY_MODULE>,
    pub analytics: RoleProgram<ANALYTICS_MODULE>,
    pub search: RoleProgram<SEARCH_MODULE>,
}
pub fn project_1h() -> Programs1H {
    let preparation = g::par(
        g::par(
            g::seq(g::send::<A, A, Design>(), g::send::<A, A, Implement>()),
            g::par(
                g::seq(g::send::<B, B, Design>(), g::send::<B, B, Implement>()),
                g::seq(g::send::<C, C, Design>(), g::send::<C, C, Implement>()),
            ),
        ),
        g::par(
            g::par(
                g::seq(g::send::<D, D, Design>(), g::send::<D, D, Implement>()),
                g::seq(g::send::<E, E, Design>(), g::send::<E, E, Implement>()),
            ),
            g::par(
                g::seq(g::send::<F, F, Design>(), g::send::<F, F, Implement>()),
                g::seq(g::send::<G, G, Design>(), g::send::<G, G, Implement>()),
            ),
        ),
    );
    let a = visit::<A, AUTH_MODULE, DATABASE_MODULE, READY_A, TEST_A, ChooseA, SkipA, SubmittedA>();
    let b = visit::<B, DATABASE_MODULE, API_MODULE, READY_B, TEST_B, ChooseB, SkipB, SubmittedB>();
    let c = visit::<C, API_MODULE, PAYMENT_MODULE, READY_C, TEST_C, ChooseC, SkipC, SubmittedC>();
    let d =
        visit::<D, PAYMENT_MODULE, NOTIFY_MODULE, READY_D, TEST_D, ChooseD, SkipD, SubmittedD>();
    let e =
        visit::<E, NOTIFY_MODULE, ANALYTICS_MODULE, READY_E, TEST_E, ChooseE, SkipE, SubmittedE>();
    let f =
        visit::<F, ANALYTICS_MODULE, SEARCH_MODULE, READY_F, TEST_F, ChooseF, SkipF, SubmittedF>();
    let g = visit::<G, AUTH_MODULE, SEARCH_MODULE, READY_G, TEST_G, ChooseG, SkipG, SubmittedG>();
    // Dispatch has one owner and order. It finishes at Submitted, before a
    // loan or test completes. Disjoint role-local loans can therefore overlap.
    // One outer roll retains each physical owner's complete loan ordering.
    let round = g::seq(g::seq(a, g::seq(c, e)), g::seq(g::seq(b, g::seq(d, f)), g));
    let final_handshakes = g::seq(
        g::seq(
            g::seq(
                g::seq(
                    g::seq(
                        g::seq(
                            finish::<A, TEST_A, FinishA, DoneA, AbortA>(),
                            finish::<B, TEST_B, FinishB, DoneB, AbortB>(),
                        ),
                        finish::<C, TEST_C, FinishC, DoneC, AbortC>(),
                    ),
                    finish::<D, TEST_D, FinishD, DoneD, AbortD>(),
                ),
                finish::<E, TEST_E, FinishE, DoneE, AbortE>(),
            ),
            finish::<F, TEST_F, FinishF, DoneF, AbortF>(),
        ),
        finish::<G, TEST_G, FinishG, DoneG, AbortG>(),
    );
    let protocol = g::seq(
        preparation,
        g::seq(
            round.roll(),
            g::route(
                g::seq(final_handshakes, notify::<Complete>()),
                notify::<Aborted>(),
            )
            .resolve::<ALL_TESTS>(),
        ),
    );
    Programs1H {
        dispatch: project(&protocol),
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        f: project(&protocol),
        g: project(&protocol),
        auth: project(&protocol),
        database: project(&protocol),
        api: project(&protocol),
        payment: project(&protocol),
        notify: project(&protocol),
        analytics: project(&protocol),
        search: project(&protocol),
    }
}
