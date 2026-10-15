# Experiment rules

## Mandatory design constraints

**No helper/wrapper hiding of localside flow. No separate state machine. No
task progress flags. No duplicate coordination or completion management.**

Write coordination with Hibana `seq`, `send`, `route`, resolvers, `par`, and
`roll`. The projected Endpoint is the only owner of task progress and permission.
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

The experiment retains all 48 TraceFix tasks, their participants, resource
constraints, dependency checks, branches, and failure conditions.

Hibana global protocols are the only source of coordination rules in the Hibana
condition. Endpoints own protocol progress. Do not add a second state machine,
program counter, approved/done flags, lock-owner table, or pending-step tracker
to authorize coordination. Do not serialize independent work to make a task pass.

Keep each task's protocol in `src/tasks/<task>/global.rs` and its participant
functions in `src/tasks/<task>/localside/`. Keep transport, model access, and
domain integration in `imp/`. The global protocol and each role's localside
must be traceable like hibana-quic. Use the actual
published Hibana API. Do not modify Hibana core to add agent-specific APIs.

If a suspected Hibana bug appears, retain a minimal reproduction and report the
affected version and exact observed behavior. Check the remote current source
before calling it a current bug. Fix confirmed defects in Hibana itself; an
application workaround is not completion. Keep the reproducing regression test
and verify the repair with Lean, Z3, Miri, and Rust tests. Use Kani where bounded
checks of the affected boundary or transition are useful. Check its impact on
the current hibana-quic with the fixed core actually selected. Keep
unrelated/dirty working copies intact. Do
not claim these gates from stale logs or an unpatched consumer build.

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

The target in an operation's arguments must agree with the projected service
and domain object. Validate that correspondence before materialization and feed
the actual validation result to an explicit resolver/rejection route. A success
for another object must not grant this branch's permission. This is payload
validation, not another task-progress authority. Preserve and report differences
between this interface and the unchanged baseline's argument matching.

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
Component tests, simulator completion and green CI cannot establish this
architecture without checking the actual request/reply path.

A generic `/tools/call` JSON service followed by a separate task validator is
not the target architecture. Passing that integration does not qualify a direct
ChoreoFS boundary. Do not hide this gap by renaming a helper or a file.

Ordinary shell failures, malformed argument data, and rejected requests return
an error to the model so that it can reconsider and continue. Represent any
required shared request/rejection/reconsideration cycle with explicit routes,
resolvers, and repetition. This does not add retries to an original task whose
domain result is terminal. Never resume a faulted Endpoint, rebuild it to erase
progress, or maintain a second expected-step/allowed-operation table.

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

Use Rust for new experiment tooling, runners, audits, and integration code.
Keep Python where the unchanged TraceFix baseline or original simulator
requires it. Do not add Python merely for convenience, and do not rewrite the
comparison implementation while moving the Hibana harness to Rust.

Do not report a run as measured until its persisted events and simulator result
have been independently checked. Keep harness errors, protocol rejection, domain
failure, timeout, deadlock, and incomplete work distinct.

Use Hayato Ohhashi <o8@vmm.dev> for commits. Do not add AI coauthor trailers.

Do not use development subagents. The separately recorded Luna xhigh benchmark
participants are the authorized experimental subjects, not development workers.
