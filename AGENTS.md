# Experiment rules

## Local validation memory budget

Do not use Docker or start Docker Desktop, a daemon or a VM for measurement.
The measurement core must use no_std, no heap allocation and caller-supplied
storage. Keep host reporting separate. Distinguish CPU instructions, logical
work counts and elapsed time; do not require Linux to validate the kernel.

The user's Mac must not be driven over 4 GB by our validation workloads.
Run builds and checks through Hibana's `.github/scripts/lib/resource_guard.sh`.
It serializes local workloads, measures all descendants including CBMC/LLVM,
and kills the owned workload at 3,000,000,000 bytes with 100 ms sampling.
On macOS use the larger of RSS and physical footprint per process. Cargo and
Kani jobs are 1. `check_kani.sh` includes the guard and must be called directly.
Do not nest an outer resource guard around it. Do not run other heavy checks
beside it or rely on `ulimit -m`, which does not provide this protection.
Save budget stops as interrupted verification. Reduce proof formula size or
split equivalent obligations without reducing the proof domain or safety checks.
Never raise the budget to make a proof pass.

After every successful build, remove superseded binaries, unused target
directories, temporary logs and caches. Delete them rather than compressing
them into archives. Keep the artifacts and cache still used by the current
work, reusable profilers, necessary raw experiment data and source hashes, and
evidence for unresolved failures. Do not delete another task's files or an
active build's storage.

## Mandatory design constraints

Always prefer a simpler, more coherent design, including breaking changes when
they improve the ownership model and preserve the required guarantees. Existing
code and past plans are not reasons to retain a worse design. Replace obsolete
instructions when the design changes instead of appending conflicting rules.
Remove superseded structures rather than retaining aliases, wrappers, modes,
fallbacks, or a second execution path for compatibility unless the user explicitly
requires it. Renaming types or moving helpers is not an architectural improvement.
Evaluate understanding cost, guarantees, expressiveness, concurrency, compilation
cost and physical size. A rewrite alone is not evidence of improvement.
Preserve atomicity and safety; do not weaken checks or tests to simplify a design.

**No helper/wrapper hiding of localside flow. No separate state machine. No
task progress flags. No duplicate coordination or completion management.**

Never work around a core, projection or runtime-checking defect by arranging the
localside differently. Reordering operations, adding notifications or receives,
special-case branches, serialization, helpers or flags to make a legal global
work is forbidden. Fix Hibana when a legal global permits or rejects the wrong
operation. Fix the global when its coordination specification is wrong; write
the localside directly from that corrected projection. Remove existing
workarounds at their cause without weakening regression expectations or guarantees.

Do not use `Rc`, `Arc`, `RefCell` or `Mutex` to share application resources,
file contents or coordination state. Give the data to one participant and send
requests, results and resource Endpoints through the global protocol. A resolver
may inspect its actual decision input; it must not acquire another workflow
cursor. Transport storage and external RPC correlation also need explicit owners
and must not become a second route for task permissions. Existing occurrences
are migration work, not an accepted implementation or a reason to add more.

Write coordination with Hibana `seq`, `send`, `route`, resolvers, `par`, and
`roll`. Hibana's projected runtime is the sole authority for task progress and
permission; Endpoint is the affine view used to perform its logical role's steps.
Keep the actual Endpoint operations directly visible in each localside.
The globals use `par`, `route`, explicit resolvers, `roll`, `seq`, and `send`;
the localsides use the published `offer`, `send`, and `recv` directly. Tokio
current-thread async/await is the executor. Do not introduce a new API name or
move phase selection into a generic guest driver to work around an ambiguous
rolled request. Distinguish the projected communication and ownership instead.
Do not move ordering, authorization, retry, branch, or completion decisions into
a helper, wrapper, driver, callback, filesystem, or transport layer.
Do not recreate them as a stage enum, program counter, allowed-operation table,
approved/done boolean, owner record, or disguised equivalent.

Low-level codecs, buffers, and object materialization belong in `imp/`; they do
not acquire choreography authority by being placed there. Review this rule in
the live source and verification evidence before reporting completion.

