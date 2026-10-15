use choreofix::{
    RunError,
    tasks::codebase::{self, Domain, Operation, Request},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
};

struct Call {
    operation: Operation,
    result: Value,
}
impl Request for Call {
    fn operation(&self) -> Operation {
        self.operation
    }
}

struct Input {
    calls: RefCell<BTreeMap<String, VecDeque<Call>>>,
    events: RefCell<Vec<(&'static str, Operation, Value)>>,
}

impl Input {
    fn failures_then_pass(failures: usize) -> Self {
        Self::for_roles(7, failures)
    }
    fn for_roles(count: usize, failures: usize) -> Self {
        let calls = "ABCDEFG"
            .chars()
            .take(count)
            .map(|role| {
                let mut calls = VecDeque::new();
                calls.push_back(Call {
                    operation: Operation::Design,
                    result: json!({"success":true}),
                });
                for failed in (0..failures).map(|_| true).chain(std::iter::once(false)) {
                    for operation in [Operation::Implement, Operation::Commit, Operation::Test] {
                        let success = operation != Operation::Test || !failed;
                        calls.push_back(Call {
                            operation,
                            result: json!({"success":success}),
                        });
                    }
                }
                (format!("DEVELOPER_{role}"), calls)
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
        self.events
            .borrow_mut()
            .push((agent, call.operation, call.result.clone()));
        Ok(call.result)
    }
}

#[tokio::test(flavor = "current_thread")]
async fn seven_original_developers_retry_twice_through_resolved_rolled_protocol() {
    let input = Input::failures_then_pass(2);
    tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1h(&input))
        .await
        .expect("protocol stalled")
        .unwrap();
    let events = input.events.borrow();
    assert_eq!(events.len(), 70);
    for role in "ABCDEFG".chars() {
        let agent = format!("DEVELOPER_{role}");
        let operations = events
            .iter()
            .filter(|event| event.0 == agent)
            .collect::<Vec<_>>();
        assert_eq!(
            operations.iter().map(|event| event.1).collect::<Vec<_>>(),
            [
                Operation::Design,
                Operation::Implement,
                Operation::Commit,
                Operation::Test,
                Operation::Implement,
                Operation::Commit,
                Operation::Test,
                Operation::Implement,
                Operation::Commit,
                Operation::Test,
            ]
        );
        let outcomes = operations
            .iter()
            .filter(|event| event.1 == Operation::Test)
            .map(|event| event.2["success"].as_bool().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(outcomes, [false, false, true]);
    }
    assert!(input.calls.borrow().values().all(VecDeque::is_empty));
}

#[tokio::test(flavor = "current_thread")]
async fn testing_before_implementation_or_commit_never_executes_domain_tool() {
    let input = Input::failures_then_pass(0);
    input
        .calls
        .borrow_mut()
        .get_mut("DEVELOPER_A")
        .unwrap()
        .push_front(Call {
            operation: Operation::Test,
            result: json!({"success":true}),
        });
    assert!(matches!(
        tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1h(&input))
            .await
            .expect("protocol stalled"),
        Err(RunError::Endpoint(_))
    ));
    assert!(input.events.borrow().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn non_test_domain_failure_stops_without_hidden_retry() {
    let input = Input::failures_then_pass(0);
    input
        .calls
        .borrow_mut()
        .get_mut("DEVELOPER_A")
        .unwrap()
        .front_mut()
        .unwrap()
        .result = json!({"success":false});
    assert!(matches!(
        tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1h(&input))
            .await
            .expect("protocol stalled"),
        Err(RunError::Domain(_))
    ));
    assert_eq!(input.events.borrow().len(), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn returned_modules_allow_b_to_commit_while_a_is_still_testing() {
    use tokio::sync::oneshot;

    struct Delayed {
        input: Input,
        a_started: RefCell<Option<oneshot::Sender<()>>>,
        wait_for_a: RefCell<Option<oneshot::Receiver<()>>>,
        b_committed: RefCell<Option<oneshot::Sender<()>>>,
        release_a: RefCell<Option<oneshot::Receiver<()>>>,
    }
    impl Domain for Delayed {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            self.input.request(agent).await
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            if agent == "DEVELOPER_A" && call.operation == Operation::Test {
                self.a_started
                    .borrow_mut()
                    .take()
                    .unwrap()
                    .send(())
                    .unwrap();
                let release = self.release_a.borrow_mut().take().unwrap();
                release.await.unwrap();
            }
            if agent == "DEVELOPER_B" && call.operation == Operation::Implement {
                let started = self.wait_for_a.borrow_mut().take().unwrap();
                started.await.unwrap();
            }
            if agent == "DEVELOPER_B" && call.operation == Operation::Commit {
                self.b_committed
                    .borrow_mut()
                    .take()
                    .unwrap()
                    .send(())
                    .unwrap();
            }
            self.input.execute(agent, call).await
        }
    }
    let (a_started, wait_for_a) = oneshot::channel();
    let (b_committed, wait_for_b) = oneshot::channel();
    let (release_a, wait_for_release) = oneshot::channel();
    let domain = Delayed {
        input: Input::failures_then_pass(0),
        a_started: RefCell::new(Some(a_started)),
        wait_for_a: RefCell::new(Some(wait_for_a)),
        b_committed: RefCell::new(Some(b_committed)),
        release_a: RefCell::new(Some(wait_for_release)),
    };
    let observer = async {
        wait_for_b.await.unwrap();
        assert!(
            !domain
                .input
                .events
                .borrow()
                .iter()
                .any(|(agent, operation, _)| *agent == "DEVELOPER_A"
                    && *operation == Operation::Test)
        );
        release_a.send(()).unwrap();
        Ok::<(), RunError<String>>(())
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        tokio::try_join!(codebase::run_1h(&domain), observer)
    })
    .await
    .expect("protocol stalled while A tested without holding modules")
    .unwrap();
    assert_eq!(domain.input.events.borrow().len(), 28);
}

#[tokio::test(flavor = "current_thread")]
async fn smaller_codebase_levels_finish_after_one_terminal_test() {
    for count in [3, 5] {
        let domain = Input::for_roles(count, 0);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            match count {
                3 => codebase::run_1e(&domain).await,
                5 => codebase::run_1m(&domain).await,
                _ => unreachable!(),
            }
        })
        .await
        .expect("codebase level stalled")
        .unwrap();
        assert_eq!(domain.events.borrow().len(), count * 4);
        assert!(domain.calls.borrow().values().all(VecDeque::is_empty));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn terminal_test_failure_does_not_start_another_attempt() {
    let domain = Input::for_roles(3, 0);
    domain
        .calls
        .borrow_mut()
        .get_mut("DEVELOPER_A")
        .unwrap()
        .back_mut()
        .unwrap()
        .result = json!({"success": false});
    tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1e(&domain))
        .await
        .unwrap_or_else(|error| {
            panic!(
                "terminal test stalled: {error}; events: {:?}",
                domain.events.borrow()
            )
        })
        .unwrap();
    assert_eq!(domain.events.borrow().len(), 12);
    assert!(domain.calls.borrow().values().all(VecDeque::is_empty));
}

#[tokio::test(flavor = "current_thread")]
async fn independent_terminal_commits_reach_the_effect_concurrently() {
    struct Concurrent {
        input: Input,
        both_committing: tokio::sync::Barrier,
    }
    impl Domain for Concurrent {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            self.input.request(agent).await
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            if call.operation == Operation::Commit
                && ["DEVELOPER_A", "DEVELOPER_C"].contains(&agent)
            {
                // Neither effect can finish until the other independent effect starts.
                self.both_committing.wait().await;
            }
            self.input.execute(agent, call).await
        }
    }
    let domain = Concurrent {
        input: Input::for_roles(5, 0),
        both_committing: tokio::sync::Barrier::new(2),
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1m(&domain))
        .await
        .expect("independent commits did not run concurrently")
        .unwrap();
    assert_eq!(domain.input.events.borrow().len(), 20);
    assert!(domain.input.calls.borrow().values().all(VecDeque::is_empty));
}

#[tokio::test(flavor = "current_thread")]
async fn waiting_for_a_model_commit_request_does_not_hold_its_modules() {
    use tokio::sync::oneshot;
    struct DelayedRequest {
        input: Input,
        wait_for_b: RefCell<Option<oneshot::Receiver<()>>>,
        b_committed: RefCell<Option<oneshot::Sender<()>>>,
    }
    impl Domain for DelayedRequest {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            let call = self.input.request(agent).await?;
            if agent == "DEVELOPER_A" && call.operation == Operation::Commit {
                let wait = self.wait_for_b.borrow_mut().take().unwrap();
                wait.await.unwrap();
            }
            Ok(call)
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            if agent == "DEVELOPER_B" && call.operation == Operation::Commit {
                self.b_committed
                    .borrow_mut()
                    .take()
                    .unwrap()
                    .send(())
                    .unwrap();
            }
            self.input.execute(agent, call).await
        }
    }
    let (b_committed, wait_for_b) = oneshot::channel();
    let input = DelayedRequest {
        input: Input::failures_then_pass(0),
        wait_for_b: RefCell::new(Some(wait_for_b)),
        b_committed: RefCell::new(Some(b_committed)),
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1h(&input))
        .await
        .expect("an idle model held a module needed by B")
        .unwrap();
    assert_eq!(input.input.events.borrow().len(), 28);
}

