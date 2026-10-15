Four unchanged real-WASI 6E cases were retried against core 64d4f496 and the
f569c6d ChoreoFix source. Each timed out after the existing 20-second test bound.
The cases are both first-review choices at depths one and two. The original
case metadata records the exact executable, guest and simulator hashes.
No model calls or scored benchmark episodes ran in these component tests.

The four-role Endpoint reproduction in minimal-endpoints.rs also times out
after rework at round one. The request is sent; offer does not observe it.
Another parallel rolled flow remains idle. The same reproduction passes on
remote main f1c4dd4. Temporarily removing passive_offer_entry_enabled also makes
it pass on the patched core. That diagnostic removal was restored immediately.
It is not a proposed repair because the ordinary-input regressions still matter.

The rejected candidate is the old review suffix on lane zero: scope6, entry8,
logical label28, selected1, no live arm, lane head8 and cursor10. Its first
receives are not the new visit's head. The new head is itself a route, so merely
disabling this lane also prevents offer from receiving the next valid request.
This is a regression introduced by 0a296df's gate, retained by 64d4f496.

stderr's terminal-transport closure is emitted during test cleanup after the
timeout. It is not independent evidence of the initiating fault. No guard
removal, serialized application workaround, successful replacement trial, or
full-task qualification is claimed by these records.
