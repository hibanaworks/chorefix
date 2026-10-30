# Shared effects and the adapter contract

The model chooses shell commands in its own hibana-wash Wasm instance. WASI
requests reach ChoreoFS through projected Endpoint receives. A task object
contains request data; opening or writing it alone does not authorize its
business operation. The task global determines when the corresponding request
and resource Endpoint can be used.

`Domain::execute` connects an admitted operation to the unchanged task simulator.
It is ordinary host Rust and is not a separate tool exposed to the model. The
receiver code calling it is part of the adapter trust boundary. Endpoint checks
do not prevent arbitrary host Rust from calling an external service directly.

## Receiver contract

1. Receive each actual request through the projected Endpoint before the effect.
   For a shared operation requiring several resources, receive every required
   resource's request. Their arguments must agree.
2. If an earlier WASI close or business message admitted the arguments, compare
   the resource request with those actual arguments before calling the domain.
   A matching JSON schema alone does not establish that equality.
3. Bind the caller and target to the projected task participant and object.
   Preserve the original argument semantics, including intentionally ignored
   fields, and record stricter interface checks in the experiment contract.
4. Invoke the effect once for that received operation and return its actual
   result through the same global. Do not synthesize a successful result from
   protocol completion or reuse a previous result as a new operation's result.
5. Keep the requests, results and Endpoint returns in the global. Request data
   retained for comparison is data, not another permission token or phase flag.

Physical resource identity and aliasing are the initial binding integration's
responsibility. The transport must satisfy the pinned core's delivery and
receiver-reachability contract. An I/O error or deadline does not revoke a
remote Endpoint or permit a new initial binding. An external effect may have
succeeded before its reply is lost; this contract does not supply exactly-once
external APIs or crash recovery.

## Migrated task receivers

This inventory covers the twelve tasks currently using the six Wasm task
families below. It does not qualify the other thirty-six task ports or legacy
native entry points. Source and executable identity are recorded separately for
each qualification.

| Tasks | Domain operations | Actual receiver and checks |
| --- | --- | --- |
| 1E/1M/1H | `design_feature`, `implement_code`, `run_local_tests` | [Developer](../src/tasks/codebase/choreofs/localside/developer.rs) and [reconsidering developer](../src/tasks/codebase/choreofs/localside/developer_reconsider.rs): typed close data, assigned feature, received ChoreoFS request. |
| 1E/1M/1H | `commit_changes` | The same receivers await both module requests, compare their bytes and materialize both ChoreoFS closes before the effect. The [codec](../src/tasks/codebase/choreofs/wire.rs) checks the assigned physical module pair. |
| 2E/2M/2H | `read_literature`, `draft_content`, `review_own_work` | [Researcher](../src/tasks/writing/choreofs/localside/researcher.rs): received file requests and the actual validation result for that close. |
| 2E/2M/2H | `write_to_document` | The researcher awaits both section requests and compares them with each other and the arguments admitted from the file close. |
| 6E | `implement_feature`, `merge_code` | [Repository](../src/tasks/review/choreofs/localside/repo.rs): actual business admission, sole repository Endpoint loan, resource request equality and assigned feature/branch. |
| 6E | `code_review` | [Reviewer A](../src/tasks/review/choreofs/localside/reviewer_a.rs) and [reviewer B](../src/tasks/review/choreofs/localside/reviewer_b.rs): actual typed review request and assigned target; actual result feeds the resolver. |
| 8E | `migrate_schema`, `implement_backend`, `run_integration_tests` | [API receivers](../src/tasks/api/choreofs/localside): received file requests, validation and actual resource requests. Resource codecs accept only the original task's fixed migration/suite arguments. |
| 9E/9M/9H | `think`, `eat` | [Philosopher](../src/tasks/philosophers/choreofs/localside/philosopher.rs): actual request; eating awaits both fork requests with equal payloads. The projected caller determines the agent, matching the original simulator. |
| 16E | `compile_and_test`, `run_integration_tests`, `deploy_application` | [CI/CD receivers](../src/tasks/cicd/choreofs/localside): received file close followed by the actual infrastructure request. Each receiver compares those two argument values before its effect. |

## Evidence and remaining obligations

The [experiment contract](experiment-contract.json) and [work plan](../plan.md)
distinguish qualification on a local core patch from qualification on the normal
Git dependencies. Actual-Wasm tests exercise model-selected shell interfaces;
native Endpoint probes can also bypass the intended guest localside and test
the production receiver directly. Record mutations measure auditor sensitivity
and are separate from runtime rejection before an effect.

The [repository receiver tests](../src/tasks/review/choreofs/tests.rs) use
separate runtimes for admission and the actual resource request. They check
substituted arguments, a wrong target, and an unchanged request whose real
result returns to the caller. The [CI/CD receiver test](../src/tasks/cicd/choreofs/tests.rs)
substitutes a compile argument after admission through an actual received
Endpoint. That test exposed a missing receiver check: the old receiver invoked
the domain with the substituted argument. The corrected receiver rejects it
before any domain call. This checks the production receiver directly, rather
than relying on the intended guest to forward the admitted arguments.

These host-side probes deliberately bypass the guest adapter. A substituted
host request ends that run; the receiver does not roll back a completed
Endpoint receive. Ordinary malformed guest arguments have their existing
rejection route before resource use, so the model can revise them and continue.

T3/T4 must connect resource custody, remaining source protocol and this adapter
contract. Their composition, target/version binding in the new artifact cases,
all effect entries in the remaining task ports, and final dependency
qualification remain required. This inventory is not a formal proof that every
host call obeys the contract.