#[tokio::test(flavor = "current_thread")]
async fn seven_role_protocol_runs_three_disjoint_commits_concurrently() {
    struct Concurrent {
        input: Input,
        commits: tokio::sync::Barrier,
        observations: RefCell<Vec<(&'static str, &'static str)>>,
    }
    impl Domain for Concurrent {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            self.input.request(agent).await
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            if call.operation == Operation::Commit {
                self.observations.borrow_mut().push((agent, "begin"));
            }
            if call.operation == Operation::Commit
                && matches!(agent, "DEVELOPER_A" | "DEVELOPER_C" | "DEVELOPER_E")
            {
                // Test fixture only. All three actual domain futures must enter
                // before any can finish. Serial execution stalls this test.
                self.commits.wait().await;
            }
            if call.operation == Operation::Commit {
                tokio::task::yield_now().await;
                self.observations.borrow_mut().push((agent, "end"));
            }
            self.input.execute(agent, call).await
        }
    }
    let domain = Concurrent {
        input: Input::failures_then_pass(0),
        commits: tokio::sync::Barrier::new(3),
        observations: RefCell::default(),
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1h(&domain))
        .await
        .expect("three disjoint commits did not run concurrently")
        .unwrap();
    assert_eq!(domain.input.events.borrow().len(), 28);
    assert!(domain.input.calls.borrow().values().all(VecDeque::is_empty));
    // Offline observation only. This set never authorizes a domain operation.
    let pairs = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (0, 6)];
    let mut active = std::collections::BTreeSet::new();
    let mut peak = 0;
    for &(agent, event) in domain.observations.borrow().iter() {
        let role = (agent.as_bytes().last().unwrap() - b'A') as usize;
        match event {
            "begin" => {
                let (first, second) = pairs[role];
                for &other in &active {
                    let (left, right) = pairs[other];
                    assert!(first != left && first != right && second != left && second != right);
                }
                assert!(active.insert(role));
                peak = peak.max(active.len());
            }
            "end" => assert!(active.remove(&role)),
            _ => panic!("unexpected domain observation"),
        }
    }
    assert!(active.is_empty());
    assert_eq!(peak, 3);
}

