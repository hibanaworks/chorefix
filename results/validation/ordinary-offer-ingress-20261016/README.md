This archive validates Hibana 0.9.9 at
c03c7af337de6d00c150039450ddb0825b39e3a8. It replaces the cached-suffix guard
introduced in 0a296df while preserving the earlier ordinary-input regressions.
The previous four actual-WASI failures remain in
`results/failures/review-reentry-core64-20261016/`.

Exact ingress matching distinguishes a branch's first visible receive from
ordinary user input. Ordinary input returns to its original port and only its
lane's cached offer is removed. The protocol position, selected arm and payload
are preserved. No public API, state field, wire format or allocation is added.

Local checks passed:

- Focused Endpoint suites: 10 tests. Full workspace: 887 passed, 12 ignored.
- Strict all-target Clippy, stable host and thumbv6m no_std checks.
- Strict-provenance Miri: 2 ordinary-input and 2 nested/rework tests.
- Kani: 2 UniqueMatch and 5 receive-receipt production harnesses.
- Full Lean audit: 709 static and 506 generated theorems. Related dependency,
  selector, resolver, admission and ingress Lean/Z3 checks also passed.
- Added boundary model: 8 Lean theorems, 8 UNSAT obligations and 1 historical
  SAT witness. Its exact-classification and transport assumptions are explicit
  in the core proof README. This is not a proof of every Rust execution.
- Size/layout gates: aggregate measured flash 94,694 bytes, SRAM 5,402 bytes,
  maximum stack 2,735 bytes. These describe the checked programs.

The committed patch and exact generated Lean sources are retained here. The
separate future-receive preview issue remains open and is outside this repair.

The 3E probe selecting this exact Git revision passed its six native tests and
strict Clippy. Its complete frames and timestamp intervals are retained; this
is a controlled async fixture, with no actual model calls. QUIC consumer
2af1e696b3d2192ddf25656b4774c6e239a4360a selecting this revision passed 593
default and 598 hq tests/doctests, four source-pin tests and the clean paired
source audit. Its external 44-cell run is 38807924806, still in progress when
recorded. The public recipient fingerprints match the operator-held certificate;
private key material is not included.

The actual ChoreoFix review-WASI binary was rebuilt from this Git dependency
before retrying its four previously failing revision cases. All four passed in
14.20 seconds. Raw case metadata, binary/guest/simulator hashes and complete
runtime events are retained in `review-wasi-raw.tar.gz`. These are boundary
tests against the unchanged simulator, separate from any model measurement.
This archive adds no scored experiment episodes.
