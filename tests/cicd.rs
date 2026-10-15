use choreofix::{
    RunError,
    tasks::cicd::{self, Domain, Operation, Request},
};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap, time::Duration};

struct Call(Operation);
impl Request for Call {
    fn operation(&self) -> Operation {
        self.0
    }
}
struct Input {
    calls: RefCell<BTreeMap<&'static str, Operation>>,
    events: RefCell<Vec<(&'static str, Operation)>>,
}
impl Input {
    fn new() -> Self {
        Self {
            calls: RefCell::new(BTreeMap::from([
                ("DEVELOPER", Operation::Compile),
                ("TESTER", Operation::Test),
                ("DEPLOYER", Operation::Deploy),
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
            .map(Call)
            .ok_or("input closed".into())
    }
    async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
        self.events.borrow_mut().push((agent, call.0));
        Ok(json!({"success":true}))
    }
}
#[tokio::test(flavor = "current_thread")]
async fn original_pipeline_finishes_only_after_both_deployment_notifications() {
    let input = Input::new();
    tokio::time::timeout(Duration::from_secs(3), cicd::run_16e(&input))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        *input.events.borrow(),
        [
            ("DEVELOPER", Operation::Compile),
            ("TESTER", Operation::Test),
            ("DEPLOYER", Operation::Deploy)
        ]
    );
    assert!(input.calls.borrow().is_empty());
}
#[tokio::test(flavor = "current_thread")]
async fn requesting_deployment_in_place_of_build_has_no_effect() {
    let input = Input::new();
    input
        .calls
        .borrow_mut()
        .insert("DEVELOPER", Operation::Deploy);
    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(3), cicd::run_16e(&input))
            .await
            .unwrap(),
        Err(RunError::Endpoint(_))
    ));
    assert!(input.events.borrow().is_empty());
}
