pub mod choreofs;
pub mod global;
mod localside;

use crate::imp::{
    domain::{Domain, RunError},
    transport::InMemoryTransport,
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};

/// Execute the 9E protocol with a supplied domain implementation.
pub async fn run_9e<D: Domain>(domain: &D) -> Result<(), RunError<D::Error>> {
    use global::{A, B, C, FORK_A, FORK_B, FORK_C, ForkA, ForkB, ForkC};
    let programs = global::project_9e();
    let mut slab = [0; 32 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, &transport_storage)
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
    let mut fork_a = rendezvous
        .enter(session, &programs.fork_a)
        .map_err(RunError::Attach)?;
    let mut fork_b = rendezvous
        .enter(session, &programs.fork_b)
        .map_err(RunError::Attach)?;
    let mut fork_c = rendezvous
        .enter(session, &programs.fork_c)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::philosopher::run::<A, ForkA, ForkC, D>("PHILOSOPHER_A", &mut a, domain),
        localside::philosopher::run::<B, ForkB, ForkA, D>("PHILOSOPHER_B", &mut b, domain),
        localside::philosopher::run::<C, ForkC, ForkB, D>("PHILOSOPHER_C", &mut c, domain),
        async {
            localside::fork::run::<FORK_A, ForkA>(&mut fork_a)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<FORK_B, ForkB>(&mut fork_b)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<FORK_C, ForkC>(&mut fork_c)
                .await
                .map_err(RunError::Endpoint)
        },
    )?;
    Ok(())
}

/// Execute the 9M protocol; fork endpoints own each lease.
pub async fn run_9m<D: Domain>(domain: &D) -> Result<(), RunError<D::Error>> {
    use global::{ForkA, ForkB, ForkC, ForkD, ForkE, project_9m, roles9m::*};
    let programs = project_9m();
    let mut slab = [0; 128 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, &transport_storage)
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
    let mut fork_a = rendezvous
        .enter(session, &programs.fork_a)
        .map_err(RunError::Attach)?;
    let mut fork_b = rendezvous
        .enter(session, &programs.fork_b)
        .map_err(RunError::Attach)?;
    let mut fork_c = rendezvous
        .enter(session, &programs.fork_c)
        .map_err(RunError::Attach)?;
    let mut fork_d = rendezvous
        .enter(session, &programs.fork_d)
        .map_err(RunError::Attach)?;
    let mut fork_e = rendezvous
        .enter(session, &programs.fork_e)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::philosopher::run::<{ A }, ForkA, ForkE, D>("PHILOSOPHER_A", &mut a, domain),
        localside::philosopher::run::<{ B }, ForkB, ForkA, D>("PHILOSOPHER_B", &mut b, domain),
        localside::philosopher::run::<{ C }, ForkC, ForkB, D>("PHILOSOPHER_C", &mut c, domain),
        localside::philosopher::run::<{ D }, ForkD, ForkC, D>("PHILOSOPHER_D", &mut d, domain),
        localside::philosopher::run::<{ E }, ForkE, ForkD, D>("PHILOSOPHER_E", &mut e, domain),
        async {
            localside::fork::run::<{ FORK_A }, ForkA>(&mut fork_a)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_B }, ForkB>(&mut fork_b)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_C }, ForkC>(&mut fork_c)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_D }, ForkD>(&mut fork_d)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_E }, ForkE>(&mut fork_e)
                .await
                .map_err(RunError::Endpoint)
        },
    )?;
    Ok(())
}

