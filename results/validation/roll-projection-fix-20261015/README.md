# Role-local roll projection repair

Core commit: `f86451b3fa65508e351fd48db32af5129a0539b6`, based on remote main
`f1c4dd4bab211b005687cfec15bb53a1f77500ed`.

Review: https://github.com/hibanaworks/hibana/pull/8

The current roll semantics permits reentry after later communication commits.
Projection must therefore reject an outbound contract that could select either
reentry or compatible later work. The old check used the first global event
and an intermediate scope boundary, missing role-erased prefixes and later
competition after receives or parallel joins.

The repair checks role-local outbound heads against compatible later sends.
Exclusive route alternatives remain separate. Public API, runtime selection,
role descriptors, and storage are unchanged. No scope-lifetime heuristic is
adopted. The nine diagnostic regressions retain ordinary sequential reuse,
receive-headed repeated replies, and independent parallel rolls. Two new UI
cases check rejection by real const projection.

Validation uses the actual source in this commit. The earlier proposal to
retire an old roll after later communication is not this repair. The original
ChoreoFix 16E and unfinished 6E globals also need to meet the stronger check;
sampled successful runs are not evidence that they meet that boundary.
The published crate remains 0.9.9. No registry release is made here.

The full workspace was run with `RUSTFLAGS='--cfg hibana_repo_tests'` and
`--test-threads=1`: 878 passed, 0 failed, 12 explicit ignored. CI-tool fixture
failures in parallel execution were reproduced on the unchanged baseline;
its sequential run and this fixed sequential workspace both pass. Those
fixture sources are not modified in this change.

The Lean gate checks its complete static/generated inventory. The focused
claims additionally exhibit both admitted continuations after a parallel join.
Z3 checks exact packed selector identity (UNSAT, UNSAT, SAT). Miri executes four
actual validator/ancestry tests. Kani executes four production-validator
harnesses with full symbolic label/schema domains, a fixed finite source, and
unwinding assertions enabled. These bounded checks are not a proof of all Rust.

QUIC's dependency tree records that direct Hibana and transitive TLS use this
fixed source. Default tests and doctests passed (593 total); hq also passed (598 total).
The core GitHub push and PR workflows both passed all Kani and final-form
checks. External QUIC interop artifacts attest 42 passes and no candidate
failures in the first run; two ECN cells were not executed because the
runner-tools empty-pcap smoke failed before the candidate build. The ECN-only
retry in run 38723148382 passed both remaining cells on the same core/QUIC
commits, exact source-pair hashes and common pins. All 44 case/direction cells
therefore passed across the two runs. The initial workflow concluded failure,
and the diagnostic retry skipped full qualification. This is not one green
44-cell qualification workflow. See status.json, interop-attempt1-audit.json
and interop-combined-audit.json.

Raw logs and the exact core patch are stored as deterministic gzip files.
`status.json` records hashes of both the compressed files and their raw contents.