Build task globals from actual business requests, resource Endpoint transfers,
results and returns. Write their localsides from that global. Do not retain a
token-era workflow and replace individual grants with Endpoint payloads.
Remove replaced native coordinators and their admission tables; test the evaluated
delegated protocol rather than keeping a second implementation for qualification.
Private file operations remain in the WASI syscall choreography; completing a
file operation or one rolled visit does not establish business completion.
Place approval-dependent effects and successful termination inside the
corresponding result branch. A continuation after an elastic `roll` does not
establish that a particular outcome occurred inside that roll.

Application code denies unsafe code. The sole exception is initial
`EndpointBinding::new()` at `src/shell/imp/bus.rs`, where the CLI establishes
one fresh simulator trial and distinct physical resource bindings. Keep that
allowance on the construction statement only. All runtimes reaching a resource
share the same binding. Moving or dropping an Endpoint and starting another
session must not mint a new binding. This is the physical integration boundary,
not another source of task permissions.

The experiment covers all 48 TraceFix tasks. Keep the comparison implementations
unchanged. In ChoreoFix, preserve the task goals, participants, resource safety
and necessary data dependencies, while improving the coordination procedure
where the original procedure needlessly restricts independent work or recovery.
Record each procedural change before scoring and report the unchanged upstream
score separately from any additional ChoreoFix criterion. Do not turn a hard
task into a terminal single attempt merely to make its port pass.

Give the model freedom to choose commands, private computations and ways to
solve the task. Protect shared effects with the projected choreography, not a
command recipe or prompt compliance. After a recoverable rejection or failed
attempt, leave a route to reconsider, revise and try again. Express the choice
with `route`, connect an actual result or local decision with its resolver, use
`roll` for repeated work and `par` for independent work. Show the corresponding
`offer`, `send` and `recv` directly in the localsides. A resolver may read the
decision's input; it must not become another workflow cursor or completion table.

Model participants use the existing ChatGPT subscription through Codex App
Server, with `gpt-6-luna` and `xhigh` checked explicitly. Require ChatGPT login
and never fall back to API-key billing, a paid provider, or another model.
Do not request an API key. Keep the comparison runtimes unchanged and qualify
their model connection separately before scoring. Local response fixtures do
not establish that a live comparison connection works.

Hibana global protocols are the only source of coordination rules in the Hibana
condition. The projected runtime owns protocol progress. Do not add a second state machine,
program counter, approved/done flags, lock-owner table, or pending-step tracker
to authorize coordination. Do not serialize independent work to make a task pass.

Keep each task's protocol in `src/tasks/<task>/global.rs` and its participant
functions in `src/tasks/<task>/localside/`. Keep transport, model access, and
domain integration in `imp/`. The global protocol and each role's localside
must be traceable like hibana-quic. Use the actual
published Hibana API. Do not modify Hibana core to add agent-specific APIs.

Check API examples against the current method definitions. The localside uses
`endpoint.send::<M>(...)`, `recv` and `offer`, and `RouteBranch::label`, `send`
and `recv`. Stale design comments do not authorize adding or renaming methods.
Label owned-payload examples as proposed until the real send/recv path works.

If a suspected Hibana bug appears, retain a minimal reproduction and report the
affected version and exact observed behavior. Check the remote current source
before calling it a current bug. Fix confirmed defects in Hibana itself; an
application workaround is not completion. Keep the reproducing regression test
and verify the repair with Lean, Z3, Miri, and Rust tests. Use Kani where bounded
checks of the affected boundary or transition are useful. Check its impact on
the current hibana-quic with the fixed core actually selected. Keep
unrelated/dirty working copies intact. Do
not claim these gates from stale logs or an unpatched consumer build.

