The previous native 1E/1M resource broker could choose a different borrower for
each resource session. Concurrent two-resource requests can then form a cycle.
This is application composition across independent sessions, not a confirmed
Hibana core defect. The measured direct-WASI 1E/1M path already uses one global
with physical owners and does not call these brokers.

CI 38774076044 at 2e0c4e6300012b8768c95fa811429e48c0fe5d3d failed the smaller
codebase-level timeout. The native source is unchanged at b645adc5bffcc1e4d87121554499dd532ab4171c,
whose native Rust source, manifests and tests are in native-source.tar.gz.
A local repeated terminal-failure test also times out before any commit starts.
The diagnostic test changes only the timeout panic to print collected domain
events. Its executable is kept locally under output/api-direct-wasi-20261016/
before-native-fix/codebase-tests; binary.sha256 identifies it. This is not a
claim that native-source.tar.gz contains every unrelated library file present
in that local build. The API-WASI implementation was also linked but not called.

Repeat the original test to exercise the nondeterministic schedule:

```sh
cargo +1.95.0 test --locked --test codebase terminal_test_failure_does_not_start_another_attempt -- --nocapture
```

The captured failure contains Design and Implement for all three developers,
with no Commit or Test. A later pass from the same binary is not a repair.
The native 1E/1M repair uses projected physical resource owners in the same
session. Both independent acquisition requests still precede either grant wait.
The separate native 1H broker foundation remains unqualified and needs its own
single-global integration. It is not covered by this terminal-path repair.
