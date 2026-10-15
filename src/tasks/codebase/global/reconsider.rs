//! Concurrent reconsideration with parallel requests and ordered grants.
use super::*;
pub trait Messages {
    type AcquireSecond: g::Message<Payload = ()>;
    type AcquireFirst: g::Message<Payload = ()>;
    type Handoff: g::Message<Payload = ()>;
    type SecondReady: g::Message<Payload = ()>;
    type GrantedFirst: g::Message<Payload = ()>;
    type ConfirmSecond: g::Message<Payload = ()>;
    type GrantedSecond: g::Message<Payload = ()>;
    type ReturnFirst: g::Message<Payload = ()>;
    type ReturnedFirst: g::Message<Payload = ()>;
    type ReturnSecond: g::Message<Payload = ()>;
    type ReturnedSecond: g::Message<Payload = ()>;
    type CloseFirst: g::Message<Payload = ()>;
    type CloseHandoff: g::Message<Payload = ()>;
    type ClosePrimed: g::Message<Payload = ()>;
    type CloseFirstRequest: g::Message<Payload = ()>;
    type CloseSecondRequest: g::Message<Payload = ()>;
    type ClosedFirst: g::Message<Payload = ()>;
    type ClosedSecond: g::Message<Payload = ()>;
    type DoneFirst: g::Message<Payload = ()>;
    type DoneSecond: g::Message<Payload = ()>;
    type AbortFirst: g::Message<Payload = ()>;
    type AbortSecond: g::Message<Payload = ()>;
    type Finished: g::Message<Payload = ()>;
    type Aborted: g::Message<Payload = ()>;
}
pub mod a {
    use hibana::g;
    pub type AcquireSecond = g::Msg<16, ()>;
    pub type AcquireFirst = g::Msg<17, ()>;
    pub type Handoff = g::Msg<18, ()>;
    pub type SecondReady = g::Msg<19, ()>;
    pub type GrantedFirst = g::Msg<20, ()>;
    pub type ConfirmSecond = g::Msg<21, ()>;
    pub type GrantedSecond = g::Msg<22, ()>;
    pub type ReturnFirst = g::Msg<23, ()>;
    pub type ReturnedFirst = g::Msg<24, ()>;
    pub type ReturnSecond = g::Msg<25, ()>;
    pub type ReturnedSecond = g::Msg<26, ()>;
    pub type CloseFirst = g::Msg<27, ()>;
    pub type CloseHandoff = g::Msg<28, ()>;
    pub type ClosePrimed = g::Msg<29, ()>;
    pub type CloseFirstRequest = g::Msg<30, ()>;
    pub type CloseSecondRequest = g::Msg<31, ()>;
    pub type ClosedFirst = g::Msg<32, ()>;
    pub type ClosedSecond = g::Msg<33, ()>;
    pub type DoneFirst = g::Msg<34, ()>;
    pub type DoneSecond = g::Msg<35, ()>;
    pub type AbortFirst = g::Msg<36, ()>;
    pub type AbortSecond = g::Msg<37, ()>;
    pub type Finished = g::Msg<38, ()>;
    pub type Aborted = g::Msg<39, ()>;
}
pub struct LoanA;
impl Messages for LoanA {
    type AcquireSecond = a::AcquireSecond;
    type AcquireFirst = a::AcquireFirst;
    type Handoff = a::Handoff;
    type SecondReady = a::SecondReady;
    type GrantedFirst = a::GrantedFirst;
    type ConfirmSecond = a::ConfirmSecond;
    type GrantedSecond = a::GrantedSecond;
    type ReturnFirst = a::ReturnFirst;
    type ReturnedFirst = a::ReturnedFirst;
    type ReturnSecond = a::ReturnSecond;
    type ReturnedSecond = a::ReturnedSecond;
    type CloseFirst = a::CloseFirst;
    type CloseHandoff = a::CloseHandoff;
    type ClosePrimed = a::ClosePrimed;
    type CloseFirstRequest = a::CloseFirstRequest;
    type CloseSecondRequest = a::CloseSecondRequest;
    type ClosedFirst = a::ClosedFirst;
    type ClosedSecond = a::ClosedSecond;
    type DoneFirst = a::DoneFirst;
    type DoneSecond = a::DoneSecond;
    type AbortFirst = a::AbortFirst;
    type AbortSecond = a::AbortSecond;
    type Finished = a::Finished;
    type Aborted = a::Aborted;
}
pub mod b {
    use hibana::g;
    pub type AcquireSecond = g::Msg<48, ()>;
    pub type AcquireFirst = g::Msg<49, ()>;
    pub type Handoff = g::Msg<50, ()>;
    pub type SecondReady = g::Msg<51, ()>;
    pub type GrantedFirst = g::Msg<52, ()>;
    pub type ConfirmSecond = g::Msg<53, ()>;
    pub type GrantedSecond = g::Msg<54, ()>;
    pub type ReturnFirst = g::Msg<55, ()>;
    pub type ReturnedFirst = g::Msg<56, ()>;
    pub type ReturnSecond = g::Msg<57, ()>;
    pub type ReturnedSecond = g::Msg<58, ()>;
    pub type CloseFirst = g::Msg<59, ()>;
    pub type CloseHandoff = g::Msg<60, ()>;
    pub type ClosePrimed = g::Msg<61, ()>;
    pub type CloseFirstRequest = g::Msg<62, ()>;
    pub type CloseSecondRequest = g::Msg<63, ()>;
    pub type ClosedFirst = g::Msg<64, ()>;
    pub type ClosedSecond = g::Msg<65, ()>;
    pub type DoneFirst = g::Msg<66, ()>;
    pub type DoneSecond = g::Msg<67, ()>;
    pub type AbortFirst = g::Msg<68, ()>;
    pub type AbortSecond = g::Msg<69, ()>;
    pub type Finished = g::Msg<70, ()>;
    pub type Aborted = g::Msg<71, ()>;
}
pub struct LoanB;
impl Messages for LoanB {
    type AcquireSecond = b::AcquireSecond;
    type AcquireFirst = b::AcquireFirst;
    type Handoff = b::Handoff;
    type SecondReady = b::SecondReady;
    type GrantedFirst = b::GrantedFirst;
    type ConfirmSecond = b::ConfirmSecond;
    type GrantedSecond = b::GrantedSecond;
    type ReturnFirst = b::ReturnFirst;
    type ReturnedFirst = b::ReturnedFirst;
    type ReturnSecond = b::ReturnSecond;
    type ReturnedSecond = b::ReturnedSecond;
    type CloseFirst = b::CloseFirst;
    type CloseHandoff = b::CloseHandoff;
    type ClosePrimed = b::ClosePrimed;
    type CloseFirstRequest = b::CloseFirstRequest;
    type CloseSecondRequest = b::CloseSecondRequest;
    type ClosedFirst = b::ClosedFirst;
    type ClosedSecond = b::ClosedSecond;
    type DoneFirst = b::DoneFirst;
    type DoneSecond = b::DoneSecond;
    type AbortFirst = b::AbortFirst;
    type AbortSecond = b::AbortSecond;
    type Finished = b::Finished;
    type Aborted = b::Aborted;
}
pub mod c {
    use hibana::g;
    pub type AcquireSecond = g::Msg<80, ()>;
    pub type AcquireFirst = g::Msg<81, ()>;
    pub type Handoff = g::Msg<82, ()>;
    pub type SecondReady = g::Msg<83, ()>;
    pub type GrantedFirst = g::Msg<84, ()>;
    pub type ConfirmSecond = g::Msg<85, ()>;
    pub type GrantedSecond = g::Msg<86, ()>;
    pub type ReturnFirst = g::Msg<87, ()>;
    pub type ReturnedFirst = g::Msg<88, ()>;
    pub type ReturnSecond = g::Msg<89, ()>;
    pub type ReturnedSecond = g::Msg<90, ()>;
    pub type CloseFirst = g::Msg<91, ()>;
    pub type CloseHandoff = g::Msg<92, ()>;
    pub type ClosePrimed = g::Msg<93, ()>;
    pub type CloseFirstRequest = g::Msg<94, ()>;
    pub type CloseSecondRequest = g::Msg<95, ()>;
    pub type ClosedFirst = g::Msg<96, ()>;
    pub type ClosedSecond = g::Msg<97, ()>;
    pub type DoneFirst = g::Msg<98, ()>;
    pub type DoneSecond = g::Msg<99, ()>;
    pub type AbortFirst = g::Msg<100, ()>;
    pub type AbortSecond = g::Msg<101, ()>;
    pub type Finished = g::Msg<102, ()>;
    pub type Aborted = g::Msg<103, ()>;
}
pub struct LoanC;
impl Messages for LoanC {
    type AcquireSecond = c::AcquireSecond;
    type AcquireFirst = c::AcquireFirst;
    type Handoff = c::Handoff;
    type SecondReady = c::SecondReady;
    type GrantedFirst = c::GrantedFirst;
    type ConfirmSecond = c::ConfirmSecond;
    type GrantedSecond = c::GrantedSecond;
    type ReturnFirst = c::ReturnFirst;
    type ReturnedFirst = c::ReturnedFirst;
    type ReturnSecond = c::ReturnSecond;
    type ReturnedSecond = c::ReturnedSecond;
    type CloseFirst = c::CloseFirst;
    type CloseHandoff = c::CloseHandoff;
    type ClosePrimed = c::ClosePrimed;
    type CloseFirstRequest = c::CloseFirstRequest;
    type CloseSecondRequest = c::CloseSecondRequest;
    type ClosedFirst = c::ClosedFirst;
    type ClosedSecond = c::ClosedSecond;
    type DoneFirst = c::DoneFirst;
    type DoneSecond = c::DoneSecond;
    type AbortFirst = c::AbortFirst;
    type AbortSecond = c::AbortSecond;
    type Finished = c::Finished;
    type Aborted = c::Aborted;
}
pub mod d {
    use hibana::g;
    pub type AcquireSecond = g::Msg<112, ()>;
    pub type AcquireFirst = g::Msg<113, ()>;
    pub type Handoff = g::Msg<114, ()>;
    pub type SecondReady = g::Msg<115, ()>;
    pub type GrantedFirst = g::Msg<116, ()>;
    pub type ConfirmSecond = g::Msg<117, ()>;
    pub type GrantedSecond = g::Msg<118, ()>;
    pub type ReturnFirst = g::Msg<119, ()>;
    pub type ReturnedFirst = g::Msg<120, ()>;
    pub type ReturnSecond = g::Msg<121, ()>;
    pub type ReturnedSecond = g::Msg<122, ()>;
    pub type CloseFirst = g::Msg<123, ()>;
    pub type CloseHandoff = g::Msg<124, ()>;
    pub type ClosePrimed = g::Msg<125, ()>;
    pub type CloseFirstRequest = g::Msg<126, ()>;
    pub type CloseSecondRequest = g::Msg<127, ()>;
    pub type ClosedFirst = g::Msg<128, ()>;
    pub type ClosedSecond = g::Msg<129, ()>;
    pub type DoneFirst = g::Msg<130, ()>;
    pub type DoneSecond = g::Msg<131, ()>;
    pub type AbortFirst = g::Msg<132, ()>;
    pub type AbortSecond = g::Msg<133, ()>;
    pub type Finished = g::Msg<134, ()>;
    pub type Aborted = g::Msg<135, ()>;
}
pub struct LoanD;
impl Messages for LoanD {
    type AcquireSecond = d::AcquireSecond;
    type AcquireFirst = d::AcquireFirst;
    type Handoff = d::Handoff;
    type SecondReady = d::SecondReady;
    type GrantedFirst = d::GrantedFirst;
    type ConfirmSecond = d::ConfirmSecond;
    type GrantedSecond = d::GrantedSecond;
    type ReturnFirst = d::ReturnFirst;
    type ReturnedFirst = d::ReturnedFirst;
    type ReturnSecond = d::ReturnSecond;
    type ReturnedSecond = d::ReturnedSecond;
    type CloseFirst = d::CloseFirst;
    type CloseHandoff = d::CloseHandoff;
    type ClosePrimed = d::ClosePrimed;
    type CloseFirstRequest = d::CloseFirstRequest;
    type CloseSecondRequest = d::CloseSecondRequest;
    type ClosedFirst = d::ClosedFirst;
    type ClosedSecond = d::ClosedSecond;
    type DoneFirst = d::DoneFirst;
    type DoneSecond = d::DoneSecond;
    type AbortFirst = d::AbortFirst;
    type AbortSecond = d::AbortSecond;
    type Finished = d::Finished;
    type Aborted = d::Aborted;
}
pub mod e {
    use hibana::g;
    pub type AcquireSecond = g::Msg<144, ()>;
    pub type AcquireFirst = g::Msg<145, ()>;
    pub type Handoff = g::Msg<146, ()>;
    pub type SecondReady = g::Msg<147, ()>;
    pub type GrantedFirst = g::Msg<148, ()>;
    pub type ConfirmSecond = g::Msg<149, ()>;
    pub type GrantedSecond = g::Msg<150, ()>;
    pub type ReturnFirst = g::Msg<151, ()>;
    pub type ReturnedFirst = g::Msg<152, ()>;
    pub type ReturnSecond = g::Msg<153, ()>;
    pub type ReturnedSecond = g::Msg<154, ()>;
    pub type CloseFirst = g::Msg<155, ()>;
    pub type CloseHandoff = g::Msg<156, ()>;
    pub type ClosePrimed = g::Msg<157, ()>;
    pub type CloseFirstRequest = g::Msg<158, ()>;
    pub type CloseSecondRequest = g::Msg<159, ()>;
    pub type ClosedFirst = g::Msg<160, ()>;
    pub type ClosedSecond = g::Msg<161, ()>;
    pub type DoneFirst = g::Msg<162, ()>;
    pub type DoneSecond = g::Msg<163, ()>;
    pub type AbortFirst = g::Msg<164, ()>;
    pub type AbortSecond = g::Msg<165, ()>;
    pub type Finished = g::Msg<166, ()>;
    pub type Aborted = g::Msg<167, ()>;
}
pub struct LoanE;
impl Messages for LoanE {
    type AcquireSecond = e::AcquireSecond;
    type AcquireFirst = e::AcquireFirst;
    type Handoff = e::Handoff;
    type SecondReady = e::SecondReady;
    type GrantedFirst = e::GrantedFirst;
    type ConfirmSecond = e::ConfirmSecond;
    type GrantedSecond = e::GrantedSecond;
    type ReturnFirst = e::ReturnFirst;
    type ReturnedFirst = e::ReturnedFirst;
    type ReturnSecond = e::ReturnSecond;
    type ReturnedSecond = e::ReturnedSecond;
    type CloseFirst = e::CloseFirst;
    type CloseHandoff = e::CloseHandoff;
    type ClosePrimed = e::ClosePrimed;
    type CloseFirstRequest = e::CloseFirstRequest;
    type CloseSecondRequest = e::CloseSecondRequest;
    type ClosedFirst = e::ClosedFirst;
    type ClosedSecond = e::ClosedSecond;
    type DoneFirst = e::DoneFirst;
    type DoneSecond = e::DoneSecond;
    type AbortFirst = e::AbortFirst;
    type AbortSecond = e::AbortSecond;
    type Finished = e::Finished;
    type Aborted = e::Aborted;
}
pub mod f {
    use hibana::g;
    pub type AcquireSecond = g::Msg<176, ()>;
    pub type AcquireFirst = g::Msg<177, ()>;
    pub type Handoff = g::Msg<178, ()>;
    pub type SecondReady = g::Msg<179, ()>;
    pub type GrantedFirst = g::Msg<180, ()>;
    pub type ConfirmSecond = g::Msg<181, ()>;
    pub type GrantedSecond = g::Msg<182, ()>;
    pub type ReturnFirst = g::Msg<183, ()>;
    pub type ReturnedFirst = g::Msg<184, ()>;
    pub type ReturnSecond = g::Msg<185, ()>;
    pub type ReturnedSecond = g::Msg<186, ()>;
    pub type CloseFirst = g::Msg<187, ()>;
    pub type CloseHandoff = g::Msg<188, ()>;
    pub type ClosePrimed = g::Msg<189, ()>;
    pub type CloseFirstRequest = g::Msg<190, ()>;
    pub type CloseSecondRequest = g::Msg<191, ()>;
    pub type ClosedFirst = g::Msg<192, ()>;
    pub type ClosedSecond = g::Msg<193, ()>;
    pub type DoneFirst = g::Msg<194, ()>;
    pub type DoneSecond = g::Msg<195, ()>;
    pub type AbortFirst = g::Msg<196, ()>;
    pub type AbortSecond = g::Msg<197, ()>;
    pub type Finished = g::Msg<198, ()>;
    pub type Aborted = g::Msg<199, ()>;
}
pub struct LoanF;
impl Messages for LoanF {
    type AcquireSecond = f::AcquireSecond;
    type AcquireFirst = f::AcquireFirst;
    type Handoff = f::Handoff;
    type SecondReady = f::SecondReady;
    type GrantedFirst = f::GrantedFirst;
    type ConfirmSecond = f::ConfirmSecond;
    type GrantedSecond = f::GrantedSecond;
    type ReturnFirst = f::ReturnFirst;
    type ReturnedFirst = f::ReturnedFirst;
    type ReturnSecond = f::ReturnSecond;
    type ReturnedSecond = f::ReturnedSecond;
    type CloseFirst = f::CloseFirst;
    type CloseHandoff = f::CloseHandoff;
    type ClosePrimed = f::ClosePrimed;
    type CloseFirstRequest = f::CloseFirstRequest;
    type CloseSecondRequest = f::CloseSecondRequest;
    type ClosedFirst = f::ClosedFirst;
    type ClosedSecond = f::ClosedSecond;
    type DoneFirst = f::DoneFirst;
    type DoneSecond = f::DoneSecond;
    type AbortFirst = f::AbortFirst;
    type AbortSecond = f::AbortSecond;
    type Finished = f::Finished;
    type Aborted = f::Aborted;
}
pub mod g_messages {
    use hibana::g;
    pub type AcquireSecond = g::Msg<208, ()>;
    pub type AcquireFirst = g::Msg<209, ()>;
    pub type Handoff = g::Msg<210, ()>;
    pub type SecondReady = g::Msg<211, ()>;
    pub type GrantedFirst = g::Msg<212, ()>;
    pub type ConfirmSecond = g::Msg<213, ()>;
    pub type GrantedSecond = g::Msg<214, ()>;
    pub type ReturnFirst = g::Msg<215, ()>;
    pub type ReturnedFirst = g::Msg<216, ()>;
    pub type ReturnSecond = g::Msg<217, ()>;
    pub type ReturnedSecond = g::Msg<218, ()>;
    pub type CloseFirst = g::Msg<219, ()>;
    pub type CloseHandoff = g::Msg<220, ()>;
    pub type ClosePrimed = g::Msg<221, ()>;
    pub type CloseFirstRequest = g::Msg<222, ()>;
    pub type CloseSecondRequest = g::Msg<223, ()>;
    pub type ClosedFirst = g::Msg<224, ()>;
    pub type ClosedSecond = g::Msg<225, ()>;
    pub type DoneFirst = g::Msg<226, ()>;
    pub type DoneSecond = g::Msg<227, ()>;
    pub type AbortFirst = g::Msg<228, ()>;
    pub type AbortSecond = g::Msg<229, ()>;
    pub type Finished = g::Msg<230, ()>;
    pub type Aborted = g::Msg<231, ()>;
}
pub struct LoanG;
impl Messages for LoanG {
    type AcquireSecond = g_messages::AcquireSecond;
    type AcquireFirst = g_messages::AcquireFirst;
    type Handoff = g_messages::Handoff;
    type SecondReady = g_messages::SecondReady;
    type GrantedFirst = g_messages::GrantedFirst;
    type ConfirmSecond = g_messages::ConfirmSecond;
    type GrantedSecond = g_messages::GrantedSecond;
    type ReturnFirst = g_messages::ReturnFirst;
    type ReturnedFirst = g_messages::ReturnedFirst;
    type ReturnSecond = g_messages::ReturnSecond;
    type ReturnedSecond = g_messages::ReturnedSecond;
    type CloseFirst = g_messages::CloseFirst;
    type CloseHandoff = g_messages::CloseHandoff;
    type ClosePrimed = g_messages::ClosePrimed;
    type CloseFirstRequest = g_messages::CloseFirstRequest;
    type CloseSecondRequest = g_messages::CloseSecondRequest;
    type ClosedFirst = g_messages::ClosedFirst;
    type ClosedSecond = g_messages::ClosedSecond;
    type DoneFirst = g_messages::DoneFirst;
    type DoneSecond = g_messages::DoneSecond;
    type AbortFirst = g_messages::AbortFirst;
    type AbortSecond = g_messages::AbortSecond;
    type Finished = g_messages::Finished;
    type Aborted = g_messages::Aborted;
}

