These are three unscored qualification trials, one per writing configuration at
depth 2. They are not the comparison matrix, success-rate estimates, or accepted
full-task ports. The models are GPT-6 Luna with xhigh reasoning in all 15 cases.

| Task | Simultaneous model turns | Disjoint write overlap | Domain calls | Shell calls | Inclusive tokens | Seconds |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2E | 3 | 1 | 12 | 18 | 327866 | 30.418903208 |
| 2M | 5 | 2 | 20 | 30 | 551218 | 32.885682583 |
| 2H | 7 | 3 | 28 | 43 | 795075 | 37.334704667 |

All three original simulator scores are complete, with zero unsafe effects.
Both resource requests precede the first grant; both returns precede the first
acknowledgement. The model turns overlap on recorded wall-clock intervals.
Disjoint write overlap is observed from actual admitted operations and original
simulator completion times. No artificial barrier is used in these model trials.
Normal injected failures are disabled by the original writing simulator at all
depths. The separate terminal-review-failure qualification is recorded elsewhere.

Execution source is 9ed7738d02d0ebdcb40706316ae5fd5c29425c79. The actual guest
runtime is the qualified 4096a2a binary with core c2001ec. All three source
manifests are identical. Source, exact binaries, raw frames, original simulator
journal, prompts, model events, returned settings, usage and errors were frozen
before model execution. `execution-source.tar.gz` and each task's `raw.tar.gz`
retain the source and raw observations. `versions.json` records binary hashes.
Binary-inclusive archives remain under `output/`, with hashes in
`complete-archives.sha256`. Those archives are required for the complete binary
checks in the model auditor. Their payloads were not added as Git build outputs.

The original frozen verifier compared section argument strings exactly. The
models legitimately omitted comma whitespace or reversed the same section pair,
which the service and original simulator accept. The verifier was repaired to
compare the exact pair, while rejecting missing, duplicate or other sections and
extra arguments. Original execution files and frozen verifiers were preserved.
The new verification source is 20dbbebe20acbadd3324a56f510daa73e152064e and is also
saved in `verification-source.tar.gz`. Each audit records both verifier hashes.

Build the verification tools from that source and run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo +1.95.0 build --locked --bin writing-boundary
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo +1.95.0 build --locked --manifest-path tools/runner/Cargo.toml
tools/runner/target/debug/choreofix-experiments --audit output/direct-wasi-preflight/writing-luna-20261016-003/2E target/debug/writing-boundary
```

The same command accepts the 2M and 2H directories. For mutation checks, supply
`--check-mutants RUN NEW_OUTPUT target/debug/writing-boundary`. Each task's
`model-mutants.json` records rejection of six altered copies after recomputing
their outer hashes. Wrong model, invalid inclusive totals, forged commands,
missing tool replies, false task completion and changed upstream source are
rejected. Original copies are unchanged. The extra 2M formatting regression
retains its actual guest events and seven rejected protocol mutations, including
serialized acquisition with conserved frame identities.

`startup-001.log` and `startup-002.log` are failures before any model started.
The first used an inventory scope wider than the original manifest. The second
invoked the old binary after Clippy had stopped the replacement build. Neither
is counted among the three model trials. Original unaccepted audit attempts are
also documented in the ledger. The paired comparison is still 0/2592 scored
episodes, and accepted full-task ports remain zero.
