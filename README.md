# ChoreoFix

This repository tests whether Hibana can enforce the communication procedures
in TraceFix with multiple GPT-6 Luna agents using xhigh reasoning.

The task set remains the full 16 scenarios at three difficulty levels, for 48
tasks. The experiment keeps the original task descriptions, tools, simulator,
and coordination checklists. It replaces coordination in the Hibana condition
with global protocols and attached endpoints.

Hibana owns coordination progress. Participant code does not keep another state
machine or flags to decide which communication step is allowed. The external
simulator owns the domain environment and checks task outcomes.

The original four TraceFix runtime conditions are pinned to the first public
release `87b01a2c0ae53c5045c9ce3d43cd0235b1831275`. Their coordination logic is
unchanged. The recorded benchmark inputs are at
`36b4bd108fcd74f7efb17613aebbe3fdf55499a9`; their descriptions and tool metadata
are byte-identical to the paper release. A separate 432-episode condition uses
the unchanged latest monitoring runtime at that revision. It uses the same
48 tasks, three failure depths and three repetitions, paired with the Hibana
episodes from the main matrix. The total plan has 2,592 episodes. Neither
comparison has scored results yet.

## Research draft

**ChoreoFix: Enforcing Multi-Agent Coordination from Global Protocols** uses
Fix to mean fixing the permitted coordination relation. It does not claim
automatic protocol repair. [The draft](paper/choreofix.tex) states the mechanism,
trust boundary, measured integration results, and unfinished evaluation.

## Status

All 48 tasks and 825 checklist items have a recorded byte-level source manifest.
Three concurrent GPT-6 Luna access checks completed with xhigh reasoning and
successful tool calls. These checks are recorded in
[the setup report](results/setup-report.json), separately from benchmark results.

The selected comparison includes the paper's four runtime conditions and
Hibana. The full matrix contains 2,160 runs using one
participant model. No scored benchmark runs have started.
The latest TraceFix monitoring runtime adds 432 paired episodes. Its upstream
code is unchanged. [The source assessment](results/latest-tracefix.json) records
the revision, existing tests, and work still needed before scored runs.

The Rust ports for 9E, 9M, and 9H use all three, five, and seven philosophers and
their forks. Fifteen real Luna xhigh participants completed an integration
preflight against the original simulator. Every participant called `think` and
`eat`, and the simulator recorded no resource conflicts. Raw events were checked
separately. These are setup results, not the five-condition benchmark results.

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

The host does not replay a failed tool call. It returns the actual failure and
waits for new model requests. The model is asked to reconsider its fix and state
a brief revised approach. Hibana constrains the resulting communication order,
not the model's thought process. The original tools simulate work and accept no
source patch, so this benchmark cannot measure the quality of a reconsidered fix.

