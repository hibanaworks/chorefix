# Hibana 0.9.9 local action API

Classification: caller API misuse. No core execution bug has been established.
The earlier description as an API inconsistency was also too strong: a self
message has the same role on both sides and is projected to one local action.

Checked against published `hibana = "=0.9.9"` and the clean current remote
source `f1c4dd4bab211b005687cfec15bb53a1f77500ed` on 2026-10-15.
No Hibana core files were changed.

The relevant global fragment is a resolved route whose successful arm begins
with `g::send::<OWNER, OWNER, Msg<LABEL, ()>>()`, followed by a reply to another
role. The rejected arm replies directly to that role.

The existing core test `tests/local_action.rs`,
`run_local_route_recv_empty_payload`, consumes the selected local action through
`endpoint.offer().await?.recv::<Msg<LABEL, ()>>().await?`. The branch receive
validates a canonical empty payload and commits the local descriptor once.
There is no transport receive. A successful send before offering can also
consume the local action. These are alternative entry paths, not two steps.

A separate probe exercised a plain resolved route, a route after an incoming
request, and a route nested in a rolled file-operation choice. For all three:

| Invocation | Observed result |
| --- | --- |
| Direct `endpoint.send` of the self message | Success |
| `offer`, then `branch.recv` of the self message | Success |
| `offer`, then `branch.send` of the self message | `send: PhaseInvariant` |
| `offer`, drop preview, then direct self `send` | `send: PhaseInvariant` |

The unsupported invocations are not successful recovery paths. They must not
be used in task localsides, and their failure does not justify resuming a
faulted Endpoint. The three mechanism tests and the real guest integration now
use the existing `branch.recv` contract for the local materialization action.

Raw diagnostic source and output remain locally at
`output/diagnostics/hibana099-local-action-probe.rs` and
`output/hibana-099-self-offer-probe.log`. The archived source was originally
compiled as `examples/repro_resolver_self_offer.rs`; its relative carrier import
refers to that original location. It is not part of the experiment executable
or a comparative benchmark result.

The design constitution's `RouteBranch::decode` is a possible future naming
direction for consuming an offered payload, including the empty local payload.
It is not the current 0.9.9 API, not a confirmed bug fix, and not implemented
here. Aligning all local operations to `send` is not required by this evidence.
