use choreofix::tasks::research_report::{self as report, Domain, Operation, Request};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    collections::{BTreeMap, VecDeque},
    future::poll_fn,
    task::Poll,
};
// Same carrier as the integrated native implementation.
#[path = "../src/imp/transport.rs"]
mod carrier;

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
    calls: Cell<BTreeMap<&'static str, VecDeque<Call>>>,
    events: Cell<Vec<(&'static str, Operation, Value)>>,
}
impl Input {
    fn new(a: usize, b: usize) -> Self {
        let mut calls = BTreeMap::new();
        for (agent, count) in [("RESEARCHER_A", a), ("RESEARCHER_B", b)] {
            let ops = [Operation::Research, Operation::Write, Operation::References]
                .into_iter()
                .chain(std::iter::repeat_n(Operation::Revise, count));
            calls.insert(
                agent,
                ops.map(|operation| Call {
                    operation,
                    result: json!({"success":true}),
                })
                .collect(),
            );
        }
        let mut editor = VecDeque::new();
        for (operation, count) in [(Operation::ReviewA, a), (Operation::ReviewB, b)] {
            for _ in 0..count {
                editor.push_back(Call {
                    operation,
                    result: json!({"success":false,"data":{"status":"needs_revision"}}),
                });
            }
            editor.push_back(Call {
                operation,
                result: json!({"success":true,"data":{"status":"reviewed"}}),
            });
        }
        editor.push_back(Call {
            operation: Operation::Combine,
            result: json!({"success":true}),
        });
        calls.insert("EDITOR", editor);
        Self {
            calls: Cell::new(calls),
            events: Cell::default(),
        }
    }
}
impl Domain for Input {
    type Call = Call;
    async fn request(&self, agent: &'static str) -> Result<Call, String> {
        let mut calls = self.calls.take();
        let call = calls.get_mut(agent).unwrap().pop_front();
        self.calls.set(calls);
        call.ok_or_else(|| format!("no pending call for {agent}"))
    }
    async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
        let mut events = self.events.take();
        events.push((agent, call.operation, call.result.clone()));
        self.events.set(events);
        Ok(call.result)
    }
}

#[hibana_runtime::main]
#[test]
async fn independent_request_futures_and_admitted_effects_have_overlapping_intervals()
-> Result<(), choreofix::shell::ShellError> {
    struct AsyncInput {
        input: Input,
        origin: std::time::Instant,
        intervals: Cell<Vec<Value>>,
    }
    impl Domain for AsyncInput {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            let start = self.origin.elapsed().as_nanos();
            let call = self.input.request(agent).await?;
            if call.operation == Operation::Research || call.operation == Operation::ReviewA {
                hibana_runtime::host::sleep(std::time::Duration::from_millis(25))
                    .await
                    .expect("test timer");
                let mut intervals = self.intervals.take();
                intervals.push(json!({
                    "kind":"request", "agent":agent, "start_ns":start,
                    "end_ns":self.origin.elapsed().as_nanos()
                }));
                self.intervals.set(intervals);
            }
            Ok(call)
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            let start = self.origin.elapsed().as_nanos();
            if (agent == "RESEARCHER_A" && call.operation == Operation::References)
                || (agent == "RESEARCHER_B" && call.operation == Operation::Write)
            {
                hibana_runtime::host::sleep(std::time::Duration::from_millis(25))
                    .await
                    .expect("test timer");
                let mut intervals = self.intervals.take();
                intervals.push(json!({
                    "kind":"effect", "agent":agent, "operation":format!("{:?}",call.operation),
                    "start_ns":start, "end_ns":self.origin.elapsed().as_nanos()
                }));
                self.intervals.set(intervals);
            }
            self.input.execute(agent, call).await
        }
    }
    let input = AsyncInput {
        input: Input::new(0, 0),
        origin: std::time::Instant::now(),
        intervals: Cell::default(),
    };
    let frames = hibana_runtime::race(
        async { Ok::<_, std::io::Error>(report::run_3e(&input).await) },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(3)),
    )
    .await
    .expect("independent futures were not polled")
    .unwrap();
    let intervals = input.intervals.take();
    for kind in ["request", "effect"] {
        let spans: Vec<_> = intervals.iter().filter(|v| v["kind"] == kind).collect();
        assert_eq!(spans.len(), if kind == "request" { 3 } else { 2 });
        assert!(
            spans
                .iter()
                .map(|span| span["start_ns"].as_u64().unwrap())
                .max()
                .unwrap()
                < spans
                    .iter()
                    .map(|span| span["end_ns"].as_u64().unwrap())
                    .min()
                    .unwrap(),
            "{kind} futures did not overlap: {spans:?}"
        );
    }
    println!(
        "native_async_evidence={}",
        json!({
            "task":"3E", "runtime":"hibana-runtime scoped", "model_calls":false,
            "stimulus":"25 ms asynchronous test domain, no barrier", "intervals":*intervals,
            "frames":frames
        })
    );

    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn both_original_authors_reconsider_until_their_actual_reviews_pass()
