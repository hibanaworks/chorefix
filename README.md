# ChoreoFix

This repository tests whether Hibana can enforce the communication procedures
in TraceFix with multiple GPT-6 Luna agents using xhigh reasoning.

The task set remains the full 16 scenarios at three difficulty levels, for 48
tasks. The experiment keeps the original task inputs, tools, simulator,
participants, resources and outcome checks. The original checklists remain
recorded source material. Hibana can improve the coordination procedure rather
than reproduce a legacy control algorithm. For example, independent resource
requests run in parallel, with both grants required before writing. Procedural
differences are recorded before scored runs; the comparison runtimes are unchanged.
Global protocols and attached endpoints own the Hibana coordination.

Hibana owns coordination progress. Participant code does not keep another state
machine or flags to decide which communication step is allowed. The external
simulator owns the domain environment and checks task outcomes.

The writing tasks 2E/2M/2H now have a direct WASI boundary qualification with
all 3/5/7 researchers and physical sections in one global. Both independent
acquisition requests are in flight before either grant. Disjoint writes overlap
in groups of 1/2/3. The [saved evidence](results/validation/writing-direct-wasi-20261015)
includes a terminal review failure, original simulator journals and Rust audits
that reject serialized acquisitions. These are deterministic tests with no model
calls or scored comparison episodes.

Codebase tasks 1E/1M now use the same direct WASI boundary with three/five
developers and physical modules in one global. Eight Luna xhigh participants
completed 32 original domain operations with no resource conflicts.
The [saved trials](results/validation/codebase-luna-20261016/evidence.md)
separate observed model concurrency, domain concurrency and shared commits.
These remain unscored qualification trials.

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

The [16E global protocol](src/tasks/cicd/choreofs/global.rs) connects each WASI
role directly to its service owner. The
[WASI localside](src/tasks/cicd/choreofs/localside/wasi.rs) sends the actual
open/write/close requests through its Endpoint. The owner receives them, uses
ChoreoFS to materialize object operations, consumes the resolved local operation,
and returns the result through the same projected protocol. There is no
channel forwarding requests between separate protocol graphs.

