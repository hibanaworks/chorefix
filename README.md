# Hibana TraceFix experiment

This repository tests whether Hibana can enforce the communication procedures
in TraceFix with multiple GPT-6 Luna agents using xhigh reasoning.

The task set remains the full 16 scenarios at three difficulty levels, for 48
tasks. The experiment keeps the original task descriptions, tools, simulator,
and coordination checklists. It replaces coordination in the Hibana condition
with global protocols and attached endpoints.

Hibana owns coordination progress. Participant code does not keep another state
machine or flags to decide which communication step is allowed. The external
simulator owns the domain environment and checks task outcomes.

The upstream implementation is pinned to
`36b4bd108fcd74f7efb17613aebbe3fdf55499a9`. Its current runtime checks step order,
which differs from the topology-only monitor described in the paper. Results
will state the baseline revision explicitly.

## Status

All 48 tasks and 825 checklist items have a recorded byte-level source manifest.
Three concurrent GPT-6 Luna access checks completed with xhigh reasoning and
successful tool calls. These checks are recorded in
[the setup report](results/setup-report.json), separately from benchmark results.

The selected comparison includes the paper's four runtime conditions, the latest
TraceFix runtime, and Hibana. The full matrix contains 2,592 runs using one
participant model. No scored benchmark runs have started.

The Rust ports for 9E, 9M, and 9H use all three, five, and seven philosophers and
their forks. Fifteen real Luna xhigh participants completed an integration
preflight against the original simulator. Every participant called `think` and
`eat`, and the simulator recorded no resource conflicts. Raw events were checked
separately. These are setup results, not the six-condition benchmark results.

## Follow the protocol

Start with [the 9E global protocol](src/tasks/philosophers/global.rs).
It gives each fork to one philosopher, permits eating after both grants, and
requires both returns before completion. Thinking is independent for every role.
In 9E every pair of philosophers shares a fork, so eating cannot overlap. The
larger variants use `g::par` for disjoint fork intervals. Controlled tests run
two eaters in 9M and three in 9H. A delayed A does not hold back a disjoint D.

Then follow [the philosopher functions](src/tasks/philosophers/localside/philosopher.rs)
and [the fork functions](src/tasks/philosophers/localside/fork.rs). Each function
uses its endpoint for the corresponding sends and receives. The fork functions
have no owner table. The philosopher functions have no stage or completion flag.
The tests reject eating before the grants, finishing before the returns, and
consuming completion twice.

These three tasks need no approval branch or revision loop. Tasks that do need
them will use `g::route`, explicit resolvers, and `.roll()`. A resolver reads the
actual domain result at a declared choice. Localside follows the branch and the
repetition recorded in the global protocol. There is no separate approval flag
or retry state machine. The full 48-task port remains unfinished.

Model access and byte queues belong in `imp/`. Those queues carry requests and
responses. They do not decide which protocol step is allowed. The external
simulator records domain work and scores it independently. A model finishing a
turn does not complete a task.

```sh
cargo +1.95.0 test --locked
cargo +1.95.0 clippy --locked --all-targets -- -D warnings
```

Run the unscored model/simulator integration check:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.lock
cargo +1.95.0 build --locked --bin philosophers
PYTHONHASHSEED=0 .venv/bin/python tools/run_philosophers.py --tasks 9E 9M 9H --run-id check-1
.venv/bin/python tools/audit_preflight.py output/integration-preflight/check-1/9H
```

In this integration check the endpoint functions perform resource coordination.
Models receive the unchanged domain tools for their roles. This mode matches
the structural approach of the mediator condition. A tool request goes to Rust,
the endpoint checks it, and only a permitted request reaches the simulator.
The Python adapter holds RPC queues and reply IDs, with no expected-step table.
An early `eat`, a repeated `think`, or a closed participant input fails the run.
The first two errors publish no corresponding simulator execution request.

Fetch and check the fixed upstream inputs:

```sh
python3 tools/bootstrap_upstream.py
python3 tools/inventory.py --check
```

Run an access check with an explicitly selected app-server binary:

```sh
python3 tools/probe_luna.py --app-server-bin /path/to/cli --run-id access-1
```

The verified binary is the desktop application's bundled CLI,
`0.162.0-alpha.17.2`. The older PATH CLI `0.145.0` rejected GPT-6 Luna with this
account. The probe fails on a substituted model or unsuccessful turn.

## Sources

- [TraceFix paper](https://arxiv.org/abs/2605.07935)
- [TraceFix implementation](https://github.com/Sensing-And-Reasoning/TraceFix)
- [Hibana](https://github.com/hibanaworks/hibana)

TraceFix is licensed under MIT. Any retained source keeps its original license.