Hibana core remains `#![no_std]` with no heap allocation or normal dependencies.
Pico is the size baseline. If required functionality exceeds it in actual size
measurements, the user permits Pico 2 as the core target. Preserve guarantees
and expressiveness, use one API across embedded and host builds, and qualify
stack, SRAM and flash for the selected target. This is not permission for
unbounded storage or arbitrary fixed capacities.
Necessary costs for the current ownership and concurrency guarantees are acceptable.
Do not treat a historical per-workload measurement as a hardware limit or keep
retesting old implementations. Validate the current design with latest stable,
remove clear waste, and justify resource budgets using the total target envelope.
During delegation completion the provisional runtime stack budget is 5 KiB
(5,120 bytes). Defer detailed stack tuning and final Pico qualification until
delegation works; keep the required semantics and concurrent execution intact.
This does not change the separate 4 GB validation ceiling or its 3 GB stop.
Tokio, owned model requests, `Vec` and host-side queues belong only to this
application, tests or integrations. They must not move into core. Qualify core
repairs with no-default-feature host and the selected Pico target builds,
including the embedded projection example. The current checked target is still
thumbv6m; Pico 2 is permitted, not yet selected or qualified. Keep the
caller-supplied runtime storage path.

The user's proposed core prerequisite is affine Endpoint delegation. Preserve
ordinary send/recv at the localside while making transfer consume the resource
Endpoint and preserve its current continuation. Initial issuance must establish
one authority for the same resource; merely making Endpoint non-Clone is not
enough. Cover send/drop/failure, distributed peer binding and replay, and the
dependencies of delegated operations, ordinary communication and related sessions
in projection and progress verification. Do not serialize a Rust pointer, revive
uncertain ownership on timeout, or replace this guarantee with an application
owner table. The selected Hibana 0.10 development branch implements owned
send/recv and concurrent endpoint fragments. Use the exact core and WASI runtime
revisions recorded in Cargo.lock. Core examples and proofs do not establish that
ChoreoFix has been ported or that its experiments have passed.

For a shared operation requiring multiple resources, the effect service must
receive the actual request through each required resource Endpoint before it
performs the effect. Use the real result as the reply before returning the
Endpoints. Put those messages and their dependencies in the same task global,
and write their send/recv operations directly in the localsides. Acquire and
return independent Endpoints with par and Tokio. Do not replace the old resource
broker with another permission token plus a separate effect path.

Delegation projection must place an owned endpoint's operations and its
holder's other communications in the same local ordering. Moving a unique
value without those dependencies is insufficient. At route joins, check the
endpoints used by the continuation or a later roll visit; do not require a
redundant return for an endpoint that will never be used again. Preserve safe
path-dependent ownership in the finished design rather than declaring every
divergent join unsafe. Ownership-only reference-model acceptance is not full
Hibana projection acceptance or evidence of distributed deadlock freedom.

Progress verification follows the candidate's actual source dependencies.
Do not require every earlier actor-local index to form a closed history.
Independent parallel work and old continuations after a rolled body can be
unrelated to that candidate. Prove closure as a property of existing progress,
without introducing another runtime state field or an application tracker.

Replace the core's one-Endpoint/one-cursor ownership structure for delegation.
Each actor owns one projected semantic progress, canonical route/roll history
and frontier. Each logical Endpoint is its unique operation view. Keep per-view
pending I/O distinct from that common progress and retain logical ports for the
run across transfers. A view move/drop must not discard accepted frames or
destroy the actor owner. Acquire the common mutation lease before forming a
mutable kernel reference, and release it within each poll so independent views
can remain Pending concurrently. Prepare and publish view issuance/retirement,
progress and required control context atomically. Do not layer a delegation
manager or synchronized independent cursors over the old ownership structure.
Use the same kernel for ordinary communication and delegation, without a public
mode or a compatibility wrapper. Private owned executions do not establish
general distributed continuation or public projection acceptance. Report the
checked source and the exact unfinished boundary.

Keep run identity, faults, registry access and shared workspace in the actor's
session context, outside mutable protocol progress and individual view ports.
Retiring a view's I/O must not remove access to shared progress inspection.
Bind the mutation lease to its actual actor, including actors in one rendezvous.

Color parallel lanes using the physical actor candidates derived from the source
ownership DAG. Distinct logical resource views held by one actor must retain
independent frontier slots. Preserve logical roles and wire peers. Reuse the
existing lane matcher instead of adding a runtime holder table or synchronization
flags. Candidate membership supplies storage and navigation facts; resolver authority still
requires the issued logical view to match the global controller. Resident row
and lane-head locators are caches. Relocating them must not commit events or
select branches.

