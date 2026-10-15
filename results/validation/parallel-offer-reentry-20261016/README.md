# Parallel reentry core validation

Core source: `0a296df98be6664f07e426c086ffbb9486be7c71`, version 0.9.9,
pushed to the existing `fix/roll-reentry-projection` branch and PR 8.
This is development verification, not a scored TraceFix run or full-task acceptance.

The ordinary-input failure reproduces on remote main
`f1c4dd4bab211b005687cfec15bb53a1f77500ed` and the previous candidate
`c2001ec06944d7f3b299e16b5b3efb5b0ba0366a`. Removing only the new pre-I/O
eligibility check makes the independent-lane regression fail. The raw logs
are compressed without editing their contents. The patch includes the tests.

Checks on the repaired core:

- Workspace Rust: 885 passed, 12 ignored. The first parallel workspace run had
  two CI fixture failures, including a child SIGKILL. The full serial rerun
  and final full serial run passed. No fixture was disabled or changed.
- All-target workspace Clippy, host no_std, thumbv6m no_std, format and diff checks.
- Full Lean gate, with its audited theorem/claim inventories, and the related
  selector, dependency, resolver and offer-ingress Lean/Z3 checks.
- Kani: nine existing frontier-selection and receive-receipt harnesses passed.
  This does not claim Kani proves the new raw payload pointer's provenance.
- Strict Miri: two new regressions, one committed-send continuation, 28 runtime
  owner tests and six receipt tests passed, 37 total.
- Size snapshot and subsystem layout gates passed. Checked aggregate flash
  94,694 bytes, SRAM 5,418 bytes, maximum measured stack 2,751 bytes.

Miri first exposed a protected shared borrow during requeue of a nonempty
inline Rx payload. `miri-focused-002` retains that failure. Staging its original
pointer and reconstructing a slice only for validation/commit fixes it without
increasing frame layout. `miri-focused-003` is the successful strict run.
Tests without OS I/O use Tokio without its I/O driver, since macOS kqueue is
unsupported by this Miri version. No production path is replaced.

The first broad Miri invocation was stopped during an exhaustive history test.
An attempted continuation test with an OS I/O driver was unsupported on macOS.
Neither attempt is counted as passing. The recorded focused tests exercise the
actual repairs; remaining CI coverage is tracked separately.

QUIC source `9951c60a145b163b1903f010524d7ec48e2d44cd` selects the repaired
core in all standalone consumers. Its 44-cell interoperability run is
`38792896985`. The workflow is dispatched from main, so its displayed workflow
head is `d9a271e6733d94805538584d6581a07d54228d7c`; `source_ref` is the
qualification source above. Its plan job passed, and results are still pending.
A new operator diagnostic certificate was installed before dispatch; its private
key is retained outside the repository. An AES-256-GCM CMS round trip passed.

The earlier core/QUIC successes are not counted as this patch's CI result.

## Unqualified native 1H prototype

Ten native tests pass, including three overlapping independent commits,
different reconsideration counts, and idle-model/test-time resource release.
The current Git-pinned build also passes all-target Clippy. These are development
checks, not proof that the global alone excludes conflicting grants.

`native-ownership-probe-qualified-core.rs` directly drives legal Endpoint
operations in another order. Both A and G receive their first grant for AUTH
before either returns it, and both commit steps are accepted. The protocol's
independent branches permit this. The intended physical-owner localside waits
for a return, but that ordering is insufficient as the only enforcement source.
The raw frame transcript and result are retained separately from the passing
implementation tests. The one-off probe links the built Git-pinned library
and uses the repository's unchanged in-memory transport. It is not an agent run.

This prototype is rejected for full-task/scored use until its global expresses
exclusive grants while preserving independent concurrency. It has not been
ported to the actual WASI model path. No scored run or full-task acceptance is
added by this evidence.

The first patched-core QUIC CI matrix completed with 37 successful cells,
one failed `quiche/client/handshakeloss`, and six unexecuted cells. Both aborted
NEQO phases record `TSharkCrashException`, analyzer exit 255. The successful
diagnostic-key decrypt is not itself a verdict about candidate correctness.
The same source is being retested only for the failed/unexecuted cells in runs
38795745657, 38795750568, and 38795754678. These subset runs are distinct from
a single-run 44-cell qualification.
