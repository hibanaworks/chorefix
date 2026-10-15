//! All researchers and physical sections belong to one global/session.
//! Local projection retains independent preparation and disjoint shared writes.
use hibana::{
    g,
    runtime::program::{Projectable, RoleProgram, project},
};

pub type Literature = g::Msg<1, ()>;
pub type Draft = g::Msg<2, ()>;
pub type Write = g::Msg<3, ()>;
pub type Review = g::Msg<4, ()>;
pub type Finished = g::Msg<5, ()>;

pub(crate) trait SectionMessages {
    type Available: g::Message<Payload = ()>;
    type Acquire: g::Message<Payload = ()>;
    type Granted: g::Message<Payload = ()>;
    type Return: g::Message<Payload = ()>;
    type Returned: g::Message<Payload = ()>;
}
pub(crate) struct Intro;
impl SectionMessages for Intro {
    type Available = g::Msg<80, ()>;
    type Acquire = g::Msg<10, ()>;
    type Granted = g::Msg<11, ()>;
    type Return = g::Msg<12, ()>;
    type Returned = g::Msg<13, ()>;
}
pub(crate) struct Background;
impl SectionMessages for Background {
    type Available = g::Msg<81, ()>;
    type Acquire = g::Msg<14, ()>;
    type Granted = g::Msg<15, ()>;
    type Return = g::Msg<16, ()>;
    type Returned = g::Msg<17, ()>;
}
pub(crate) struct Related;
impl SectionMessages for Related {
    type Available = g::Msg<82, ()>;
    type Acquire = g::Msg<18, ()>;
    type Granted = g::Msg<19, ()>;
    type Return = g::Msg<20, ()>;
    type Returned = g::Msg<21, ()>;
}
pub(crate) struct Methods;
impl SectionMessages for Methods {
    type Available = g::Msg<83, ()>;
    type Acquire = g::Msg<22, ()>;
    type Granted = g::Msg<23, ()>;
    type Return = g::Msg<24, ()>;
    type Returned = g::Msg<25, ()>;
}
pub(crate) struct Experiments;
impl SectionMessages for Experiments {
    type Available = g::Msg<84, ()>;
    type Acquire = g::Msg<26, ()>;
    type Granted = g::Msg<27, ()>;
    type Return = g::Msg<28, ()>;
    type Returned = g::Msg<29, ()>;
}
pub(crate) struct Results;
impl SectionMessages for Results {
    type Available = g::Msg<85, ()>;
    type Acquire = g::Msg<30, ()>;
    type Granted = g::Msg<31, ()>;
    type Return = g::Msg<32, ()>;
    type Returned = g::Msg<33, ()>;
}
pub(crate) struct Discussion;
impl SectionMessages for Discussion {
    type Available = g::Msg<86, ()>;
    type Acquire = g::Msg<34, ()>;
    type Granted = g::Msg<35, ()>;
    type Return = g::Msg<36, ()>;
    type Returned = g::Msg<37, ()>;
}

fn prepare<const ROLE: u8>() -> impl Projectable {
    g::seq(
        g::send::<ROLE, ROLE, Literature>(),
        g::send::<ROLE, ROLE, Draft>(),
    )
}
fn write<
    const ROLE: u8,
    const FIRST: u8,
    const SECOND: u8,
    F: SectionMessages,
    S: SectionMessages,
>() -> impl Projectable {
    let acquire = g::par(
        g::seq(
            g::send::<FIRST, ROLE, F::Available>(),
            g::seq(
                g::send::<ROLE, FIRST, F::Acquire>(),
                g::send::<FIRST, ROLE, F::Granted>(),
            ),
        ),
        g::seq(
            g::send::<SECOND, ROLE, S::Available>(),
            g::seq(
                g::send::<ROLE, SECOND, S::Acquire>(),
                g::send::<SECOND, ROLE, S::Granted>(),
            ),
        ),
    );
    let release = g::par(
        g::seq(
            g::send::<ROLE, SECOND, S::Return>(),
            g::send::<SECOND, ROLE, S::Returned>(),
        ),
        g::seq(
            g::send::<ROLE, FIRST, F::Return>(),
            g::send::<FIRST, ROLE, F::Returned>(),
        ),
    );
    g::seq(
        acquire,
        g::seq(
            g::send::<ROLE, ROLE, Write>(),
            g::seq(
                release,
                g::seq(
                    g::send::<ROLE, ROLE, Review>(),
                    g::send::<ROLE, ROLE, Finished>(),
                ),
            ),
        ),
    )
}

pub mod roles2e {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const INTRO: u8 = 7;
    pub const METHODS: u8 = 8;
    pub const RESULTS: u8 = 9;
}
pub struct Programs2E {
    pub a: RoleProgram<{ roles2e::A }>,
    pub b: RoleProgram<{ roles2e::B }>,
    pub c: RoleProgram<{ roles2e::C }>,
    pub intro: RoleProgram<{ roles2e::INTRO }>,
    pub methods: RoleProgram<{ roles2e::METHODS }>,
    pub results: RoleProgram<{ roles2e::RESULTS }>,
}
pub fn project_2e() -> Programs2E {
    use roles2e::*;
    let preparation = g::par(prepare::<A>(), g::par(prepare::<B>(), prepare::<C>()));
    let writes_0 = write::<A, INTRO, METHODS, Intro, Methods>();
    let writes_1 = write::<B, METHODS, RESULTS, Methods, Results>();
    let writes_2 = write::<C, INTRO, RESULTS, Intro, Results>();
    let protocol = g::seq(preparation, g::seq(writes_0, g::seq(writes_1, writes_2)));
    Programs2E {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        intro: project(&protocol),
        methods: project(&protocol),
        results: project(&protocol),
    }
}