Preserve valid composition rather than accidental acceptance. In core work,
retain independent `par` branches containing `roll`, ordinary sequential reuse,
receive-headed request/reply repetition, exclusive route alternatives, and
valid outer repetition. Validate ambiguity after role projection, including
global prefixes erased for a particular role. Do not infer that an elastic roll
has retired merely because a later operation committed. Existing passing runs
do not establish that its public sends are unambiguous. Do not add public aliases,
operation modes, or application progress state for compatibility.

Keep a send-selected controller's committed choice while locating its receive
continuation. Completing that send alone does not retire the choice. Completed
observer/resolver visits must still yield to enabled independent resolver work.
Scheduling an enabled entry does not itself authorize either branch.

Use `g::par` for independent work and `g::route` for alternatives. Connect
explicit decision sites to the actual domain result through a resolver. Use
`.roll()` for repeated work and revision cycles. Keep the repetition boundary
and its exit visible in the global protocol and localside functions. Do not
replace them with a stage enum, approval flags, or an application program counter.
Do not add choices or repeated work that the task does not require.

`roll` makes a fragment repeatable; it does not give a route implicit
"right retries, left exits" semantics. Keep any success-dependent continuation
inside the successful branch. Putting that continuation after a rolled route
can admit it after a completed rejection visit too. Check the global rather
than relying on the intended localside to stop only after success.

Resolver registration is keyed by the compiled global program and resolver ID,
not by the role argument alone. Independent owners with independent validation
results need distinct resolver IDs. Reuse an ID only for the same policy owner
and callback data; a RoleProgram argument does not create a role-local namespace.

Preserve parallel acquisition of independent resources in the global and in
actual execution. Issue both asynchronous acquisition requests before waiting
for either grant. Require both grants before the effect and returns after it.
Do not serialize a parallel fragment in the localside. Preserve the task's
outcome, participants, resource constraints and necessary data dependencies;
improve the coordination procedure using Hibana rather than copying legacy
control flow. Keep baselines unchanged and record procedural differences before
scored runs, including departure from sequential-acquisition checklist text.
Run independent model participants and already-admitted domain operations as
concurrent async tasks too. A shared stdout reader must keep reading while an
independent simulator operation awaits I/O. Measure observed overlap from raw
timestamps; declared `par` nodes alone are not evidence of concurrent execution.
Do not retain a serial localside merely to match an older experimental recipe.
Start independent work before awaiting its results and join only where the
global requires both. Keep overlapping participant calls and admitted effects
visible in the persisted evidence. Treat resource conflicts and data dependencies
separately from historical procedural order when documenting fidelity.
Async test fixtures establish scheduling behavior, not measured model overlap.
Keep their controlled delays and barriers labeled in the evidence. Scored runs
must record overlapping actual model calls and independent admitted effects
without inserting test barriers to manufacture overlap.
One physical owner Endpoint alone does not prove exclusive grants: independent
`par` branches can permit several loans. Test the global by driving other legal
Endpoint interleavings as well as the intended localside. Do not qualify a port
when exclusion depends only on waiting for a return before calling `offer` again.
Keep failed global-authority audits explicit and out of scored experiments.

## Shell, WASI, and ChoreoFS boundary

The model chooses its own shell commands, scripts, calculations, and private
work. Do not prescribe a command recipe or use a small command allow-list as
the coordination mechanism. Shell freedom is a requirement, not a claim that
the current bounded guest has already met.

Use the real `hibana-wasip1-runtime` and ChoreoFS. Do not substitute a native
host shell, host filesystem, another Wasm engine, or a second virtual filesystem.
The shell is the small, dependency-free `hibana-wash` WASI guest, pinned as the
`vendor/hibana-wash` submodule from its own repository. Keep shell
syntax and file I/O here, never task-specific commands, permission tables, or
coordination state. Do not grow the runtime merely to embed a general Bash
implementation. Shell variables and syntax are computation, not task authority.
ChoreoFS is the object interface to projected communication. It is not a second
authorization engine. A pathname, fd fact, rights mask, directory listing, or
successful open does not authorize a task effect.

Shell freedom does not grant access to other participants' private objects or
the host filesystem. Private computation may read and write its own objects
without task ordering constraints. If an original task constrains a shared
read, put that read's request, dependency, and reply in the global too. Do not
silently treat all reads as unrestricted or claim that 16E proves such a gate.

