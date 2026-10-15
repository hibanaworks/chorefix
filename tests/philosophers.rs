use std::cell::RefCell;

use hibana_tracefix::{Domain, Work, tasks::philosophers};

struct Inputs(RefCell<std::collections::BTreeMap<String, std::collections::VecDeque<Work>>>);

impl Default for Inputs {
    fn default() -> Self {
        Self(RefCell::new(
            "ABCDEFG"
                .chars()
                .map(|role| {
                    (
                        format!("PHILOSOPHER_{role}"),
                        [Work::Think, Work::Eat].into(),
                    )
                })
                .collect(),
        ))
    }
}

impl Inputs {
    fn next(&self, agent: &str) -> Work {
        self.0
            .borrow_mut()
            .get_mut(agent)
            .unwrap()
            .pop_front()
            .unwrap()
    }
}

#[derive(Default)]
struct Journal {
    inputs: Inputs,
    events: RefCell<Vec<(&'static str, Work)>>,
}

impl Domain for Journal {
    type Error = std::convert::Infallible;
    type Call = Work;

    async fn request(&self, agent: &'static str) -> Result<Work, Self::Error> {
        Ok(self.inputs.next(agent))
    }

    async fn execute(&self, agent: &'static str, work: Work) -> Result<(), Self::Error> {
        self.events.borrow_mut().push((agent, work));
        Ok(())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn protocol_9e_runs_each_philosophers_domain_work() {
    let journal = Journal::default();
    philosophers::run_9e(&journal).await.expect("complete 9E");
    let events = journal.events.borrow();
    assert_eq!(events.len(), 6);
    for agent in ["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"] {
        let local: Vec<_> = events
            .iter()
            .filter(|(who, _)| *who == agent)
            .map(|(_, work)| *work)
            .collect();
        assert_eq!(local, [Work::Think, Work::Eat]);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn domain_error_cannot_be_reported_as_protocol_completion() {
    struct Unavailable;
    impl Domain for Unavailable {
        type Error = &'static str;
        type Call = Work;

        async fn request(&self, _agent: &'static str) -> Result<Work, Self::Error> {
            Err("domain unavailable")
        }

        async fn execute(&self, _agent: &'static str, _work: Work) -> Result<(), Self::Error> {
            unreachable!("no domain call was received")
        }
    }
    assert!(matches!(
        philosophers::run_9e(&Unavailable).await,
        Err(hibana_tracefix::RunError::Domain("domain unavailable"))
    ));
}

struct Overlap {
    inputs: Inputs,
    active: RefCell<Vec<&'static str>>,
    maximum: std::cell::Cell<usize>,
}

impl Domain for Overlap {
    type Error = std::convert::Infallible;
    type Call = Work;

    async fn request(&self, agent: &'static str) -> Result<Work, Self::Error> {
        Ok(self.inputs.next(agent))
    }

    async fn execute(&self, agent: &'static str, work: Work) -> Result<(), Self::Error> {
        if work == Work::Eat {
            // This is an observer. It never grants a fork or advances an endpoint.
            self.active.borrow_mut().push(agent);
            self.maximum
                .set(self.maximum.get().max(self.active.borrow().len()));
            tokio::task::yield_now().await;
            self.active.borrow_mut().retain(|who| *who != agent);
        }
        Ok(())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn five_philosophers_keep_two_independent_eaters() {
    let observer = Overlap {
        inputs: Inputs::default(),
        active: RefCell::default(),
        maximum: std::cell::Cell::new(0),
    };
    philosophers::run_9m(&observer).await.unwrap();
    assert_eq!(observer.maximum.get(), 2);
    assert!(observer.active.borrow().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn seven_philosophers_keep_three_independent_eaters() {
    let observer = Overlap {
        inputs: Inputs::default(),
        active: RefCell::default(),
        maximum: std::cell::Cell::new(0),
    };
    philosophers::run_9h(&observer).await.unwrap();
    assert_eq!(observer.maximum.get(), 3);
    assert!(observer.active.borrow().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn disjoint_eater_d_does_not_wait_for_delayed_a() {
    struct DelayA {
        inputs: Inputs,
        wake_a: RefCell<Option<tokio::sync::oneshot::Sender<()>>>,
        wait_a: RefCell<Option<tokio::sync::oneshot::Receiver<()>>>,
    }
    impl Domain for DelayA {
        type Error = std::convert::Infallible;
        type Call = Work;

        async fn request(&self, agent: &'static str) -> Result<Work, Self::Error> {
            Ok(self.inputs.next(agent))
        }

        async fn execute(&self, agent: &'static str, work: Work) -> Result<(), Self::Error> {
            if work == Work::Eat {
                match agent {
                    "PHILOSOPHER_A" => {
                        let wait = self.wait_a.borrow_mut().take().unwrap();
                        wait.await.unwrap();
                    }
                    "PHILOSOPHER_D" => {
                        self.wake_a.borrow_mut().take().unwrap().send(()).unwrap();
                    }
                    _ => (),
                }
            }
            Ok(())
        }
    }
    let (wake_a, wait_a) = tokio::sync::oneshot::channel();
    let domain = DelayA {
        inputs: Inputs::default(),
        wake_a: RefCell::new(Some(wake_a)),
        wait_a: RefCell::new(Some(wait_a)),
    };
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        philosophers::run_9m(&domain),
    )
    .await
    .expect("D must progress while A is delayed")
    .unwrap();
}
