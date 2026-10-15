use hibana_tracefix::{
    RunError,
    tasks::writing::{self, Domain, Operation, Request},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    time::Duration,
};

struct Call {
    operation: Operation,
    success: bool,
}
impl Request for Call {
    fn operation(&self) -> Operation {
        self.operation
    }
}

struct Input {
    calls: RefCell<BTreeMap<String, VecDeque<Call>>>,
    events: RefCell<Vec<(&'static str, Operation)>>,
}
impl Input {
    fn for_roles(count: usize) -> Self {
        let calls = "ABCDEFG"
            .chars()
            .take(count)
            .map(|role| {
                let calls = [
                    Operation::Literature,
                    Operation::Draft,
                    Operation::Write,
                    Operation::Review,
                ]
                .into_iter()
                .map(|operation| Call {
                    operation,
                    success: true,
                })
                .collect();
                (format!("RESEARCHER_{role}"), calls)
            })
            .collect();
        Self {
            calls: RefCell::new(calls),
            events: RefCell::default(),
        }
    }
}
impl Domain for Input {
    type Call = Call;
    async fn request(&self, agent: &'static str) -> Result<Call, String> {
        self.calls
            .borrow_mut()
            .get_mut(agent)
            .unwrap()
            .pop_front()
            .ok_or("input closed".into())
    }
    async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
        self.events.borrow_mut().push((agent, call.operation));
        Ok(json!({"success":call.success}))
    }
}
async fn run(count: usize, input: &impl Domain) -> Result<(), RunError<String>> {
    tokio::time::timeout(Duration::from_secs(5), async {
        match count {
            3 => writing::run_2e(input).await,
            5 => writing::run_2m(input).await,
            7 => writing::run_2h(input).await,
            _ => unreachable!(),
        }
    })
    .await
    .expect("writing protocol stalled")
}

#[tokio::test(flavor = "current_thread")]
async fn all_original_levels_finish_with_a_terminal_review() {
    for count in [3, 5, 7] {
        let input = Input::for_roles(count);
        // A review finding ends work even in 2H. It must not create an attempt.
        input
            .calls
            .borrow_mut()
            .get_mut("RESEARCHER_A")
            .unwrap()
            .back_mut()
            .unwrap()
            .success = false;
        run(count, &input).await.unwrap();
        assert_eq!(input.events.borrow().len(), count * 4);
        assert!(input.calls.borrow().values().all(VecDeque::is_empty));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn a_write_before_literature_has_no_domain_effect() {
    let input = Input::for_roles(7);
    input
        .calls
        .borrow_mut()
        .get_mut("RESEARCHER_A")
        .unwrap()
        .push_front(Call {
            operation: Operation::Write,
            success: true,
        });
    assert!(matches!(run(7, &input).await, Err(RunError::Endpoint(_))));
    assert!(input.events.borrow().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn a_duplicate_write_does_not_run_after_returning_the_sections() {
    let input = Input::for_roles(3);
    input
        .calls
        .borrow_mut()
        .get_mut("RESEARCHER_A")
        .unwrap()
        .back_mut()
        .unwrap()
        .operation = Operation::Write;
    assert!(matches!(run(3, &input).await, Err(RunError::Endpoint(_))));
    assert_eq!(
        input
            .events
            .borrow()
            .iter()
            .filter(|(agent, op)| *agent == "RESEARCHER_A" && *op == Operation::Write)
            .count(),
        1
    );
}

#[tokio::test(flavor = "current_thread")]
async fn disjoint_writes_overlap_and_a_review_does_not_hold_sections() {
    use tokio::sync::oneshot;
    struct Delayed {
        input: Input,
        a_writing: RefCell<Option<oneshot::Sender<()>>>,
        wait_for_a_write: RefCell<Option<oneshot::Receiver<()>>>,
        c_writing: RefCell<Option<oneshot::Sender<()>>>,
        wait_for_c_write: RefCell<Option<oneshot::Receiver<()>>>,
        a_reviewing: RefCell<Option<oneshot::Sender<()>>>,
        wait_for_a_review: RefCell<Option<oneshot::Receiver<()>>>,
        b_writing: RefCell<Option<oneshot::Sender<()>>>,
        wait_for_b_write: RefCell<Option<oneshot::Receiver<()>>>,
    }
    impl Domain for Delayed {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            self.input.request(agent).await
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            match (agent, call.operation) {
                ("RESEARCHER_A", Operation::Write) => {
                    self.a_writing
                        .borrow_mut()
                        .take()
                        .unwrap()
                        .send(())
                        .unwrap();
                    let wait = self.wait_for_c_write.borrow_mut().take().unwrap();
                    wait.await.unwrap();
                }
                ("RESEARCHER_C", Operation::Draft) => {
                    let wait = self.wait_for_a_write.borrow_mut().take().unwrap();
                    wait.await.unwrap();
                }
                ("RESEARCHER_C", Operation::Write) => {
                    self.c_writing
                        .borrow_mut()
                        .take()
                        .unwrap()
                        .send(())
                        .unwrap();
                }
                ("RESEARCHER_A", Operation::Review) => {
                    self.a_reviewing
                        .borrow_mut()
                        .take()
                        .unwrap()
                        .send(())
                        .unwrap();
                    let wait = self.wait_for_b_write.borrow_mut().take().unwrap();
                    wait.await.unwrap();
                }
                ("RESEARCHER_B", Operation::Draft) => {
                    let wait = self.wait_for_a_review.borrow_mut().take().unwrap();
                    wait.await.unwrap();
                }
                ("RESEARCHER_B", Operation::Write) => {
                    self.b_writing
                        .borrow_mut()
                        .take()
                        .unwrap()
                        .send(())
                        .unwrap();
                }
                _ => {}
            }
            self.input.execute(agent, call).await
        }
    }
    let (a_writing, wait_for_a_write) = oneshot::channel();
    let (c_writing, wait_for_c_write) = oneshot::channel();
    let (a_reviewing, wait_for_a_review) = oneshot::channel();
    let (b_writing, wait_for_b_write) = oneshot::channel();
    let input = Delayed {
        input: Input::for_roles(7),
        a_writing: RefCell::new(Some(a_writing)),
        wait_for_a_write: RefCell::new(Some(wait_for_a_write)),
        c_writing: RefCell::new(Some(c_writing)),
        wait_for_c_write: RefCell::new(Some(wait_for_c_write)),
        a_reviewing: RefCell::new(Some(a_reviewing)),
        wait_for_a_review: RefCell::new(Some(wait_for_a_review)),
        b_writing: RefCell::new(Some(b_writing)),
        wait_for_b_write: RefCell::new(Some(wait_for_b_write)),
    };
    run(7, &input).await.unwrap();
    assert_eq!(input.input.events.borrow().len(), 28);
}