Put shared-service WASI requests and task dependencies in the global protocol.
Keep the corresponding `Endpoint::send`, `recv`, `offer`, and branch operations
visible in the localsides. The receiver must consume the projected request and
any required materialization permission before applying the shared effect.
Use ChoreoFS operation results to materialize the admitted object operation.
Static object binding identifies the operation; parsing payloads does not infer
a route or repair an out-of-order protocol.

Pass the `ReceivedPayload` returned by an actual Endpoint receive into ChoreoFS.
This applies to private argument-buffer open/write/close as well as shared
effects. Raw WASI request data and successful codec validation are not received
requests. Complete the ChoreoFS operation before advancing its borrowed Endpoint.
Use the distinct base-write and object-write request types and respect the fd's
binding. Keep private I/O freely selectable through its global route/roll; do not
restore a direct ChoreoFS call to avoid writing the corresponding communication.

The target in an operation's arguments must agree with the projected service
and domain object. Validate that correspondence before materialization. An ABI
conversion may reject malformed or wrong-object input before constructing a typed
request, without sending a message or faulting an Endpoint. The receiver must
independently validate the exact wire payload before committing that request.
Successful payload conversion grants no task step: projection still determines
which message may be sent and what effects may follow its receive. Use explicit
resolver/rejection routes when rejection itself is a shared protocol event, and
keep any dependent effect inside the accepted branch. Do not add a validation
loop or permission flag when a finite typed request expresses the dependency.
Preserve and report differences from the unchanged baseline's argument matching.

The WASI-driving role itself must send each shared-operation request through
its projected Endpoint. The global must connect that request to the service
owner's receive, the admitted operation, and the reply to the WASI role:

WASI Endpoint -> global request -> owner Endpoint -> ChoreoFS -> global reply.

Putting WASI and task protocols beside each other with `par` does not establish
this connection. Do not forward shared requests between those graphs with
`mpsc`, oneshot, callbacks, a shared queue, or another unprojected channel and
then validate them at a later Endpoint. Channels may implement the Transport
that carries projected messages; they may not supply missing choreography edges.
Keep these Endpoint sends and receives visible in the actual WASI localside.
Do not introduce a relay role, a progress flag, an operation-permission table,
or a second state machine to repair missing global connections. Payload, fd and
path conversion belongs at the ABI boundary; coordination belongs in the global.
One Wasm guest may directly own several projected roles from that same global.
Use independent service endpoints when their lifetimes progress independently.
An unqualified `offer()` observes any enabled branch of its endpoint, not the
response for a particular fd or previous request. Do not assume that a request
implicitly scopes the next offer, or drain unrelated previews into progress flags.
Use `offer()` for a projected choice and `recv::<M>()` for a known sequential
receive. A return after a rolled choice is not itself a branch. If a service
must distinguish further requests from guest termination, put that distinction
in the global, receive the actual exit result, then receive the returned Endpoint.
Component tests, simulator completion and green CI cannot establish this
architecture without checking the actual request/reply path.

Updating the Hibana dependency does not complete either the WASI or ChoreoFix
rewrite. Their Hibana localsides must pass the actual operation Endpoint and
use that received value for the request, result and return in the same global.
Do not retain a unit grant that authorizes an operation through another path.

A generic `/tools/call` JSON service followed by a separate task validator is
not the target architecture. Passing that integration does not qualify a direct
ChoreoFS boundary. Do not hide this gap by renaming a helper or a file.

Ordinary shell failures, malformed argument data, and rejected requests return
an error to the model so that it can reconsider and continue. Represent shared
request/rejection/reconsideration cycles with explicit routes, resolvers and
repetition. Distinguish a recoverable domain result from a transport fault or
lost ownership. A retry must remain possible under the domain's actual rules;
never resume a faulted Endpoint, rebuild it to erase progress, or maintain a
second expected-step/allowed-operation table. Record added recovery paths as
ChoreoFix procedural changes rather than silently changing a comparison arm.

File contents, offsets, buffers, transport queues, and RPC correlation are data.
They must not become permission or task progress. Use affine resources and
receiver lifetimes where appropriate; do not disguise an approved/done flag as
an enum, `Option`, callback, channel status, or another filesystem.

