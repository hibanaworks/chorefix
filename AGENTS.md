# Experiment rules

The experiment retains all 48 TraceFix tasks, their participants, resource
constraints, dependency checks, branches, and failure conditions.

Hibana global protocols are the only source of coordination rules in the Hibana
condition. Endpoints own protocol progress. Do not add a second state machine,
program counter, approved/done flags, lock-owner table, or pending-step tracker
to authorize coordination. Do not serialize independent work to make a task pass.

Keep task protocols in `src/global/`, participant functions in `src/local/`, and
transport, model access, and domain integration in `src/imp/`. Use the actual
published Hibana API. Do not modify Hibana core to add agent-specific APIs.

The unchanged upstream simulator is an external environment and scorer. Its
resource and artifact state must not authorize Hibana endpoint transitions.
Upstream state tracking belongs only to the explicitly named baseline condition.
Static compilation data and append-only observations must not become a second
runtime progress authority.

Rejected operations do not commit progress or cause domain side effects. A
process crash is a failed run, not a successful protocol completion. Protocol
completion does not prove domain task correctness. Score both independently.

All participant model calls use GPT-6 Luna with xhigh reasoning. Record the
requested and returned model, reasoning effort, usage, raw tool events, source
versions, and failures. Never silently substitute a model or a protocol.

Do not report a run as measured until its persisted events and simulator result
have been independently checked. Keep harness errors, protocol rejection, domain
failure, timeout, deadlock, and incomplete work distinct.

Use Hayato Ohhashi <o8@vmm.dev> for commits. Do not add AI coauthor trailers.