Follow [the developer](src/tasks/codebase/localside/developer.rs) to see those
operations in order. [The module localside](src/resource/localside.rs)
receives only projected requests. It forwards requests, grants, returns and
acknowledgements through Endpoints. [Each module's global protocol](src/resource/global.rs)
permits one outstanding loan. The broker records which request arrived. The resource needs only an ordinary
request, grant, return and acknowledgement; it stores no borrower identity. In 1H both borrowers send explicit completion messages before the module closes.
In 1E/1M each borrower has one loan; the module closes after both projected loans
return. No lease request or return uses a separate channel.

A resource can be lent again as soon as its return is acknowledged. It does not
wait for its former borrower's local test. A regression test holds A's test open
while B commits using a returned module. Another test checks that a controller
cannot send against its declared resolver when a repeated choice starts again.

[The writing protocol](src/tasks/writing/global.rs) covers 2E, 2M and 2H,
with all three, five and seven researchers and shared sections. Literature and
drafting are local. Each shared write needs two section grants. Researchers
return the sections before reviewing their own work. Review is terminal at all
three levels, even when it finds issues. The original simulator disables
failure injection for this family; setting scenario depth 2 does not create
review failures. [The researcher functions](src/tasks/writing/localside/researcher.rs)
show these operations directly. The same resource protocol handles modules and
sections. Tests keep disjoint writes concurrent and allow another researcher
to write while a former borrower is still reviewing.

[The CI/CD protocol](src/tasks/cicd/global.rs) covers the original 16E pipeline.
It includes the developer, tester, deployer, build server and deployment
environment in one global. The developer releases the build server before
notifying the tester. Deployment follows the tests-passed message and ends by
notifying both other participants. The independent final notifications use
`g::par`. Each [localside function](src/tasks/cicd/localside/) follows its role
explicitly. No separate progress or resource state is kept. The original 16E
simulator disables failure injection and supplies combined build/test and
deploy/smoke-test tools. This port preserves that behavior. 16M and 16H remain
unimplemented.

[The API and database protocol](src/tasks/api/global.rs) covers 8E. Migration
precedes backend work, which precedes one integration test. A resolver reads the
actual test result and chooses distinct pass or fail notifications to both
other participants. The two notifications use `g::par`. The
[localsides](src/tasks/api/localside/) show each loan, effect and notification
directly. Both outcomes end the protocol. A failed test is a domain failure,
not a successful benchmark result. The description and simulator specify this
terminal failure, while checklist O2 asks for retries. This source conflict is
recorded in [the fidelity review](results/task-fidelity/8E.json); acceptance
remains open.

Task localsides match the tool request that actually arrived. Each handler
shows its Endpoint sends and receives directly, before the corresponding domain
effect. No helper performs these communication steps, and there is no stored
expected step, phase, completion flag, or owner table. `Operation` values name
actual incoming tools. The retry resolver reads the actual tool response.
Neither stores the next permitted step; the Endpoint holds that progress.

A model can pause before requesting a commit without holding the modules.
A regression test withholds A's commit request until B commits on a shared
module. This prevents the application from holding resources while waiting for
its next model request. File offsets, WASI flags and RPC reply identifiers stay
in the I/O layer and do not authorize task work.

The host programs and tests use Tokio's current-thread runtime with async/await.
Hibana endpoints stay on that runtime. Tokio schedules I/O and futures; the
projected programs authorize communication. The host supplies descriptor memory
on the heap explicitly. This does not add an allocation fallback to Hibana.

Eleven tasks have Rust implementations. The full 48-task port and independent
checklist review remain unfinished. No implementation is yet accepted for the
scored five-condition comparison.

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

A separate shell integration uses a real Wasm guest under
`hibana-wasip1-runtime`, pinned to `7905fa07bfd66db69ea7fb74883b7ca1e2b572bd`.
Both the experiment and runtime use published Hibana `=0.9.9`. The local updated
runtime source is saved in that commit; there is no mutable sibling core patch.

Every file operation goes through WASI and ChoreoFS. This includes terminal I/O,
work files, scripts, and the tool request/reply objects. The guest has no second
filesystem and no host filesystem access. The answering role owns file contents
and offsets. ChoreoFS provides object and fd facts. Neither records task progress.

Read [the WASI global protocol](src/shell/global.rs) and its
[guest](src/shell/localside/guest.rs) and
[answering role](src/shell/localside/environment.rs). They govern ABI request and
reply order. The task-specific globals and localsides above still authorize the
actual domain work. A pathname, approval file or WASI write cannot replace that
authorization. The test sends an early `eat` through the real guest and ChoreoFS;
the task endpoint rejects it before an execute request reaches the simulator.

The shell supports `help`, `ls`, `cat`, `echo`, `apply`, `script`, and `call`.
Eight bounded work objects hold notes or scripts. A `call` writes its original
tool arguments to `/tools/call` and reads the actual result from `/tools/reply`.
The shell does not run arbitrary POSIX binaries or provide a Rust compiler.
It demonstrates choice among command sequences within this recorded profile,
not unrestricted program execution. No fixed command script is given to the
model. The original task description and input data define its work.

This integration is separate from the 2,160-run comparison. TraceFix keeps its
original interfaces. Completion rates from these different interfaces cannot
isolate the effect of coordination. A future causal shell comparison would
require the same guest and object rights in both conditions.

Measure task completion, artifact quality, unauthorized committed effects,
independent work, elapsed time and inclusive token use. Record refusals,
crashes, stalls and incomplete work as well. Rejecting every effect is not a
successful task. The original simulator checks task effects; it does not assess
arbitrary source-code quality. The shell adds no such quality guarantee.

Build and test the actual boundary:

```sh
bash tools/build_shell.sh
PYTHONPATH=. .venv/bin/pytest -q tests/test_shell.py
.venv/bin/python tools/run_shell_preflight.py --tasks 1H 9M --scenario 2 --run-id wasi-choreofs-001
```

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

## Measured shell integration

[The independently audited results](results/shell-preflight.json) retain every
saved shell episode. The latest implementation batch, `direct-localside-001`,
used GPT-6 Luna xhigh and the direct localsides above:

| Task | Participants | Domain calls | Max. shared users | Seconds | Inclusive tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| 9M | 5 | 10 | 2 | 23.67 | 294,684 |
| 1H | 7 | 70 | 3 | 55.68 | 1,411,942 |
| 2H | 7 | 28 | 3 | 30.16 | 640,499 |
| 16E | 3 | 3 | 1 | 41.21 | 271,576 |

All four tasks completed in the unchanged simulator with zero recorded resource
violations. Each 1H developer saw two failed tests before success. All task and
WASI processes exited successfully. The source and binary snapshots, raw model
calls, task requests and simulator events were checked separately.

In 16E, a model also requested four undeclared notification or discovery tools.
The static tool schema boundary rejected them before task RPC or simulator
execution. They remain in the logs and count toward model calls and tokens.
These are schema rejections, not Hibana protocol rejections. The declared
notifications were sent by the localsides through Endpoints.

The models chose help commands, listings and file reads. None chose a script
in this batch. Real script execution and private work writes are covered by the
boundary tests and earlier retained probes. These unscored integration results
do not estimate a success, speed, cost or maintenance advantage.

Two later 8E shell checks used three Luna xhigh participants each. Depth 0
completed the domain task in 21.67 seconds with 152,527 inclusive tokens. Depth 2
ended after a failed test in 29.21 seconds with 170,586 tokens. Both completed
the projected protocol, made exactly three domain calls, and recorded no
resource violations. The failed episode remains a domain failure. Both raw
records and their source snapshots passed the independent event audit.

The [WASI quality workflow](https://github.com/hibanaworks/hibana-wasip1-runtime/actions/workflows/quality.yml)
and [experiment quality workflow](https://github.com/hibanaworks/chorefix/actions/workflows/quality.yml)
check their respective runtime boundaries. The experiment CI issues no model
calls.

## Sources

- [TraceFix paper](https://arxiv.org/abs/2605.07935)
- [TraceFix implementation](https://github.com/Sensing-And-Reasoning/TraceFix)
- [Hibana](https://github.com/hibanaworks/hibana)

TraceFix is licensed under MIT. Any retained source keeps its original license.
