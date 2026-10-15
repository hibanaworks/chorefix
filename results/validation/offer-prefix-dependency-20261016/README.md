# Offer prefix and concurrent consumer verification

Core: `ef65f0956104441a7bcf2dc4eabbc62930e4c30d`, version 0.9.9.
ChoreoFix base: `e4812e349494bfb1496cfa1241e004b7ca01143e` plus the recorded patch.
This is development verification, without model calls or scored episodes.

Passive offer could preview a later receive before an earlier visible message.
The failure occurred on remote main f1c4dd4 and c03; the previous failure record
is retained. Applying candidate-path dependency guards only when no committed
parent arm existed still failed after a completed parent revision. That rejected
intermediate candidate has its own failure directory.

The final kernel checks dependencies under the projected candidate path while
checking reentry against committed history. Only exact, unambiguous, otherwise
valid future input waits for a predecessor. It is returned to its original port
without protocol progress. A 32-byte poll-local lane mask bounds repeated
inspection of a deferred lane; it is not stored in Endpoint or a future. Newly
collected primary-lane input is classified by the same rules. Public API, wire
format, persistent ownership state and allocation behavior are unchanged.

## Core checks

- Workspace: 888 passed, zero failed, 12 explicitly ignored. Focused suites: 11 passed.
- All-target strict Clippy, stable host no_std and thumbv6m no_std passed.
- Full Lean audit: 709 static / 506 generated claims, plus the related Lean/Z3 checks.
- New boundary: 13 Lean theorems, 12 UNSAT obligations, one historical SAT witness.
  Descriptor decoding, exact matching and port binding are assumptions of this
  boundary model. It is not a proof of every Rust line or an arbitrary Transport.
- Strict-provenance Miri: three nested and two ordinary-input tests passed.
  The delayed-prefix test covers queued and already-pending offers, both nested
  branches and a completed containing revision, nonempty payloads, repeated
  pending polls, real wake/delivery and an abandoned preview.
- Kani: ten production harnesses for exact frame keys, unique matches, affine
  receive receipts and lane-bit mutation passed.
- Measurement/layout gates passed: aggregate flash 94,832 bytes, SRAM 5,306 bytes,
  largest measured stack 2,639 bytes. Checked Endpoint/branch/future sizes remain
  16 bytes on the host. These are measurements of the checked programs.

GitHub ef65 final-form stopped after 180 seconds in the nested Miri group, with
two of three tests complete. The same full local group passed in 320.86 seconds.
Follow-up e500c7e39931ff6736b63d2227fd90fba0a497ea only assigns that group the
existing 480-second deep-route allowance. Its CI is pending. No tests were
disabled and no production or test semantics changed.

## Actual consumers

Two workaround `DocumentReviewReady` acknowledgements were removed from the 3E
global and localsides. The real `BeginDocumentReview` notifications remain.
The final path build passed six native tests and two standalone prefix tests.
The published Git pin then passed the independent probe and all six integrated
Root 3E tests, plus all-target strict Clippy. No participant flow wrapper or
second progress authority was added.

`async-integrated.json` preserves every carrier frame and four measured fixture
intervals. Independent requests overlap by about 36.6 ms; A's reference update
and B's document write overlap by about 28.0 ms. The stimulus is a 25 ms async
test delay without a barrier. This proves scheduling of the fixture, not
measured model overlap or a performance benefit. The earlier path-built record
is retained separately.

The actual `review-wasi` executable was rebuilt from the Git pin. Only the four
previously failing 6E revision cases were retried: both review orders at depth
one and two. All four passed in 14.69 seconds. The guest is the real hibana-wash;
WASI uses runtime 4ebdad1443cf49488eae924afe528f67f5a5953b and ChoreoFS.
`real-wasi-cases.tar.gz` retains each case's source/binary hashes, actual carrier
events and stderr, including invalid/early/duplicate requests. The unchanged
simulator and existing adapter perform these deterministic boundary tests.

The binary hashes are in `consumer-binaries.sha256`. These checks do not replace
full-task fidelity acceptance or the actual Luna xhigh comparison. The earlier
qualified model executable record is not retagged as this build. Full accepted
tasks and scored episodes remain zero.

QUIC consumer evidence and remaining Linux/interop checks are recorded in the
adjacent `quic-current.json` and raw logs. Earlier 43/44 and 44/44 sources remain
separate results.
