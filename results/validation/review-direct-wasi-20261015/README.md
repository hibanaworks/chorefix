# Direct WASI review trials, 6E

Three unscored integration trials used all three original participants, GPT-6
Luna with xhigh reasoning, the unchanged original 6E simulator and inputs, the
std wash guest, and the actual WASIp1/ChoreoFS path. No development agents ran.
Source, prompts, raw model and carrier events, results, and audit outputs are
retained. Gzip preserves raw event bytes. `source.tar.gz` is the common source
snapshot saved before any model call. All three source manifests are identical.
The local full archive additionally retains the frozen runtime and Wasm.

| Failure depth | Participants | Shell calls | Domain calls | Frames | Seconds | Inclusive tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 3 | 20 | 6 | 798 | 59.481 | 361,915 |
| 1 | 3 | 23 | 8 | 1,020 | 45.387 | 412,438 |
| 2 | 3 | 22 | 10 | 1,156 | 46.913 | 397,336 |

Every trial completed both correct merges, consumed all accepted protocol
frames, and recorded no simulator violations. The first depth review calls fail
according to the original simulator. This schedule is global across features,
not a per-feature failure count. At depth 2 the model reviewed A, B, A, B, with
one revision of each feature. Independent replay reproduces those results.
These are three integration observations, not an estimate of success rate or a
paired performance comparison. Earlier failed development trials remain saved.

The audit checks exact model/effort replies, turn completion, tool commands,
inclusive usage, source/binary hashes, carrier identity and consumption, actual
WASI-to-owner close before effect, original simulator replay, nonoverlapping
REPO use, code-ready notifications, actual review feedback, and both merges.
Altered copies remove close/notification frames, change a source, forge or remove
feedback, and overlap resource intervals. The auditor must reject each even
after the outer event hash is recomputed. Mutation checks never alter originals.

Hibana is package version 0.9.9 at Git repair c2001ec, not a new registry
release. The consumer source is 4a9937f, wash is 12e137a, and WASI runtime is
4ebdad1. Exact revisions and binary hashes appear in each result and dependency
tree. The service objects require the matching canonical feature/branch. The
developer's implementation reply includes actual review feedback and waits for
it. These interface differences from the unchanged baseline are explicit.

The endpoints constrain communication. Trusted localsides materialize effects
only after the prescribed steps. These trials do not prove arbitrary Rust
effects safe, every global schedule live, all 48 ports faithful, or the 2,592-run
comparison complete. Full-task acceptance and scored comparison counts remain 0.