fn developer<const DEV: u8, const FIRST: u8, const SECOND: u8, const ID: u16, M: Messages>()
-> impl Projectable {
    let close = g::seq(
        g::send::<DEV, FIRST, M::CloseFirst>(),
        g::seq(
            g::send::<FIRST, SECOND, M::CloseHandoff>(),
            g::seq(
                g::send::<SECOND, FIRST, M::SecondReady>(),
                g::send::<FIRST, DEV, M::ClosePrimed>(),
            ),
        ),
    );
    let acquire = g::seq(
        g::send::<DEV, DEV, Implement>(),
        g::seq(
            g::send::<DEV, FIRST, M::AcquireFirst>(),
            g::seq(
                g::send::<FIRST, SECOND, M::Handoff>(),
                g::seq(
                    g::send::<SECOND, FIRST, M::SecondReady>(),
                    g::send::<FIRST, DEV, M::GrantedFirst>(),
                ),
            ),
        ),
    );
    // Both requests can be sent before any grant. Only reservation/grant order
    // follows increasing physical module IDs, so queued intents cannot form a cycle.
    let inputs = g::par(
        g::send::<DEV, SECOND, M::AcquireSecond>(),
        g::route(close, acquire).resolve::<ID>(),
    );
    let closed = g::seq(
        g::par(
            g::send::<DEV, FIRST, M::CloseFirstRequest>(),
            g::send::<DEV, SECOND, M::CloseSecondRequest>(),
        ),
        g::par(
            g::send::<FIRST, DEV, M::ClosedFirst>(),
            g::send::<SECOND, DEV, M::ClosedSecond>(),
        ),
    );
    let returns = g::par(
        g::seq(
            g::send::<DEV, FIRST, M::ReturnFirst>(),
            g::send::<FIRST, DEV, M::ReturnedFirst>(),
        ),
        g::seq(
            g::send::<DEV, SECOND, M::ReturnSecond>(),
            g::send::<SECOND, DEV, M::ReturnedSecond>(),
        ),
    );
    let work = g::seq(
        g::send::<DEV, SECOND, M::ConfirmSecond>(),
        g::seq(
            g::send::<SECOND, DEV, M::GrantedSecond>(),
            g::seq(
                g::send::<DEV, DEV, Commit>(),
                g::seq(returns, g::send::<DEV, DEV, Test>()),
            ),
        ),
    );
    let body = g::seq(inputs, g::route(closed, work).resolve::<ID>()).roll();
    let finish = g::seq(
        g::par(
            g::send::<DEV, FIRST, M::DoneFirst>(),
            g::send::<DEV, SECOND, M::DoneSecond>(),
        ),
        g::send::<DEV, DEV, M::Finished>(),
    );
    let abort = g::seq(
        g::par(
            g::send::<DEV, FIRST, M::AbortFirst>(),
            g::send::<DEV, SECOND, M::AbortSecond>(),
        ),
        g::send::<DEV, DEV, M::Aborted>(),
    );
    g::seq(
        g::send::<DEV, DEV, Design>(),
        g::seq(body, g::route(finish, abort).resolve::<ID>()),
    )
}
pub struct Programs1H {
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
    let a = developer::<A, AUTH_MODULE, DATABASE_MODULE, TEST_A, LoanA>();
    let b = developer::<B, DATABASE_MODULE, API_MODULE, TEST_B, LoanB>();
    let c = developer::<C, API_MODULE, PAYMENT_MODULE, TEST_C, LoanC>();
    let d = developer::<D, PAYMENT_MODULE, NOTIFY_MODULE, TEST_D, LoanD>();
    let e = developer::<E, NOTIFY_MODULE, ANALYTICS_MODULE, TEST_E, LoanE>();
    let f = developer::<F, ANALYTICS_MODULE, SEARCH_MODULE, TEST_F, LoanF>();
    let g = developer::<G, AUTH_MODULE, SEARCH_MODULE, TEST_G, LoanG>();
    let global = g::par(g::par(a, g::par(b, c)), g::par(g::par(d, e), g::par(f, g)));
    Programs1H {
        a: project(&global),
        b: project(&global),
        c: project(&global),
        d: project(&global),
        e: project(&global),
        f: project(&global),
        g: project(&global),
        auth: project(&global),
        database: project(&global),
        api: project(&global),
        payment: project(&global),
        notify: project(&global),
        analytics: project(&global),
        search: project(&global),
    }
}