/// Execute the 9H protocol; fork endpoints own each lease.
pub async fn run_9h<D: Domain>(domain: &D) -> Result<(), RunError<D::Error>> {
    use global::{ForkA, ForkB, ForkC, ForkD, ForkE, ForkF, ForkG, project_9h, roles9h::*};
    let programs = project_9h();
    let mut slab = [0; 128 * 1024];
    let transport_storage = InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
    let kit = storage.init();
    let rendezvous = kit
        .rendezvous(&mut slab, &transport_storage)
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
    let mut fork_a = rendezvous
        .enter(session, &programs.fork_a)
        .map_err(RunError::Attach)?;
    let mut fork_b = rendezvous
        .enter(session, &programs.fork_b)
        .map_err(RunError::Attach)?;
    let mut fork_c = rendezvous
        .enter(session, &programs.fork_c)
        .map_err(RunError::Attach)?;
    let mut fork_d = rendezvous
        .enter(session, &programs.fork_d)
        .map_err(RunError::Attach)?;
    let mut fork_e = rendezvous
        .enter(session, &programs.fork_e)
        .map_err(RunError::Attach)?;
    let mut fork_f = rendezvous
        .enter(session, &programs.fork_f)
        .map_err(RunError::Attach)?;
    let mut fork_g = rendezvous
        .enter(session, &programs.fork_g)
        .map_err(RunError::Attach)?;
    tokio::try_join!(
        localside::philosopher::run::<{ A }, ForkA, ForkG, D>("PHILOSOPHER_A", &mut a, domain),
        localside::philosopher::run::<{ B }, ForkB, ForkA, D>("PHILOSOPHER_B", &mut b, domain),
        localside::philosopher::run::<{ C }, ForkC, ForkB, D>("PHILOSOPHER_C", &mut c, domain),
        localside::philosopher::run::<{ D }, ForkD, ForkC, D>("PHILOSOPHER_D", &mut d, domain),
        localside::philosopher::run::<{ E }, ForkE, ForkD, D>("PHILOSOPHER_E", &mut e, domain),
        localside::philosopher::run::<{ F }, ForkF, ForkE, D>("PHILOSOPHER_F", &mut f, domain),
        localside::philosopher::run::<{ G }, ForkG, ForkF, D>("PHILOSOPHER_G", &mut g, domain),
        async {
            localside::fork::run::<{ FORK_A }, ForkA>(&mut fork_a)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_B }, ForkB>(&mut fork_b)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_C }, ForkC>(&mut fork_c)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_D }, ForkD>(&mut fork_d)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_E }, ForkE>(&mut fork_e)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_F }, ForkF>(&mut fork_f)
                .await
                .map_err(RunError::Endpoint)
        },
        async {
            localside::fork::run::<{ FORK_G }, ForkG>(&mut fork_g)
                .await
                .map_err(RunError::Endpoint)
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Domain, Work};
    use std::cell::RefCell;

    struct Journal {
        inputs: RefCell<std::collections::BTreeMap<&'static str, std::collections::VecDeque<Work>>>,
        events: RefCell<Vec<Work>>,
    }

    impl Domain for Journal {
        type Error = std::convert::Infallible;
        type Call = Work;

        async fn request(&self, agent: &'static str) -> Result<Work, Self::Error> {
            Ok(self
                .inputs
                .borrow_mut()
                .get_mut(agent)
                .unwrap()
                .pop_front()
                .unwrap())
        }

        async fn execute(&self, _agent: &'static str, work: Work) -> Result<(), Self::Error> {
            self.events.borrow_mut().push(work);
            Ok(())
        }
    }

    async fn reject_after_thinking<M: hibana::g::Message<Payload = ()>>() {
        let programs = global::project_9e();
        let mut slab = [0; 32 * 1024];
        let transport_storage = InMemoryTransport::new([]);
        let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
        let kit = storage.init();
        let rendezvous = kit.rendezvous(&mut slab, &transport_storage).unwrap();
        let mut endpoint = rendezvous.enter(SessionId::new(2), &programs.a).unwrap();
        endpoint.send::<global::Think>(&()).await.unwrap();
        assert!(endpoint.send::<M>(&()).await.is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn eating_without_both_forks_is_rejected() {
        reject_after_thinking::<global::Eat>().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn finishing_without_returning_forks_is_rejected() {
        reject_after_thinking::<global::Finished>().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn completion_is_consumed_once() {
        use global::{A, B, C, FORK_A, FORK_B, FORK_C, ForkA, ForkB, ForkC};
        let programs = global::project_9e();
        let journal = Journal {
            inputs: RefCell::new(
                ["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"]
                    .into_iter()
                    .map(|agent| (agent, [Work::Think, Work::Eat].into()))
                    .collect(),
            ),
            events: RefCell::new(Vec::new()),
        };
        let mut slab = [0; 32 * 1024];
        let transport_storage = InMemoryTransport::new([]);
        let mut storage = SessionKitStorage::<&InMemoryTransport>::uninit();
        let kit = storage.init();
        let rendezvous = kit.rendezvous(&mut slab, &transport_storage).unwrap();
        let session = SessionId::new(3);
        let mut a = rendezvous.enter(session, &programs.a).unwrap();
        let mut b = rendezvous.enter(session, &programs.b).unwrap();
        let mut c = rendezvous.enter(session, &programs.c).unwrap();
        let mut fork_a = rendezvous.enter(session, &programs.fork_a).unwrap();
        let mut fork_b = rendezvous.enter(session, &programs.fork_b).unwrap();
        let mut fork_c = rendezvous.enter(session, &programs.fork_c).unwrap();
        tokio::try_join!(
            localside::philosopher::run::<A, ForkA, ForkC, _>("PHILOSOPHER_A", &mut a, &journal),
            localside::philosopher::run::<B, ForkB, ForkA, _>("PHILOSOPHER_B", &mut b, &journal),
            localside::philosopher::run::<C, ForkC, ForkB, _>("PHILOSOPHER_C", &mut c, &journal),
            async {
                localside::fork::run::<FORK_A, ForkA>(&mut fork_a)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::fork::run::<FORK_B, ForkB>(&mut fork_b)
                    .await
                    .map_err(RunError::Endpoint)
            },
            async {
                localside::fork::run::<FORK_C, ForkC>(&mut fork_c)
                    .await
                    .map_err(RunError::Endpoint)
            },
        )
        .unwrap();
        assert!(a.send::<global::Finished>(&()).await.is_err());
        assert_eq!(journal.events.borrow().len(), 6);
    }
}
