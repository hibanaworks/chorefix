# Writing direct WASI qualification

The 2E/2M/2H cases run the actual hibana-wash guest and WASI runtime. Requests,
validation, physical section loans, effects and replies share one task global.
They execute 12/20/28 original simulator calls with 3/5/7 participants. Independent
write operations overlap in groups of 1/2/3. Both Acquire requests precede even
the first grant being sent. Both Return requests precede acknowledgement sends.

This is a deterministic mechanism qualification with zero model calls. It is
not a scored comparison, a wall-time benchmark, or acceptance of the full corpus.
The environment uses explicit adversarial delays to test parallel progress and
return-before-review. The separate terminal-review-failure case reaches protocol
completion while the unchanged domain scorer reports incomplete work. Each role
still performs four operations, with no extra review retry.

The Rust audit compares exact task inputs, frame conservation, closes before
effects, resource intervals, delivered results and the original simulator journal.
Seven mutated logs per normal case are rejected. One preserves every frame but
serializes the second acquisition after the first grant, and must fail the audit.

To regenerate, install the existing locked environment and build the recorded
guest, then run from the repository root:

```sh
cargo +1.95.0 build --locked --bins
target/debug/writing-boundary 2E output/check-writing-2E
target/debug/writing-boundary --audit output/check-writing-2E/events.jsonl
target/debug/writing-boundary --check-mutants output/check-writing-2E/events.jsonl output/check-writing-mutants
target/debug/writing-boundary 2E:terminal-review-failure output/check-writing-failed-review
```

Use 2M/2H for the other sizes. Compressed logs are lossless gzip. Decompress one
before passing it to --audit. status.json and artifacts.sha256 identify the code
snapshot and actual binaries. Development failures remain separate. The original
sequential-acquisition checklist procedure is intentionally changed, as recorded
in benchmark/experiment-contract.json. No baseline input or implementation changes.