pub mod roles2m {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const BACKGROUND: u8 = 7;
    pub const INTRO: u8 = 8;
    pub const METHODS: u8 = 9;
    pub const EXPERIMENTS: u8 = 10;
    pub const RESULTS: u8 = 11;
}
pub struct Programs2M {
    pub a: RoleProgram<{ roles2m::A }>,
    pub b: RoleProgram<{ roles2m::B }>,
    pub c: RoleProgram<{ roles2m::C }>,
    pub d: RoleProgram<{ roles2m::D }>,
    pub e: RoleProgram<{ roles2m::E }>,
    pub background: RoleProgram<{ roles2m::BACKGROUND }>,
    pub intro: RoleProgram<{ roles2m::INTRO }>,
    pub methods: RoleProgram<{ roles2m::METHODS }>,
    pub experiments: RoleProgram<{ roles2m::EXPERIMENTS }>,
    pub results: RoleProgram<{ roles2m::RESULTS }>,
}
pub fn project_2m() -> Programs2M {
    use roles2m::*;
    let preparation = g::par(
        g::par(prepare::<A>(), prepare::<B>()),
        g::par(prepare::<C>(), g::par(prepare::<D>(), prepare::<E>())),
    );
    let writes_0 = g::par(
        write::<A, BACKGROUND, INTRO, Background, Intro>(),
        write::<C, EXPERIMENTS, METHODS, Experiments, Methods>(),
    );
    let writes_1 = g::par(
        write::<B, BACKGROUND, METHODS, Background, Methods>(),
        write::<D, EXPERIMENTS, RESULTS, Experiments, Results>(),
    );
    let writes_2 = write::<E, INTRO, RESULTS, Intro, Results>();
    let protocol = g::seq(preparation, g::seq(writes_0, g::seq(writes_1, writes_2)));
    Programs2M {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        background: project(&protocol),
        intro: project(&protocol),
        methods: project(&protocol),
        experiments: project(&protocol),
        results: project(&protocol),
    }
}

pub mod roles2h {
    pub const A: u8 = 0;
    pub const B: u8 = 1;
    pub const C: u8 = 2;
    pub const D: u8 = 3;
    pub const E: u8 = 4;
    pub const F: u8 = 5;
    pub const G: u8 = 6;
    pub const BACKGROUND: u8 = 7;
    pub const INTRO: u8 = 8;
    pub const RELATED: u8 = 9;
    pub const METHODS: u8 = 10;
    pub const EXPERIMENTS: u8 = 11;
    pub const RESULTS: u8 = 12;
    pub const DISCUSSION: u8 = 13;
}
pub struct Programs2H {
    pub a: RoleProgram<{ roles2h::A }>,
    pub b: RoleProgram<{ roles2h::B }>,
    pub c: RoleProgram<{ roles2h::C }>,
    pub d: RoleProgram<{ roles2h::D }>,
    pub e: RoleProgram<{ roles2h::E }>,
    pub f: RoleProgram<{ roles2h::F }>,
    pub g: RoleProgram<{ roles2h::G }>,
    pub background: RoleProgram<{ roles2h::BACKGROUND }>,
    pub intro: RoleProgram<{ roles2h::INTRO }>,
    pub related: RoleProgram<{ roles2h::RELATED }>,
    pub methods: RoleProgram<{ roles2h::METHODS }>,
    pub experiments: RoleProgram<{ roles2h::EXPERIMENTS }>,
    pub results: RoleProgram<{ roles2h::RESULTS }>,
    pub discussion: RoleProgram<{ roles2h::DISCUSSION }>,
}
pub fn project_2h() -> Programs2H {
    use roles2h::*;
    let preparation = g::par(
        g::par(prepare::<A>(), g::par(prepare::<B>(), prepare::<C>())),
        g::par(
            g::par(prepare::<D>(), prepare::<E>()),
            g::par(prepare::<F>(), prepare::<G>()),
        ),
    );
    let writes_0 = g::par(
        write::<A, BACKGROUND, INTRO, Background, Intro>(),
        g::par(
            write::<C, METHODS, RELATED, Methods, Related>(),
            write::<E, EXPERIMENTS, RESULTS, Experiments, Results>(),
        ),
    );
    let writes_1 = g::par(
        write::<B, BACKGROUND, RELATED, Background, Related>(),
        g::par(
            write::<D, EXPERIMENTS, METHODS, Experiments, Methods>(),
            write::<F, DISCUSSION, RESULTS, Discussion, Results>(),
        ),
    );
    let writes_2 = write::<G, DISCUSSION, INTRO, Discussion, Intro>();
    let protocol = g::seq(preparation, g::seq(writes_0, g::seq(writes_1, writes_2)));
    Programs2H {
        a: project(&protocol),
        b: project(&protocol),
        c: project(&protocol),
        d: project(&protocol),
        e: project(&protocol),
        f: project(&protocol),
        g: project(&protocol),
        background: project(&protocol),
        intro: project(&protocol),
        related: project(&protocol),
        methods: project(&protocol),
        experiments: project(&protocol),
        results: project(&protocol),
        discussion: project(&protocol),
    }
}
