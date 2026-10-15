# Preserve parallel prefixes while observing future input

Core: `594cac066d8b99b5caf852d061b8ee7121dd8698`, version 0.9.9.
ChoreoFix base: `fa4db6d718b59b2b763fc9a69dfe0ed8bcb17367` plus the recorded
consumer patch. This checkpoint contains development verification without
model calls or scored episodes.

The confirmed defect is in the runtime. Projection already records the
predecessor of a future receive. Runtime admission checked current lane-head
membership before allowing that predecessor to complete, and offer required
uniqueness even among ordered future aliases. Both could turn legitimate
cross-lane arrival order into `PhaseInvariant`.

The repair validates identity, conflicts and visit freshness first. Exact
future input waits on its original port until its projected predecessor has
committed. Both dependency completion and lane-head membership remain required
before execution. Only enabled candidates must be unique; retention selects no
event. It adds no public API, Endpoint/future field, allocation path or wire
change. The existing poll-local lane mask bounds deferred reads without
self-wake spinning or persistent application progress state.

## Core validation

- Rust workspace: 889 passed, zero failed, 12 explicitly ignored. Focused
  suites: 11 passed. Strict all-target Clippy, stable host no_std and thumbv6m
  no_std checks passed.
- Full Lean audit: 709 static and 506 generated claims; related Lean/Z3 checks
  passed. All five generated Lean exports are retained as compressed source.
- Updated boundary: 18 Lean theorems, 17 UNSAT obligations and two historical
  SAT witnesses. Descriptor decoding, exact matching and lane-to-port binding
  remain explicit assumptions. This does not prove every Rust line or every
  Transport implementation.
- Strict-provenance Miri: four nested-resolver, two ordinary-input and one
  delayed-offer tests passed. The four-test group took 344.38 seconds.
- Kani: 19 production harnesses passed, covering unique matching, receive
  receipts, typestate facts and the full lane-bit domain.
- Measurement/layout gates passed: aggregate flash 94,908 bytes, SRAM 5,306
  bytes and maximum measured stack 2,639 bytes. Checked host Endpoint, branch
  and future layouts remain 16 bytes. These are the checked programs' metrics.

The strengthened regression delays an already accepted prefix while two future
ordered receives reuse one wire key. It polls repeatedly, delivers with a real
wakeup, abandons the first preview and checks all nonempty payloads. Exact
unpatched and partial-repair failures are kept in the adjacent failure record.

## Consumer changes

The 3E editor begins its independent initial request before waiting for authors.
Its first review overlaps B's document write; A's revision can overlap B's
reference work. A projected `ReadyForFeedback` message marks the actual change
of B's next incoming peer. Success-dependent B review and combination remain
inside A's accepted branch. `roll` makes a fragment repeatable; it does not
implicitly distinguish retry from exit.

Two earlier service-switch acknowledgement workarounds remain removed. The
global and literal Endpoint operations own coordination. No phase enum,
permission table, task-progress flag or wrapper hiding localside flow was added.
The probe imports the actual root tests rather than maintaining a second copy.

The exact Git-pinned independent probe passes two standalone prefix tests,
seven native task tests and strict Clippy. Its raw carrier events and controlled
async fixture intervals are retained separately. Three independent request
futures overlap. Controlled delays/barriers establish fixture scheduling, not
actual model overlap, success rate or performance superiority.

Integrated root, actual WASI and QUIC qualification results are recorded below
as they complete. Older source-specific results do not qualify this revision.
Full-task acceptance and scored episodes remain zero.

QUIC 9437c9a04587ccdeb9802e4fae3f33c8494db9a1 selects this core in all six
consumer locks. Its default suite passes 593 tests. Clean source/dependency
auditing and the 645-file paired export pass. HQ, driver Clippy and source-pin
tests are still running. The first local build stopped on disk exhaustion. That
log is retained and the same checks restarted after completed temporary build
caches were cleaned. Active consumer targets and frozen model artifacts were
preserved.

The current-source full 44-cell run is 38816922328, with explicit immutable
QUIC source input. Linux runtime run 38816943242 selects that branch commit.
Both are pending. The public diagnostic certificate matches the retained
operator certificate. Its private key remains outside the repository.
