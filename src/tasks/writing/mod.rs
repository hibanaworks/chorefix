pub mod global;
mod imp;
mod localside;

use crate::{RunError, imp::transport::InMemoryTransport};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
pub use imp::{Domain, Operation, Request};

/// All 3 original researchers and 3 sections share one projected session.
pub async fn run_2e<Work: Domain>(domain: &Work) -> Result<(), RunError<String>> {
    use global::{roles2e::*, *};
    let programs = project_2e();
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut c = rendezvous
        .enter(session, &programs.c)
        .map_err(RunError::Attach)?;
    let mut intro = rendezvous
        .enter(session, &programs.intro)
        .map_err(RunError::Attach)?;
    let mut methods = rendezvous
        .enter(session, &programs.methods)
        .map_err(RunError::Attach)?;
    let mut results = rendezvous
        .enter(session, &programs.results)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::researcher::run::<A, Intro, Methods, Work>("RESEARCHER_A", &mut a, domain),
        localside::researcher::run::<B, Methods, Results, Work>("RESEARCHER_B", &mut b, domain),
        localside::researcher::run::<C, Intro, Results, Work>("RESEARCHER_C", &mut c, domain),
        localside::section::run::<INTRO, Intro>(&mut intro),
        localside::section::run::<METHODS, Methods>(&mut methods),
        localside::section::run::<RESULTS, Results>(&mut results),
    )?;
    Ok(())
}

/// All 5 original researchers and 5 sections share one projected session.
pub async fn run_2m<Work: Domain>(domain: &Work) -> Result<(), RunError<String>> {
    use global::{roles2m::*, *};
    let programs = project_2m();
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut c = rendezvous
        .enter(session, &programs.c)
        .map_err(RunError::Attach)?;
    let mut d = rendezvous
        .enter(session, &programs.d)
        .map_err(RunError::Attach)?;
    let mut e = rendezvous
        .enter(session, &programs.e)
        .map_err(RunError::Attach)?;
    let mut background = rendezvous
        .enter(session, &programs.background)
        .map_err(RunError::Attach)?;
    let mut intro = rendezvous
        .enter(session, &programs.intro)
        .map_err(RunError::Attach)?;
    let mut methods = rendezvous
        .enter(session, &programs.methods)
        .map_err(RunError::Attach)?;
    let mut experiments = rendezvous
        .enter(session, &programs.experiments)
        .map_err(RunError::Attach)?;
    let mut results = rendezvous
        .enter(session, &programs.results)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::researcher::run::<A, Background, Intro, Work>("RESEARCHER_A", &mut a, domain),
        localside::researcher::run::<B, Background, Methods, Work>("RESEARCHER_B", &mut b, domain),
        localside::researcher::run::<C, Experiments, Methods, Work>("RESEARCHER_C", &mut c, domain),
        localside::researcher::run::<D, Experiments, Results, Work>("RESEARCHER_D", &mut d, domain),
        localside::researcher::run::<E, Intro, Results, Work>("RESEARCHER_E", &mut e, domain),
        localside::section::run::<BACKGROUND, Background>(&mut background),
        localside::section::run::<INTRO, Intro>(&mut intro),
        localside::section::run::<METHODS, Methods>(&mut methods),
        localside::section::run::<EXPERIMENTS, Experiments>(&mut experiments),
        localside::section::run::<RESULTS, Results>(&mut results),
    )?;
    Ok(())
}

/// All 7 original researchers and 7 sections share one projected session.
pub async fn run_2h<Work: Domain>(domain: &Work) -> Result<(), RunError<String>> {
    use global::{roles2h::*, *};
    let programs = project_2h();
    let mut slab = vec![0; 192 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, InMemoryTransport::new())
        .map_err(RunError::Attach)?;
    let session = SessionId::new(1);
    let mut a = rendezvous
        .enter(session, &programs.a)
        .map_err(RunError::Attach)?;
    let mut b = rendezvous
        .enter(session, &programs.b)
        .map_err(RunError::Attach)?;
    let mut c = rendezvous
        .enter(session, &programs.c)
        .map_err(RunError::Attach)?;
    let mut d = rendezvous
        .enter(session, &programs.d)
        .map_err(RunError::Attach)?;
    let mut e = rendezvous
        .enter(session, &programs.e)
        .map_err(RunError::Attach)?;
    let mut f = rendezvous
        .enter(session, &programs.f)
        .map_err(RunError::Attach)?;
    let mut g = rendezvous
        .enter(session, &programs.g)
        .map_err(RunError::Attach)?;
    let mut background = rendezvous
        .enter(session, &programs.background)
        .map_err(RunError::Attach)?;
    let mut intro = rendezvous
        .enter(session, &programs.intro)
        .map_err(RunError::Attach)?;
    let mut related = rendezvous
        .enter(session, &programs.related)
        .map_err(RunError::Attach)?;
    let mut methods = rendezvous
        .enter(session, &programs.methods)
        .map_err(RunError::Attach)?;
    let mut experiments = rendezvous
        .enter(session, &programs.experiments)
        .map_err(RunError::Attach)?;
    let mut results = rendezvous
        .enter(session, &programs.results)
        .map_err(RunError::Attach)?;
    let mut discussion = rendezvous
        .enter(session, &programs.discussion)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::researcher::run::<A, Background, Intro, Work>("RESEARCHER_A", &mut a, domain),
        localside::researcher::run::<B, Background, Related, Work>("RESEARCHER_B", &mut b, domain),
        localside::researcher::run::<C, Methods, Related, Work>("RESEARCHER_C", &mut c, domain),
        localside::researcher::run::<D, Experiments, Methods, Work>("RESEARCHER_D", &mut d, domain),
        localside::researcher::run::<E, Experiments, Results, Work>("RESEARCHER_E", &mut e, domain),
        localside::researcher::run::<F, Discussion, Results, Work>("RESEARCHER_F", &mut f, domain),
        localside::researcher::run::<G, Discussion, Intro, Work>("RESEARCHER_G", &mut g, domain),
        localside::section::run::<BACKGROUND, Background>(&mut background),
        localside::section::run::<INTRO, Intro>(&mut intro),
        localside::section::run::<RELATED, Related>(&mut related),
        localside::section::run::<METHODS, Methods>(&mut methods),
        localside::section::run::<EXPERIMENTS, Experiments>(&mut experiments),
        localside::section::run::<RESULTS, Results>(&mut results),
        localside::section::run::<DISCUSSION, Discussion>(&mut discussion),
    )?;
    Ok(())
}
