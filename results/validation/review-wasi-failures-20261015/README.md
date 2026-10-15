# Unresolved direct 6E WASI boundary

These are failed deterministic boundary tests. Neither candidate is an accepted
full-task port. They are not scored model episodes and have no model calls.

The first readiness candidate completes depth 0, but depths 1/2 fail after the
second implementation: the developer WASI role's receive of Result reports
PhaseInvariant. The second candidate adds a separate outer visit notice and
explicit projected review outcome messages. It stalls at depth 0 and fails at
REPO offer after a later review at depths 1/2. Do not adopt its extra messages
as a proven repair.

Both use the exact fixed-core commit f86451b3fa65508e351fd48db32af5129a0539b6
through an isolated Cargo patch. The public registry is unchanged. Source
snapshots, actual carrier events, simulator effects, stderr, and native test
logs are retained. The existing source at ChoreoFix commit 25a2290 supplies
shared integration modules; the integration additions and Python test are here.
The task source and simulator are unchanged pinned TraceFix 6E.

The two-, three-, four-, and seven-role reductions complete. They rule out some
simple hypotheses, but do not establish the cause of the real failure. Preserve
this distinction before changing the core validator or runtime. No endpoint
wrapper, application phase flag, or second coordination state should be added.
