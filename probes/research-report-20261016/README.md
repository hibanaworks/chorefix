This development crate compiles the actual research_report source and carrier
without rebuilding every unrelated guest protocol. It selects Hibana 0.9.9 at
Git revision 64d4f4966403c967f2b80393acec2c36556bfc5c.

Run the six native component tests with:

```sh
cargo +1.95.0 test --locked --manifest-path probes/research-report-20261016/Cargo.toml --test report -- --nocapture
```

The async timing fixture uses a controlled 25 ms delay. It verifies that
independent futures are actually polled concurrently; it is not a model trial.
The real WASI, ChoreoFS, unchanged simulator and Luna paths remain unqualified.

`tests/nested.rs` retains a separate unresolved core reproduction. Its first
test fails because offer previews a later receive before a required earlier
receive. Its second test checks that consuming that later step is rejected.
Do not count that known failure as a successful full crate or task qualification.
`failures/extra-exit-choice` retains the earlier failing protocol and raw log.
