//! Actual callers, materializers and physical sections belong to one session.
pub mod global;
mod imp;
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
use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
use localside::researcher::Assignment;

#[hibana_runtime::localside]
async fn run_2e_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 3]) -> ShellResult<()> {
    use global::{roles2e::*, *};
    let programs = project_2e();
    bus.observe(programs.observation());
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
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
    let intro_resource = rv.enter(session, &programs.intro_resource)?;
    let mut methods = rv.enter(session, &programs.methods)?;
    let methods_resource = rv.enter(session, &programs.methods_resource)?;
    let mut results = rv.enter(session, &programs.results)?;
    let results_resource = rv.enter(session, &programs.results_resource)?;
    let mut a_wasi = rv.enter(session, &programs.a_wasi)?;
    let mut b_wasi = rv.enter(session, &programs.b_wasi)?;
    let mut c_wasi = rv.enter(session, &programs.c_wasi)?;
    let mut a_io = rv.enter(session, &programs.a_io)?;
    let mut b_io = rv.enter(session, &programs.b_io)?;
    let mut c_io = rv.enter(session, &programs.c_io)?;
    let [aterm, bterm, cterm] = terminals;
    let afs = PrivateFs::new(aterm, bus);
    let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bfs = PrivateFs::new(bterm, bus);
    let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cfs = PrivateFs::new(cterm, bus);
    let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    async {
        localside::researcher::run::<A, INTRO_RESOURCE, METHODS_RESOURCE>(
            &mut a,
            bus,
            (
                &acheck,
                &Assignment {
                    agent: "RESEARCHER_A",
                    topic: "intro_methods_connection",
                    section: "sec_intro_methods",
                    sections: ["INTRO_SECTION", "METHODS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<B, METHODS_RESOURCE, RESULTS_RESOURCE>(
            &mut b,
            bus,
            (
                &bcheck,
                &Assignment {
                    agent: "RESEARCHER_B",
                    topic: "methods_results_connection",
                    section: "sec_methods_results",
                    sections: ["METHODS_SECTION", "RESULTS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<C, INTRO_RESOURCE, RESULTS_RESOURCE>(
            &mut c,
            bus,
            (
                &ccheck,
                &Assignment {
                    agent: "RESEARCHER_C",
                    topic: "results_intro_analysis",
                    section: "sec_results_intro",
                    sections: ["INTRO_SECTION", "RESULTS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::section::run(&mut intro, intro_resource).await?;
    };
    async {
        localside::section::run(&mut methods, methods_resource).await?;
    };
    async {
        localside::section::run(&mut results, results_resource).await?;
    };
    async {
        localside::wasi::run::<A_WASI, INTRO_RESOURCE, METHODS_RESOURCE>(aguest, &mut a_wasi)
            .await?;
    };
    async {
        private_io::run(&mut a_io, afs).await?;
    };
    async {
        localside::wasi::run::<B_WASI, METHODS_RESOURCE, RESULTS_RESOURCE>(bguest, &mut b_wasi)
            .await?;
    };
    async {
        private_io::run(&mut b_io, bfs).await?;
    };
    async {
        localside::wasi::run::<C_WASI, INTRO_RESOURCE, RESULTS_RESOURCE>(cguest, &mut c_wasi)
            .await?;
    };
    async {
        private_io::run(&mut c_io, cfs).await?;
    };
    Ok(())
}

#[hibana_runtime::localside]
async fn run_2m_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 5]) -> ShellResult<()> {
    use global::{roles2m::*, *};
    let programs = project_2m();
    bus.observe(programs.observation());
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let dcheck = Validation::new();
    let echeck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
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
    let background_resource = rv.enter(session, &programs.background_resource)?;
    let mut intro = rv.enter(session, &programs.intro)?;
    let intro_resource = rv.enter(session, &programs.intro_resource)?;
    let mut methods = rv.enter(session, &programs.methods)?;
    let methods_resource = rv.enter(session, &programs.methods_resource)?;
    let mut experiments = rv.enter(session, &programs.experiments)?;
    let experiments_resource = rv.enter(session, &programs.experiments_resource)?;
    let mut results = rv.enter(session, &programs.results)?;
    let results_resource = rv.enter(session, &programs.results_resource)?;
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
    let afs = PrivateFs::new(aterm, bus);
    let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bfs = PrivateFs::new(bterm, bus);
    let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cfs = PrivateFs::new(cterm, bus);
    let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let dfs = PrivateFs::new(dterm, bus);
    let mut dmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let efs = PrivateFs::new(eterm, bus);
    let mut ememory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    async {
        localside::researcher::run::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(
            &mut a,
            bus,
            (
                &acheck,
                &Assignment {
                    agent: "RESEARCHER_A",
                    topic: "intro_background_connection",
                    section: "sec_intro_background",
                    sections: ["BACKGROUND_SECTION", "INTRO_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<B, BACKGROUND_RESOURCE, METHODS_RESOURCE>(
            &mut b,
            bus,
            (
                &bcheck,
                &Assignment {
                    agent: "RESEARCHER_B",
                    topic: "background_methods_connection",
                    section: "sec_background_methods",
                    sections: ["BACKGROUND_SECTION", "METHODS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<C, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(
            &mut c,
            bus,
            (
                &ccheck,
                &Assignment {
                    agent: "RESEARCHER_C",
                    topic: "methods_experiments_connection",
                    section: "sec_methods_experiments",
                    sections: ["EXPERIMENTS_SECTION", "METHODS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<D, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(
            &mut d,
            bus,
            (
                &dcheck,
                &Assignment {
                    agent: "RESEARCHER_D",
                    topic: "experiments_results_connection",
                    section: "sec_experiments_results",
                    sections: ["EXPERIMENTS_SECTION", "RESULTS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<E, INTRO_RESOURCE, RESULTS_RESOURCE>(
            &mut e,
            bus,
            (
                &echeck,
                &Assignment {
                    agent: "RESEARCHER_E",
                    topic: "results_intro_analysis",
                    section: "sec_results_intro",
                    sections: ["INTRO_SECTION", "RESULTS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::section::run(&mut background, background_resource).await?;
    };
    async {
        localside::section::run(&mut intro, intro_resource).await?;
    };
    async {
        localside::section::run(&mut methods, methods_resource).await?;
    };
    async {
        localside::section::run(&mut experiments, experiments_resource).await?;
    };
    async {
        localside::section::run(&mut results, results_resource).await?;
    };
    async {
        localside::wasi::run::<A_WASI, BACKGROUND_RESOURCE, INTRO_RESOURCE>(aguest, &mut a_wasi)
            .await?;
    };
    async {
        private_io::run(&mut a_io, afs).await?;
    };
    async {
        localside::wasi::run::<B_WASI, BACKGROUND_RESOURCE, METHODS_RESOURCE>(bguest, &mut b_wasi)
            .await?;
    };
    async {
        private_io::run(&mut b_io, bfs).await?;
    };
    async {
        localside::wasi::run::<C_WASI, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(cguest, &mut c_wasi)
            .await?;
    };
    async {
        private_io::run(&mut c_io, cfs).await?;
    };
    async {
        localside::wasi::run::<D_WASI, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(dguest, &mut d_wasi)
            .await?;
    };
    async {
        private_io::run(&mut d_io, dfs).await?;
    };
    async {
        localside::wasi::run::<E_WASI, INTRO_RESOURCE, RESULTS_RESOURCE>(eguest, &mut e_wasi)
            .await?;
    };
    async {
        private_io::run(&mut e_io, efs).await?;
    };
    Ok(())
}

#[hibana_runtime::localside]
async fn run_2h_guests(module: &[u8], bus: &Bus, terminals: [Terminal; 7]) -> ShellResult<()> {
    use global::{roles2h::*, *};
    let programs = project_2h();
    bus.observe(programs.observation());
    let acheck = Validation::new();
    let bcheck = Validation::new();
    let ccheck = Validation::new();
    let dcheck = Validation::new();
    let echeck = Validation::new();
    let fcheck = Validation::new();
    let gcheck = Validation::new();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &bus.transport)?;
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
    let background_resource = rv.enter(session, &programs.background_resource)?;
    let mut intro = rv.enter(session, &programs.intro)?;
    let intro_resource = rv.enter(session, &programs.intro_resource)?;
    let mut related = rv.enter(session, &programs.related)?;
    let related_resource = rv.enter(session, &programs.related_resource)?;
    let mut methods = rv.enter(session, &programs.methods)?;
    let methods_resource = rv.enter(session, &programs.methods_resource)?;
    let mut experiments = rv.enter(session, &programs.experiments)?;
    let experiments_resource = rv.enter(session, &programs.experiments_resource)?;
    let mut results = rv.enter(session, &programs.results)?;
    let results_resource = rv.enter(session, &programs.results_resource)?;
    let mut discussion = rv.enter(session, &programs.discussion)?;
    let discussion_resource = rv.enter(session, &programs.discussion_resource)?;
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
    let afs = PrivateFs::new(aterm, bus);
    let mut amemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut astore = HibanaWasiGuestStorage::uninit();
    let aguest = astore.init(module, GuestMemory::new(&mut amemory), bindings())?;
    let bfs = PrivateFs::new(bterm, bus);
    let mut bmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut bstore = HibanaWasiGuestStorage::uninit();
    let bguest = bstore.init(module, GuestMemory::new(&mut bmemory), bindings())?;
    let cfs = PrivateFs::new(cterm, bus);
    let mut cmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut cstore = HibanaWasiGuestStorage::uninit();
    let cguest = cstore.init(module, GuestMemory::new(&mut cmemory), bindings())?;
    let dfs = PrivateFs::new(dterm, bus);
    let mut dmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut dstore = HibanaWasiGuestStorage::uninit();
    let dguest = dstore.init(module, GuestMemory::new(&mut dmemory), bindings())?;
    let efs = PrivateFs::new(eterm, bus);
    let mut ememory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut estore = HibanaWasiGuestStorage::uninit();
    let eguest = estore.init(module, GuestMemory::new(&mut ememory), bindings())?;
    let ffs = PrivateFs::new(fterm, bus);
    let mut fmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut fstore = HibanaWasiGuestStorage::uninit();
    let fguest = fstore.init(module, GuestMemory::new(&mut fmemory), bindings())?;
    let gfs = PrivateFs::new(gterm, bus);
    let mut gmemory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
    let mut gstore = HibanaWasiGuestStorage::uninit();
    let gguest = gstore.init(module, GuestMemory::new(&mut gmemory), bindings())?;
    async {
        localside::researcher::run::<A, BACKGROUND_RESOURCE, INTRO_RESOURCE>(
            &mut a,
            bus,
            (
                &acheck,
                &Assignment {
                    agent: "RESEARCHER_A",
                    topic: "intro_background_connection",
                    section: "sec_intro_background",
                    sections: ["BACKGROUND_SECTION", "INTRO_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<B, BACKGROUND_RESOURCE, RELATED_RESOURCE>(
            &mut b,
            bus,
            (
                &bcheck,
                &Assignment {
                    agent: "RESEARCHER_B",
                    topic: "background_related_connection",
                    section: "sec_background_related",
                    sections: ["BACKGROUND_SECTION", "RELATED_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<C, METHODS_RESOURCE, RELATED_RESOURCE>(
            &mut c,
            bus,
            (
                &ccheck,
                &Assignment {
                    agent: "RESEARCHER_C",
                    topic: "related_methods_connection",
                    section: "sec_related_methods",
                    sections: ["METHODS_SECTION", "RELATED_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<D, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(
            &mut d,
            bus,
            (
                &dcheck,
                &Assignment {
                    agent: "RESEARCHER_D",
                    topic: "methods_experiments_connection",
                    section: "sec_methods_experiments",
                    sections: ["EXPERIMENTS_SECTION", "METHODS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<E, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(
            &mut e,
            bus,
            (
                &echeck,
                &Assignment {
                    agent: "RESEARCHER_E",
                    topic: "experiments_results_connection",
                    section: "sec_experiments_results",
                    sections: ["EXPERIMENTS_SECTION", "RESULTS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<F, DISCUSSION_RESOURCE, RESULTS_RESOURCE>(
            &mut f,
            bus,
            (
                &fcheck,
                &Assignment {
                    agent: "RESEARCHER_F",
                    topic: "results_discussion_connection",
                    section: "sec_results_discussion",
                    sections: ["DISCUSSION_SECTION", "RESULTS_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::researcher::run::<G, DISCUSSION_RESOURCE, INTRO_RESOURCE>(
            &mut g,
            bus,
            (
                &gcheck,
                &Assignment {
                    agent: "RESEARCHER_G",
                    topic: "discussion_intro_connection",
                    section: "sec_discussion_intro",
                    sections: ["DISCUSSION_SECTION", "INTRO_SECTION"],
                },
            ),
        )
        .await?;
    };
    async {
        localside::section::run(&mut background, background_resource).await?;
    };
    async {
        localside::section::run(&mut intro, intro_resource).await?;
    };
    async {
        localside::section::run(&mut related, related_resource).await?;
    };
    async {
        localside::section::run(&mut methods, methods_resource).await?;
    };
    async {
        localside::section::run(&mut experiments, experiments_resource).await?;
    };
    async {
        localside::section::run(&mut results, results_resource).await?;
    };
    async {
        localside::section::run(&mut discussion, discussion_resource).await?;
    };
    async {
        localside::wasi::run::<A_WASI, BACKGROUND_RESOURCE, INTRO_RESOURCE>(aguest, &mut a_wasi)
            .await?;
    };
    async {
        private_io::run(&mut a_io, afs).await?;
    };
    async {
        localside::wasi::run::<B_WASI, BACKGROUND_RESOURCE, RELATED_RESOURCE>(bguest, &mut b_wasi)
            .await?;
    };
    async {
        private_io::run(&mut b_io, bfs).await?;
    };
    async {
        localside::wasi::run::<C_WASI, METHODS_RESOURCE, RELATED_RESOURCE>(cguest, &mut c_wasi)
            .await?;
    };
    async {
        private_io::run(&mut c_io, cfs).await?;
    };
    async {
        localside::wasi::run::<D_WASI, EXPERIMENTS_RESOURCE, METHODS_RESOURCE>(dguest, &mut d_wasi)
            .await?;
    };
    async {
        private_io::run(&mut d_io, dfs).await?;
    };
    async {
        localside::wasi::run::<E_WASI, EXPERIMENTS_RESOURCE, RESULTS_RESOURCE>(eguest, &mut e_wasi)
            .await?;
    };
    async {
        private_io::run(&mut e_io, efs).await?;
    };
    async {
        localside::wasi::run::<F_WASI, DISCUSSION_RESOURCE, RESULTS_RESOURCE>(fguest, &mut f_wasi)
            .await?;
    };
    async {
        private_io::run(&mut f_io, ffs).await?;
    };
    async {
        localside::wasi::run::<G_WASI, DISCUSSION_RESOURCE, INTRO_RESOURCE>(gguest, &mut g_wasi)
            .await?;
    };
    async {
        private_io::run(&mut g_io, gfs).await?;
    };
    Ok(())
}

pub async fn run_stdio(module: &[u8], level: &str) -> ShellResult<()> {
    match level {
        "2E" => {
            let (bus, mut reader, terminals) = Bus::open(
                ["RESEARCHER_A", "RESEARCHER_B", "RESEARCHER_C"],
                [28, 29, 30].map(|role| (SessionId::new(18), role)),
            );
            let result =
                hibana_runtime::race(run_2e_guests(module, &bus, terminals), reader.run(&bus))
                    .await;
            match &result {
Ok(()) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"complete"})).await?,
Err(error) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
}
            result
        }
        "2M" => {
            let (bus, mut reader, terminals) = Bus::open(
                [
                    "RESEARCHER_A",
                    "RESEARCHER_B",
                    "RESEARCHER_C",
                    "RESEARCHER_D",
                    "RESEARCHER_E",
                ],
                [28, 29, 30, 31, 32].map(|role| (SessionId::new(18), role)),
            );
            let result =
                hibana_runtime::race(run_2m_guests(module, &bus, terminals), reader.run(&bus))
                    .await;
            match &result {
Ok(()) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"complete"})).await?,
Err(error) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
}
            result
        }
        "2H" => {
            let (bus, mut reader, terminals) = Bus::open(
                [
                    "RESEARCHER_A",
                    "RESEARCHER_B",
                    "RESEARCHER_C",
                    "RESEARCHER_D",
                    "RESEARCHER_E",
                    "RESEARCHER_F",
                    "RESEARCHER_G",
                ],
                [28, 29, 30, 31, 32, 33, 34].map(|role| (SessionId::new(18), role)),
            );
            let result =
                hibana_runtime::race(run_2h_guests(module, &bus, terminals), reader.run(&bus))
                    .await;
            match &result {
Ok(()) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"complete"})).await?,
Err(error) => reader.emit(&bus,serde_json::json!({"type":"run_result", "status":"failed", "error":error.to_string()})).await?,
}
            result
        }
        _ => Err(ShellError::message("expected writing task 2E, 2M or 2H")),
    }
}

pub fn projection(task: &str) -> ShellResult<serde_json::Value> {
    match task {
        "2E" => Ok(global::project_2e().observation()),
        "2M" => Ok(global::project_2m().observation()),
        "2H" => Ok(global::project_2h().observation()),
        _ => Err(ShellError::message("unsupported task projection")),
    }
}