-> Result<(), choreofix::shell::ShellError> {
    for (a, b) in [(0, 0), (2, 1), (1, 2)] {
        let input = Input::new(a, b);
        hibana_runtime::race(
            async { Ok::<_, std::io::Error>(report::run_3e(&input).await) },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(3)),
        )
        .await
        .expect("report protocol stalled")
        .unwrap();
        assert!(input.calls.take().values().all(VecDeque::is_empty));
        let events = input.events.take();
        assert_eq!(
            events.iter().filter(|e| e.1 == Operation::Revise).count(),
            a + b
        );
        assert_eq!(
            events.iter().filter(|e| e.1 == Operation::Combine).count(),
            1
        );
        assert_eq!(events.len(), 9 + 2 * (a + b));
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn document_write_and_other_authors_reference_update_overlap()
-> Result<(), choreofix::shell::ShellError> {
    struct Concurrent {
        input: Input,
        started: Cell<Vec<(&'static str, Operation)>>,
    }
    impl Domain for Concurrent {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            self.input.request(agent).await
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            if (agent == "RESEARCHER_A" && call.operation == Operation::References)
                || (agent == "RESEARCHER_B" && call.operation == Operation::Write)
            {
                let mut started = self.started.take();
                started.push((agent, call.operation));
                self.started.set(started);
                poll_fn(|cx| {
                    cx.waker().wake_by_ref();
                    Poll::Ready(())
                })
                .await;
                poll_fn(|_| {
                    let started = self.started.take();
                    let ready = [
                        ("RESEARCHER_A", Operation::References),
                        ("RESEARCHER_B", Operation::Write),
                    ]
                    .iter()
                    .all(|item| started.contains(item));
                    self.started.set(started);
                    if ready {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
            }
            self.input.execute(agent, call).await
        }
    }
    let domain = Concurrent {
        input: Input::new(0, 0),
        started: Cell::default(),
    };
    hibana_runtime::race(
        async { Ok::<_, std::io::Error>(report::run_3e(&domain).await) },
        hibana_runtime::host::deadline(std::time::Duration::from_secs(3)),
    )
    .await
    .expect("independent resource effects were serialized")
    .unwrap();

    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn section_review_and_revision_do_not_wait_for_unrelated_work()
-> Result<(), choreofix::shell::ShellError> {
    struct IndependentEffects {
        input: Input,
        pair: [(&'static str, Operation); 2],
        started: Cell<Vec<(&'static str, Operation)>>,
    }
    impl Domain for IndependentEffects {
        type Call = Call;
        async fn request(&self, agent: &'static str) -> Result<Call, String> {
            self.input.request(agent).await
        }
        async fn execute(&self, agent: &'static str, call: Call) -> Result<Value, String> {
            if self.pair.contains(&(agent, call.operation)) {
                let mut started = self.started.take();
                started.push((agent, call.operation));
                self.started.set(started);
                poll_fn(|cx| {
                    cx.waker().wake_by_ref();
                    Poll::Ready(())
                })
                .await;
                poll_fn(|_| {
                    let started = self.started.take();
                    let ready = self.pair.iter().all(|item| started.contains(item));
                    self.started.set(started);
                    if ready {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
            }
            self.input.execute(agent, call).await
        }
    }
    for (revisions, pair) in [
        (
            0,
            [
                ("EDITOR", Operation::ReviewA),
                ("RESEARCHER_B", Operation::Write),
            ],
        ),
        (
            1,
            [
                ("RESEARCHER_A", Operation::Revise),
                ("RESEARCHER_B", Operation::References),
            ],
        ),
    ] {
        let domain = IndependentEffects {
            input: Input::new(revisions, 0),
            pair,
            started: Cell::default(),
        };
        hibana_runtime::race(
            async { Ok::<_, std::io::Error>(report::run_3e(&domain).await) },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(3)),
        )
        .await
        .unwrap_or_else(|_| panic!("independent effects serialized: {pair:?}"))
        .unwrap();
        assert!(domain.input.calls.take().values().all(VecDeque::is_empty));
    }
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn early_combine_has_no_domain_effect() -> Result<(), choreofix::shell::ShellError> {
    let input = Input::new(0, 0);
    let mut calls = input.calls.take();
    calls.get_mut("EDITOR").unwrap().push_front(Call {
        operation: Operation::Combine,
        result: json!({"success":true}),
    });
    input.calls.set(calls);
    assert!(
        hibana_runtime::race(
            async { Ok::<_, std::io::Error>((report::run_3e(&input)).await) },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(3))
        )
        .await
        .unwrap()
        .is_err()
    );
    assert!(
        input
            .events
            .take()
            .iter()
            .all(|e| e.1 != Operation::Combine)
    );

    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn global_blocks_second_document_loan_before_first_return()
-> Result<(), choreofix::shell::ShellError> {
    use hibana::runtime::{SessionKitStorage, ids::SessionId};
    use report::global::{self as g, author_a as a, author_b as b};
    let p = g::project_3e();
    let mut slab = vec![0; 256 * 1024];
    let transport = carrier::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &transport).unwrap();
    let session = SessionId::new(48);
    let mut author_a = rv.enter(session, &p.a).unwrap();
    let mut author_b = rv.enter(session, &p.b).unwrap();
    let mut document = rv.enter(session, &p.document).unwrap();
    author_a.send::<g::Research>(&()).await.unwrap();
    author_b.send::<g::Research>(&()).await.unwrap();
    document.send::<a::DocAvailable>(&()).await.unwrap();
    author_a.recv::<a::DocAvailable>().await.unwrap();
    author_a.send::<a::DocRequest>(&()).await.unwrap();
    document.recv::<a::DocRequest>().await.unwrap();
    document.send::<a::DocGranted>(&()).await.unwrap();
    assert!(
        document.send::<b::DocAvailable>(&()).await.is_err(),
        "global allowed another DOCUMENT loan before the first return"
    );
    assert!(!transport.take_observations().is_empty());

    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn global_blocks_second_database_loan_before_first_return()
-> Result<(), choreofix::shell::ShellError> {
    use hibana::runtime::{SessionKitStorage, ids::SessionId};
    use report::global::{self as g, author_a as a, author_b as b};
    let p = g::project_3e();
    let mut slab = vec![0; 256 * 1024];
    let transport = carrier::InMemoryTransport::new([]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &transport).unwrap();
    let session = SessionId::new(50);
    let mut author_a = rv.enter(session, &p.a).unwrap();
    let mut author_b = rv.enter(session, &p.b).unwrap();
    let mut document = rv.enter(session, &p.document).unwrap();
    let mut database = rv.enter(session, &p.database).unwrap();
    author_a.send::<g::Research>(&()).await.unwrap();
    author_b.send::<g::Research>(&()).await.unwrap();
    document.send::<a::DocAvailable>(&()).await.unwrap();
    author_a.recv::<a::DocAvailable>().await.unwrap();
    author_a.send::<a::DocRequest>(&()).await.unwrap();
    document.recv::<a::DocRequest>().await.unwrap();
    document.send::<a::DocGranted>(&()).await.unwrap();
    author_a.recv::<a::DocGranted>().await.unwrap();
    author_a.send::<g::Write>(&()).await.unwrap();
    author_a.send::<a::DocReturn>(&()).await.unwrap();
    document.recv::<a::DocReturn>().await.unwrap();
    document.send::<a::DocReturned>(&()).await.unwrap();
    author_a.recv::<a::DocReturned>().await.unwrap();
    author_a.send::<a::RefsRequest>(&()).await.unwrap();
    database.recv::<a::RefsRequest>().await.unwrap();
    database.send::<a::RefsGranted>(&()).await.unwrap();
    author_a.recv::<a::RefsGranted>().await.unwrap();
    assert!(
        database.send::<b::RefsAvailable>(&()).await.is_err(),
        "global allowed another DATABASE loan before the first return"
    );

    Ok(())
}