One Endpoint mechanism test and six real-WASI tests cover invalid arguments,
corrected work, early dependants, duplicate effects, guest exit, and shell scripts.
They use the independent [hibana-wash](https://github.com/hibanaworks/hibana-wash)
guest pinned by `vendor/hibana-wash`. Resource loans, build/test notifications,
and both deployment-complete notifications remain explicit messages. Closed
services answer errors while downstream work continues. A carrier trace checks
that each close arrives from its WASI role before the shared effect, and that
the actual result returns to that role. Missing requests and altered sources
fail the audit. This validates this connection on 16E, not all 48 task ports.

The former rust-bash candidate exceeded the runtime's capacities and was
retired. Its [qualification record](results/validation/bash-guest-qualification.json)
remains as failed evidence. Wash uses Rust `std` with no external Rust
dependencies or guest FS. All file operations go through WASI, projected
Endpoints, and ChoreoFS. Each guest starts with 64 KiB of committed memory and
can request more through the projected memory-growth protocol, up to its
1 MiB caller-supplied backing.
Four unscored Luna attempts on this direct boundary are retained in
[the run ledger](results/direct-wasi-preflight.json), including two integration
failures and a preliminary recording error. The final component attempt passed
[a separate raw-event audit](results/audits/direct-wasi-wash-std-004-16E.json).
This is one component check on 16E, which has no injected domain failures.
That earlier audit checks component outcomes, not the corrected connection above.
It does not establish task fidelity across all 48 tasks or a comparative result.

A new Luna xhigh trial uses the corrected connection. All three participants
completed, with three domain operations and all four notifications.
[The independent audit](results/audits/projected-wasi-005-16E.json) checks the
actual request/reply carrier trace as well as source, model, and token records.
This is one unscored 16E integration trial, not a success-rate comparison.
See [the work plan](plan.md) for the remaining experiment gates.

The [6E global protocol](src/tasks/review/choreofs/global.rs) connects the two
developers, reviewer services, WASI roles, and one shared repository owner.
The [localsides](src/tasks/review/choreofs/localside) keep each Endpoint operation
visible. Review results choose approval or revision through explicit resolvers.
The protocol permits independent development and review, and repeats rejected
work without application progress flags. Real-WASI tests passed with zero, one,
and two revisions. [The retained evidence](results/validation/static-controller-continuation-20261015)
includes original-simulator replay, shared-resource checks, and core regressions.
Three later [Luna xhigh trials](results/review-direct-wasi-preflight.json) used
this direct connection at failure depths 0, 1, and 2. All nine participants
completed. Independent audits replay the original simulator, check resource
intervals and both notification channels, and match returned feedback to the
actual review. The trials used 1,171,689 inclusive tokens in total. They remain
unscored integration evidence, with no baseline contrast or success-rate claim.
At depth 2 each feature was revised once, as chosen by the participants. This
differs from the deterministic cases that revise one feature twice.
Cargo keeps the 0.9.9 dependency and pins unreleased core repairs from
[Hibana PR 8](https://github.com/hibanaworks/hibana/pull/8).

The same repaired core passed all 44 distinct QUIC case/direction cells in
[one external interoperability run](results/validation/interop-static-controller-20261015).
The saved audit checks the individual phase results, expected matrix, and exact
core/TLS pins. Reference self-tests were omitted. The runner and capture-shutdown
patches are disclosed in the evidence. The core's separate full GitHub gate
also completed successfully at c2001ec.

Later duplicate-request tests exposed an application error in the review guest:
an `offer()` on the shared caller endpoint could observe the other service's
still-live branch. The current global gives each review service its own caller
role. One guest directly drives both endpoints and its private-I/O endpoint in
that same global. All six boundary cases now include post-completion duplicate
requests and chunked JSON writes. No response-routing table or progress flags
are added. The earlier model trials retain their original source checkpoint.

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

[The direct codebase protocol](src/tasks/codebase/choreofs/global.rs) connects
1E/1M's WASI callers, service owners, physical modules and private I/O in one
global and session. Each developer designs, implements, requests both modules,
waits for both grants, commits, returns both modules and tests. Both requests
precede either grant wait, and both returns precede either acknowledgement wait.
The [localsides](src/tasks/codebase/choreofs/localside/) show those Endpoint
operations directly. Service input errors follow explicit rejection routes and
repeated requests. The test result is terminal, including failure.

The deterministic 1M trial observes two disjoint commits together and allows
B to commit while A's local test waits. The live five-person trial has four
admitted domain calls in flight, but its shared commits do not overlap because
the disjoint developers reach them at different times. Both observations are
retained. The original sequential-acquisition checklist procedure is replaced
by parallel requests; the original resource and task constraints are preserved.

[The earlier native codebase protocol](src/tasks/codebase/global.rs) also has
1H's result-driven reconsideration loop. Its resolver reads the actual test
response, `g::route` chooses another attempt or completion, and `.roll()` permits
another attempt. This native foundation does not qualify 1H's actual WASI path,
which still needs to be connected.

The host does not replay a failed tool call. It returns the actual failure and
waits for new model requests. The model is asked to reconsider its fix and state
a brief revised approach. Hibana constrains the resulting communication order,
not the model's thought process. The original tools simulate work and accept no
source patch, so this benchmark cannot measure the quality of a reconsidered fix.

The new [physical module localside](src/tasks/codebase/choreofs/localside/module.rs)
has its two explicit projected visits. It holds no borrower table. Module loans
and service requests share the task's global; the measured 1E/1M condition does
not use the earlier native resource-broker sessions.

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
`hibana-wasip1-runtime`, pinned to `4ebdad1443cf49488eae924afe528f67f5a5953b`.
Both the experiment and runtime use published Hibana `=0.9.9`. The runtime
source is saved in that public commit; there is no mutable sibling core patch.

Every file operation goes through WASI and ChoreoFS. This includes terminal I/O,
work files, scripts, and the tool request/reply objects. The guest has no second
filesystem and no host filesystem access. The answering role owns file contents
and offsets. ChoreoFS provides object and fd facts. Neither records task progress.

In 16E, each guest can read and write its own `/work` objects and read its own
`/service/result` at any time. Those namespaces are separate between guests;
the result object is read-only. The task constraints govern the effectful
build/test/deploy services. This test does not establish a task-dependent gate
on shared reads. Tasks that require one must include the read and its
dependencies in their global protocol.

For the direct 16E boundary, read [the composed global](src/tasks/cicd/choreofs/global.rs),
[guest localside](src/tasks/cicd/choreofs/localside/wasi.rs),
[private I/O localside](src/shell/localside/private_io.rs), and
[task localsides](src/tasks/cicd/choreofs/localside/).
The global governs shared service requests, dependencies, and private WASI I/O.
The receiving role consumes the permitted local step before applying a shared
effect. A pathname, fd right or WASI write does not replace that authorization.
Memory growth also uses an explicit request and reply in the same projected
protocol. Shell computation and private file contents hold no task authority.

The older [WASI global](src/shell/global.rs),
[guest](src/shell/localside/guest.rs) and
[answering role](src/shell/localside/environment.rs) remain for historical
preflight regressions. Their early-`eat` test rejects the request before the
simulator receives an execute request.

The older integration guest supports `help`, `ls`, `cat`, `echo`, `apply`,
`script`, and `call`. It remains for recorded preflight regressions. The direct
16E boundary uses the separate hibana-wash guest described above.
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
git submodule update --init --recursive
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