Qualify the actual path from model shell command to WASI import, projected
request, ChoreoFS materialization, and unchanged simulator effect. Test invalid
arguments followed by valid work, early and duplicate effects, chunked I/O,
error recovery, independent work, and the original resource constraints.
Compilation, synthetic import tests, and previous generic-RPC episodes are not
substitutes for that end-to-end evidence.

Read `plan.md` before continuing this work. Keep its verified state, next action,
and remaining gates current. Do not replace incomplete evidence with a success
claim or count unaccepted ports as accepted full-task implementations.

The 2026-10-30 research plan makes resource ownership, remaining protocol and
actual effects the central claim. Establish the T3/T4 minimal composition and
its real effect-boundary counterexamples before spending the scored-run budget.
Reuse the existing core semantics for T1/T2, including evaluated par and roll;
record the exact correspondence and assumptions for T5/T6. Ghost ownership and
occurrence histories belong in the model, never in a new application authority
table. Do not expand completion into a proof of arbitrary Rust or all failures.

After a core change, run the complete local gate set, including protocol fuzz,
on the final source. A consumer-discovered core defect needs an actual Endpoint
regression and coverage of its structure, repeated message contracts and schedule
in generated tests. Check sensitivity by removing only that fix. Identify whether
the formal model omitted the execution or the implementation correspondence was
unchecked, and strengthen that boundary. Individual proofs or fixtures are not a
successful complete gate.

Audit every effect entry: WASI imports, host APIs, argument codecs and physical
bindings. `Domain::execute` is ordinary host Rust. Its agreement with the
received request, exact arguments and required resource Endpoints is an explicit
adapter trust boundary. Record-mutation rejection only tests the auditor;
test runtime rejection at the effect boundary separately. A local I/O error or
deadline does not revoke remote ownership or authorize reissuance.

Keep the existing 2,592-run comparison and its 864-run first stage. Additional
ZipperGen, Concordat, handoff and causal comparisons are planned in `plan.md`,
not qualified by the existing matrix. Audit native expressiveness before task
selection. Preserve comparator engines and distinguish native system comparison
from matched causal comparison. Count unsupported tasks separately from failed
runs. Freeze precision, reserve allocation and rerun rules before scored results;
do not repeat until the preferred result appears. Preserve negative evidence.

The unchanged upstream simulator is an external environment and scorer. Its
resource and artifact state must not authorize Hibana endpoint transitions.
Upstream state tracking belongs only to the explicitly named baseline condition.
Static compilation data and append-only observations must not become a second
runtime progress authority.
Build qualification records may select and pin the measured executable before
a trial. Verify them against the execution's Git commit and preserve the source,
checks and exact artifact hashes. Never consult these records from a guest,
task localside or resolver to grant an operation or advance the protocol.

Rejected operations do not commit progress or cause domain side effects. A
process crash is a failed run, not a successful protocol completion. Protocol
completion does not prove domain task correctness. Score both independently.

All participant model calls use GPT-6 Luna with xhigh reasoning. Record the
requested and returned model, reasoning effort, usage, raw tool events, source
versions, and failures. Never silently substitute a model or a protocol.

Use Rust for application code, tests, experiment tooling, runners, audits, and
integration code. Python is permitted only for external QUIC interoperability
tests. Do not add Python for source packaging, CI checks, fixtures, model access,
WASI integration, profiling or experiment analysis. Do not embed Python snippets
in Rust or shell commands to evade this rule.
Existing self-authored Python is migration debt, not an exception. Replace it
with Rust while retaining its behavioral coverage. Keep the original vendored
TraceFix comparison implementation byte-for-byte unchanged. Its original
language belongs to that external comparison and does not authorize Python in
the Hibana implementation or harness.

Do not report a run as measured until its persisted events and simulator result
have been independently checked. Keep harness errors, protocol rejection, domain
failure, timeout, deadlock, and incomplete work distinct.

Use Hayato Ohhashi <o8@vmm.dev> for commits. Do not add AI coauthor trailers.

Do not use development subagents. The separately recorded Luna xhigh benchmark
participants are the authorized experimental subjects, not development workers.
