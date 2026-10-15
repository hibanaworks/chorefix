use choreofix::{
    RunError,
    tasks::api::{self, Domain, Operation, Request},
};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};

struct Call(Operation, Value);
impl Request for Call {
    fn operation(&self) -> Operation {
        self.0
    }
}
struct Input {
    calls: RefCell<BTreeMap<&'static str, Call>>,
    events: RefCell<Vec<(&'static str, Operation)>>,
}
impl Input {
    fn with_test_result(result: bool) -> Self {
        Self {
            calls: RefCell::new(BTreeMap::from([
                (
                    "DB_ADMIN",
                    Call(Operation::Migrate, json!({"success":true})),
                ),
                (
                    "BACKEND_DEV",
                    Call(Operation::Implement, json!({"success":true})),
                ),
                ("TESTER", Call(Operation::Test, json!({"success":result}))),
            ])),
            events: RefCell::default(),
        }
    }
}
impl Domain for Input {
    type Call = Call;
    async fn request(&self, agent: &'static str) -> Result<Call, String> {
        self.calls
            .borrow_mut()
            .remove(agent)
            .ok_or("input closed".into())
    }
    async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
        self.events.borrow_mut().push((agent, call.0));
        Ok(call.1)
    }
}
#[tokio::test(flavor = "current_thread")]
async fn either_test_outcome_notifies_both_peers_without_retry() {
    for result in [false, true] {
        let input = Input::with_test_result(result);
        tokio::time::timeout(std::time::Duration::from_secs(3), api::run_8e(&input))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            *input.events.borrow(),
            [
                ("DB_ADMIN", Operation::Migrate),
                ("BACKEND_DEV", Operation::Implement),
                ("TESTER", Operation::Test)
            ]
        );
        assert!(input.calls.borrow().is_empty());
    }
}
#[tokio::test(flavor = "current_thread")]
async fn mismatched_test_request_has_no_test_effect() {
    let input = Input::with_test_result(true);
    input
        .calls
        .borrow_mut()
        .insert("TESTER", Call(Operation::Migrate, json!({"success":true})));
    assert!(matches!(
        tokio::time::timeout(std::time::Duration::from_secs(3), api::run_8e(&input))
            .await
            .unwrap(),
        Err(RunError::Endpoint(_))
    ));
    assert_eq!(input.events.borrow().len(), 2);
}
