# Codebase live Luna qualification

The clean execution consumer is `2e0c4e6300012b8768c95fa811429e48c0fe5d3d`.
The selected WASI runtime and both Rust verifiers come from source
`80a46eb86aaa7600acb59bb5d897c9c603ca0c83`. Build qualification, exact binaries,
original inputs and raw tool replies were frozen before the participants started.

All eight participants returned `gpt-6-luna` and `xhigh`. Native model tools were
not used. Both trials use depth 2, seed 0 and the original delay multiplier 1.0.
There are no artificial barriers in these model trials. The original 1E/1M
simulators disable probabilistic failure injection at all depths.

| Task | Simultaneous model turns | Domain calls | Maximum admitted calls in flight | Maximum shared commits | Inclusive tokens | Seconds |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1E | 3 | 12 | 3 | 1 | 375,541 | 32.416 |
| 1M | 5 | 20 | 4 | 1 | 680,037 | 37.749 |

All 32 operations, both protocols and all eight guest exits completed. The
original simulator and the carrier audit recorded no resource violations.
The subjects chose 59 shell calls. The total inclusive token count is 1,055,578.
Participant turns count the outer App Server turns, not internal model requests.

The 1M shared commits did not overlap in this natural trial. Relative to the
first simulator call, A committed at 6.766–7.464 seconds while C was still
implementing until 8.289 seconds. B committed at 12.166–13.030 seconds while D
was still implementing until 14.448 seconds. This result is retained as observed.
The separate deterministic boundary trial demonstrates two disjoint commits
when both developers are ready. It is not substituted for the model result.

`domain_intervals.rs` measures the original simulator RPC entry immediately
before `registry.call` through its return. These intervals include the unchanged
tool delay. They measure admitted calls in flight, not simultaneous CPU execution.
The raw timestamps and all intervals are saved per task. Three Rust tests check
nested overlap, adjacent intervals and invalid timestamps. This offline utility
is never consulted to authorize or advance task execution.

Independent Rust audits check requested/returned model and effort, every shell
command/reply, inclusive usage, source manifests, qualification from the exact
execution commit, binary hashes, actual WASI close before each effect, original
simulator results, resource intervals, both parallel acquisitions/returns and
complete frame conservation. Eight hash-recomputed mutations per trial are
rejected, including altered binaries and removed participants.

Per-task raw archives contain every model event, prompt, runtime frame, result,
usage and failure record. Source and verifier archives are separate. Executable
copies remain in the local complete archives named in `complete-archives.sha256`.
No raw run was rewritten to obtain an audit pass. The terminal-failure boundary
trials are saved separately under `codebase-direct-wasi-20261016`.

These are two unscored qualification trials. They do not estimate comparative
success rate or speed. Full task acceptance and the 2,592-episode comparison are
unfinished. 1H still needs its actual WASI reconsideration path.