#[tokio::test(flavor = "current_thread")]
async fn early_finished_developers_do_not_stop_remaining_reconsideration() {
    let input = Input::failures_then_pass(3);
    let failures = [0, 2, 1, 3, 0, 2, 1];
    for (role, count) in "ABCDEFG".chars().zip(failures) {
        let mut all = input.calls.borrow_mut();
        let calls = all.get_mut(&format!("DEVELOPER_{role}")).unwrap();
        for _ in 0..(3 - count) * 3 {
            calls.remove(1).unwrap();
        }
    }
    tokio::time::timeout(std::time::Duration::from_secs(5), codebase::run_1h(&input))
        .await
        .expect("remaining developers stalled after an independent owner finished")
        .unwrap();
    let events = input.events.borrow();
    assert_eq!(events.len(), 28 + 3 * failures.iter().sum::<usize>());
    for (role, count) in "ABCDEFG".chars().zip(failures) {
        let tests: Vec<_> = events
            .iter()
            .filter(|event| event.0 == format!("DEVELOPER_{role}") && event.1 == Operation::Test)
            .collect();
        assert_eq!(tests.len(), count + 1);
        assert!(
            tests[..count]
                .iter()
                .all(|event| event.2["success"] == false)
        );
        assert_eq!(tests.last().unwrap().2["success"], true);
    }
    assert!(input.calls.borrow().values().all(VecDeque::is_empty));
}
