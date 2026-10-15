//! Actual callers, materializers and physical sections belong to one session.
pub mod global;
pub(crate) mod localside;
use crate::shell::{
    imp::private_fs::{PrivateFs, bindings},
    localside::private_io,
};
use crate::{
    imp::transport::InMemoryTransport,
    shell::{
        ShellError, ShellResult,
        imp::{
            bus::{Bus, Terminal},
            service::Validation,
        },
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};
use hibana_wasip1_runtime::{DEFAULT_GUEST_MEMORY_BYTES, GuestMemory, HibanaWasiGuestStorage};
use localside::researcher::Assignment;
use std::{cell::RefCell, rc::Rc};

async fn run_2e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use crate::tasks::writing::global::{Intro, Methods, Results};
    use global::{roles2e::*, *};
    let programs = project_2e();
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&acheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&bcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&ccheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(18);
    let mut a = rv.enter(session, &programs.a)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut intro = rv.enter(session, &programs.intro)?;
    let mut methods = rv.enter(session, &programs.methods)?;
    let mut results = rv.enter(session, &programs.results)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let [aterm, bterm, cterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    tokio::try_join!(
        localside::researcher::run::<A, Intro, Methods, _>(
            &mut a,
            bus,
            &acheck,
            &Assignment {
                agent: "RESEARCHER_A",
                topic: "intro_methods_connection",
                section: "sec_intro_methods",
                sections: ["INTRO_SECTION", "METHODS_SECTION"]
            }
        ),
        localside::researcher::run::<B, Methods, Results, _>(
            &mut b,
            bus,
            &bcheck,
            &Assignment {
                agent: "RESEARCHER_B",
                topic: "methods_results_connection",
                section: "sec_methods_results",
                sections: ["METHODS_SECTION", "RESULTS_SECTION"]
            }
        ),
        localside::researcher::run::<C, Intro, Results, _>(
            &mut c,
            bus,
            &ccheck,
            &Assignment {
                agent: "RESEARCHER_C",
                topic: "results_intro_analysis",
                section: "sec_results_intro",
                sections: ["INTRO_SECTION", "RESULTS_SECTION"]
            }
        ),
        localside::section::run::<INTRO, Intro>(&mut intro),
        localside::section::run::<METHODS, Methods>(&mut methods),
        localside::section::run::<RESULTS, Results>(&mut results),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut a_wasi, abytes),
                private_io::run(&mut a_io, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher a guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut b_wasi, bbytes),
                private_io::run(&mut b_io, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher b guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut c_wasi, cbytes),
                private_io::run(&mut c_io, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher c guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

async fn run_2m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use crate::tasks::writing::global::{Background, Experiments, Intro, Methods, Results};
    use global::{roles2m::*, *};
    let programs = project_2m();
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let dcheck = Validation::new();
    let echeck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&acheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&bcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&ccheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<CHECK_D>::decision_state(&dcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<CHECK_E>::decision_state(&echeck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(18);
    let mut a = rv.enter(session, &programs.a)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut background = rv.enter(session, &programs.background)?;
    let mut intro = rv.enter(session, &programs.intro)?;
    let mut methods = rv.enter(session, &programs.methods)?;
    let mut experiments = rv.enter(session, &programs.experiments)?;
    let mut results = rv.enter(session, &programs.results)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let [aterm, bterm, cterm, dterm, eterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let dbytes = Rc::new(RefCell::new(Vec::new()));
    let mut dfs = PrivateFs::new(dterm, bus, dbytes.clone());
    let mut dmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let ebytes = Rc::new(RefCell::new(Vec::new()));
    let mut efs = PrivateFs::new(eterm, bus, ebytes.clone());
    let mut ememory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    tokio::try_join!(
        localside::researcher::run::<A, Background, Intro, _>(
            &mut a,
            bus,
            &acheck,
            &Assignment {
                agent: "RESEARCHER_A",
                topic: "intro_background_connection",
                section: "sec_intro_background",
                sections: ["BACKGROUND_SECTION", "INTRO_SECTION"]
            }
        ),
        localside::researcher::run::<B, Background, Methods, _>(
            &mut b,
            bus,
            &bcheck,
            &Assignment {
                agent: "RESEARCHER_B",
                topic: "background_methods_connection",
                section: "sec_background_methods",
                sections: ["BACKGROUND_SECTION", "METHODS_SECTION"]
            }
        ),
        localside::researcher::run::<C, Experiments, Methods, _>(
            &mut c,
            bus,
            &ccheck,
            &Assignment {
                agent: "RESEARCHER_C",
                topic: "methods_experiments_connection",
                section: "sec_methods_experiments",
                sections: ["EXPERIMENTS_SECTION", "METHODS_SECTION"]
            }
        ),
        localside::researcher::run::<D, Experiments, Results, _>(
            &mut d,
            bus,
            &dcheck,
            &Assignment {
                agent: "RESEARCHER_D",
                topic: "experiments_results_connection",
                section: "sec_experiments_results",
                sections: ["EXPERIMENTS_SECTION", "RESULTS_SECTION"]
            }
        ),
        localside::researcher::run::<E, Intro, Results, _>(
            &mut e,
            bus,
            &echeck,
            &Assignment {
                agent: "RESEARCHER_E",
                topic: "results_intro_analysis",
                section: "sec_results_intro",
                sections: ["INTRO_SECTION", "RESULTS_SECTION"]
            }
        ),
        localside::section::run::<BACKGROUND, Background>(&mut background),
        localside::section::run::<INTRO, Intro>(&mut intro),
        localside::section::run::<METHODS, Methods>(&mut methods),
        localside::section::run::<EXPERIMENTS, Experiments>(&mut experiments),
        localside::section::run::<RESULTS, Results>(&mut results),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut a_wasi, abytes),
                private_io::run(&mut a_io, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher a guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut b_wasi, bbytes),
                private_io::run(&mut b_io, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher b guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut c_wasi, cbytes),
                private_io::run(&mut c_io, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher c guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(dguest, &mut d_wasi, dbytes),
                private_io::run(&mut d_io, &mut dfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher d guest/IO status differs"));
            }
            dfs.terminal
                .finish(bus, vm, std::mem::take(&mut dfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(eguest, &mut e_wasi, ebytes),
                private_io::run(&mut e_io, &mut efs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher e guest/IO status differs"));
            }
            efs.terminal
                .finish(bus, vm, std::mem::take(&mut efs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

async fn run_2h_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use crate::tasks::writing::global::{
        Background, Discussion, Experiments, Intro, Methods, Related, Results,
    };
    use global::{roles2h::*, *};
    let programs = project_2h();
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let dcheck = Validation::new();
    let echeck = Validation::new();
    let fcheck = Validation::new();
    let gcheck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, bus.transport.clone())?;
    rv.set_resolver(
        &programs.a,
        ResolverRef::<CHECK_A>::decision_state(&acheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.b,
        ResolverRef::<CHECK_B>::decision_state(&bcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.c,
        ResolverRef::<CHECK_C>::decision_state(&ccheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.d,
        ResolverRef::<CHECK_D>::decision_state(&dcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.e,
        ResolverRef::<CHECK_E>::decision_state(&echeck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.f,
        ResolverRef::<CHECK_F>::decision_state(&fcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    rv.set_resolver(
        &programs.g,
        ResolverRef::<CHECK_G>::decision_state(&gcheck, Validation::decide),
    )
    .map_err(|e| ShellError::message(format!("{e:?}")))?;
    let session = SessionId::new(18);
    let mut a = rv.enter(session, &programs.a)?;
    let mut b = rv.enter(session, &programs.b)?;
    let mut c = rv.enter(session, &programs.c)?;
    let mut d = rv.enter(session, &programs.d)?;
    let mut e = rv.enter(session, &programs.e)?;
    let mut f = rv.enter(session, &programs.f)?;
    let mut g = rv.enter(session, &programs.g)?;
    let mut background = rv.enter(session, &programs.background)?;
    let mut intro = rv.enter(session, &programs.intro)?;
    let mut related = rv.enter(session, &programs.related)?;
    let mut methods = rv.enter(session, &programs.methods)?;
    let mut experiments = rv.enter(session, &programs.experiments)?;
    let mut results = rv.enter(session, &programs.results)?;
    let mut discussion = rv.enter(session, &programs.discussion)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut d_wasi = rv.enter(session, &programs.d_wasi)?;
    let mut e_wasi = rv.enter(session, &programs.e_wasi)?;
    let mut f_wasi = rv.enter(session, &programs.f_wasi)?;
    let mut g_wasi = rv.enter(session, &programs.g_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let mut d_io = rv.enter(session, &programs.d_io)?;
    let mut e_io = rv.enter(session, &programs.e_io)?;
    let mut f_io = rv.enter(session, &programs.f_io)?;
    let mut g_io = rv.enter(session, &programs.g_io)?;
    let [aterm, bterm, cterm, dterm, eterm, fterm, gterm] = terminals;
    let abytes = Rc::new(RefCell::new(Vec::new()));
    let mut afs = PrivateFs::new(aterm, bus, abytes.clone());
    let mut amemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bbytes = Rc::new(RefCell::new(Vec::new()));
    let mut bfs = PrivateFs::new(bterm, bus, bbytes.clone());
    let mut bmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cbytes = Rc::new(RefCell::new(Vec::new()));
    let mut cfs = PrivateFs::new(cterm, bus, cbytes.clone());
    let mut cmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let dbytes = Rc::new(RefCell::new(Vec::new()));
    let mut dfs = PrivateFs::new(dterm, bus, dbytes.clone());
    let mut dmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let ebytes = Rc::new(RefCell::new(Vec::new()));
    let mut efs = PrivateFs::new(eterm, bus, ebytes.clone());
    let mut ememory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    let fbytes = Rc::new(RefCell::new(Vec::new()));
    let mut ffs = PrivateFs::new(fterm, bus, fbytes.clone());
    let mut fmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut fstore = HibanaWasiGuestStorage::uninit();
    let fguest = fstore.init(module, GuestMemory::new(&mut fmemory), bindings())?;
    let gbytes = Rc::new(RefCell::new(Vec::new()));
    let mut gfs = PrivateFs::new(gterm, bus, gbytes.clone());
    let mut gmemory = vec![0; 16 * DEFAULT_GUEST_MEMORY_BYTES];
    let mut gstore = HibanaWasiGuestStorage::uninit();
    let gguest = gstore.init(module, GuestMemory::new(&mut gmemory), bindings())?;
    tokio::try_join!(
        localside::researcher::run::<A, Background, Intro, _>(
            &mut a,
            bus,
            &acheck,
            &Assignment {
                agent: "RESEARCHER_A",
                topic: "intro_background_connection",
                section: "sec_intro_background",
                sections: ["BACKGROUND_SECTION", "INTRO_SECTION"]
            }
        ),
        localside::researcher::run::<B, Background, Related, _>(
            &mut b,
            bus,
            &bcheck,
            &Assignment {
                agent: "RESEARCHER_B",
                topic: "background_related_connection",
                section: "sec_background_related",
                sections: ["BACKGROUND_SECTION", "RELATED_SECTION"]
            }
        ),
        localside::researcher::run::<C, Methods, Related, _>(
            &mut c,
            bus,
            &ccheck,
            &Assignment {
                agent: "RESEARCHER_C",
                topic: "related_methods_connection",
                section: "sec_related_methods",
                sections: ["METHODS_SECTION", "RELATED_SECTION"]
            }
        ),
        localside::researcher::run::<D, Experiments, Methods, _>(
            &mut d,
            bus,
            &dcheck,
            &Assignment {
                agent: "RESEARCHER_D",
                topic: "methods_experiments_connection",
                section: "sec_methods_experiments",
                sections: ["EXPERIMENTS_SECTION", "METHODS_SECTION"]
            }
        ),
        localside::researcher::run::<E, Experiments, Results, _>(
            &mut e,
            bus,
            &echeck,
            &Assignment {
                agent: "RESEARCHER_E",
                topic: "experiments_results_connection",
                section: "sec_experiments_results",
                sections: ["EXPERIMENTS_SECTION", "RESULTS_SECTION"]
            }
        ),
        localside::researcher::run::<F, Discussion, Results, _>(
            &mut f,
            bus,
            &fcheck,
            &Assignment {
                agent: "RESEARCHER_F",
                topic: "results_discussion_connection",
                section: "sec_results_discussion",
                sections: ["DISCUSSION_SECTION", "RESULTS_SECTION"]
            }
        ),
        localside::researcher::run::<G, Discussion, Intro, _>(
            &mut g,
            bus,
            &gcheck,
            &Assignment {
                agent: "RESEARCHER_G",
                topic: "discussion_intro_connection",
                section: "sec_discussion_intro",
                sections: ["DISCUSSION_SECTION", "INTRO_SECTION"]
            }
        ),
        localside::section::run::<BACKGROUND, Background>(&mut background),
        localside::section::run::<INTRO, Intro>(&mut intro),
        localside::section::run::<RELATED, Related>(&mut related),
        localside::section::run::<METHODS, Methods>(&mut methods),
        localside::section::run::<EXPERIMENTS, Experiments>(&mut experiments),
        localside::section::run::<RESULTS, Results>(&mut results),
        localside::section::run::<DISCUSSION, Discussion>(&mut discussion),
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(aguest, &mut a_wasi, abytes),
                private_io::run(&mut a_io, &mut afs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher a guest/IO status differs"));
            }
            afs.terminal
                .finish(bus, vm, std::mem::take(&mut afs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(bguest, &mut b_wasi, bbytes),
                private_io::run(&mut b_io, &mut bfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher b guest/IO status differs"));
            }
            bfs.terminal
                .finish(bus, vm, std::mem::take(&mut bfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(cguest, &mut c_wasi, cbytes),
                private_io::run(&mut c_io, &mut cfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher c guest/IO status differs"));
            }
            cfs.terminal
                .finish(bus, vm, std::mem::take(&mut cfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(dguest, &mut d_wasi, dbytes),
                private_io::run(&mut d_io, &mut dfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher d guest/IO status differs"));
            }
            dfs.terminal
                .finish(bus, vm, std::mem::take(&mut dfs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(eguest, &mut e_wasi, ebytes),
                private_io::run(&mut e_io, &mut efs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher e guest/IO status differs"));
            }
            efs.terminal
                .finish(bus, vm, std::mem::take(&mut efs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(fguest, &mut f_wasi, fbytes),
                private_io::run(&mut f_io, &mut ffs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher f guest/IO status differs"));
            }
            ffs.terminal
                .finish(bus, vm, std::mem::take(&mut ffs.output))
                .await?;
            Ok(())
        },
        async {
            let (vm, io) = tokio::try_join!(
                localside::wasi::run(gguest, &mut g_wasi, gbytes),
                private_io::run(&mut g_io, &mut gfs)
            )?;
            if vm != io {
                return Err(ShellError::message("researcher g guest/IO status differs"));
            }
            gfs.terminal
                .finish(bus, vm, std::mem::take(&mut gfs.output))
                .await?;
            Ok(())
        },
    )?;
    Ok(())
}

pub async fn run_stdio(module: &[u8], level: &str) -> ShellResult<()> {
    match level {
        "2E" => {
            let (bus, mut reader, terminals) =
                Bus::open(["RESEARCHER_A", "RESEARCHER_B", "RESEARCHER_C"]);
            let result = tokio::select! {result = run_2e_guests(module, &bus, terminals) => result, result = reader.run(&bus) => result};
            match &result {
Ok(()) => bus.emit(serde_json::json!({"type":"run_result", "status":"complete"})).await?,
Err(error) => bus.emit(serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
}
            result
        }
        "2M" => {
            let (bus, mut reader, terminals) = Bus::open([
                "RESEARCHER_A",
                "RESEARCHER_B",
                "RESEARCHER_C",
                "RESEARCHER_D",
                "RESEARCHER_E",
            ]);
            let result = tokio::select! {result = run_2m_guests(module, &bus, terminals) => result, result = reader.run(&bus) => result};
            match &result {
Ok(()) => bus.emit(serde_json::json!({"type":"run_result", "status":"complete"})).await?,
Err(error) => bus.emit(serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
}
            result
        }
        "2H" => {
            let (bus, mut reader, terminals) = Bus::open([
                "RESEARCHER_A",
                "RESEARCHER_B",
                "RESEARCHER_C",
                "RESEARCHER_D",
                "RESEARCHER_E",
                "RESEARCHER_F",
                "RESEARCHER_G",
            ]);
            let result = tokio::select! {result = run_2h_guests(module, &bus, terminals) => result, result = reader.run(&bus) => result};
            match &result {
Ok(()) => bus.emit(serde_json::json!({"type":"run_result", "status":"complete"})).await?,
Err(error) => bus.emit(serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
}
            result
        }
        _ => Err(ShellError::message("expected writing task 2E, 2M or 2H")),
    }
}
