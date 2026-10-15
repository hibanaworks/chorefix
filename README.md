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

[The codebase protocol](src/tasks/codebase/global.rs) covers 1E, 1M and 1H
with all three, five and seven developers. Each developer designs once, then
implements, borrows two modules, commits, returns both modules, and tests.
The 1E/1M test result is terminal, so these protocols have no retry branch.
Only 1H repeats failed work. Its resolver reads the actual tool response. `g::route` chooses another attempt
or completion, and `.roll()` permits another attempt. `g::par` keeps developers
and the two returns independent. Acquiring modules in one common order prevents
a cycle while retaining concurrent commits on disjoint modules.

Follow [the developer](src/tasks/codebase/localside/developer.rs) to see those
operations in order. [The module localside](src/tasks/codebase/localside/module.rs)
receives only projected requests. It forwards requests, grants, returns and
acknowledgements through Endpoints. [Each module's global protocol](src/tasks/codebase/global_resource.rs)
permits one outstanding loan. Borrower identity is enum message data, not a
resource state machine. In 1H both borrowers send explicit completion messages before the module closes.
In 1E/1M each borrower has one loan; the module closes after both projected loans
return. No lease request or return uses a separate channel.

A resource can be lent again as soon as its return is acknowledged. It does not
wait for its former borrower's local test. A regression test holds A's test open
while B commits using a returned module. Another test checks that a controller
cannot send against its declared resolver when a repeated choice starts again.

The host programs and tests use Tokio's current-thread runtime with async/await.
Hibana endpoints stay on that runtime. Tokio schedules I/O and futures; the
projected programs authorize communication. The host supplies descriptor memory
on the heap explicitly. This does not add an allocation fallback to Hibana.

Six tasks have Rust implementations. The full 48-task port and independent
checklist review remain unfinished. No implementation is yet accepted for the
scored six-condition comparison.

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
cargo +1.95.0 build --locked --bins
PYTHONHASHSEED=0 .venv/bin/python tools/run_preflight.py --tasks 1H 9M --scenario 2 --run-id check-1
.venv/bin/python tools/audit_preflight.py output/integration-preflight/check-1/1H
```

A real 1H preflight used seven Luna xhigh participants with two failed tests per
developer. All 70 domain calls completed, with no simulator violations and up to
three concurrent commits. Saved tool events, simulator events, source snapshots,
and the executed binary were checked independently. This is an integration
check, not a benchmark result.

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

## What this comparison can show

The current preflight gives participants explicit tool sequences and performs
resource coordination in Rust. It checks integration and enforcement, not agent
autonomy or an improvement in task completion. The simulated domain tools do not
measure the quality of real generated code or documents.

TraceFix's mediator condition also performs coordination in the runtime. Its
latest monitoring runtime checks step order as well. Enforcing an order is
therefore not, by itself, a new contribution. The Hibana implementation tests a
different execution structure: project a shared Rust choreography and drive its
local Endpoints without application progress flags or ownership tables.

The scored comparison must preserve the same work, legal decisions, resource
constraints and independent execution. It must distinguish the effect of the
protocol from the effect of its implementation. TLA+ model checking and Hibana
projection establish different properties; neither tool validates arbitrary
agent output quality. A passing preflight is not a proof of distributed deadlock
freedom or an experimental advantage.

## Shell experiment

Matching a task means preserving its goal, participants, shared resources,
required dependencies and failure conditions. It does not require giving the
model a fixed command sequence. The scored TraceFix comparison above retains
the original simulator and tool interfaces.

A separate shell experiment will test the model's freedom to do local work.
The proposed boundary is a Wasm guest running under `hibana-wasip1-runtime`,
with ChoreoFS objects answered by Hibana localside roles. The model chooses its
commands, scripts and local work. Global protocols govern shared-object effects,
resource handoffs, submission, review and publication. Every path to a protected
effect must cross that boundary; a pathname, a shell command or an approval file
cannot grant protocol authority. Refusal, waiting and retry paths must also be
explicit in the protocol.

Compare the same shell, runtime, object rights, task inputs, model instructions
and budgets with and without task-specific Hibana coordination. Both conditions
retain the same Wasm isolation and ABI checks. Only the task coordination rules
change. A TraceFix adapter may be added only when it preserves that same command
interface and decision scope. The shell experiment is separate from the frozen
2,592-run matrix because its tools and environment differ from the original
benchmark.

Measure task completion, artifact quality, unauthorized committed effects,
independent work, elapsed time and inclusive token use. Record refusals,
crashes, stalls and incomplete work as well. Rejecting every effect is not a
successful task. A valid protocol does not establish that an agent made a good
review or produced a correct artifact.

The current WASI shell is a bounded proof of concept, not a general-purpose
shell: its existing commands include `ls`, `cat`, `echo` and `apply`. The existing
demos show that ordinary guest filesystem calls follow the chosen choreography.
The multi-agent shell experiment and its full command workload are not yet
implemented or measured.

The LLM terminal reference is `hibana-pico/examples/uno-q-heterogeneous`.
Its Wasm guest writes the terminal transcript through `/llm/stdout`, reads the
model's next command through `/llm/stdin`, and performs object writes through
WASI. The global choreography combines independent context inputs with `par`
and governs the effectful turns. For this experiment, the corresponding objects
will represent private work, shared artifacts, review and publication. Task
coordination belongs in `global.rs`; each localside must show its admitted
operations directly.

The shell tests must accept different legal command sequences for the same
goal, reject unauthorized shared effects before they occur, and keep unrelated
work independent. These tests establish that the boundary enforces task
requirements without turning one demonstration's command recipe into the task.

## Sources

- [TraceFix paper](https://arxiv.org/abs/2605.07935)
- [TraceFix implementation](https://github.com/Sensing-And-Reasoning/TraceFix)
- [Hibana](https://github.com/hibanaworks/hibana)

TraceFix is licensed under MIT. Any retained source keeps its original license.
