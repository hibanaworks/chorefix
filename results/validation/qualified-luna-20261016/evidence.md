Four unscored trials ran the qualified application build with actual GPT-6 Luna
xhigh participants. None had an artificial barrier. The source, executable,
module, prompts and checks were frozen before each trial, with subsequent events
recorded as they occurred. The execution commit is 5646c0b, the application build
source is 1876bfb and the independent model verifier source is dc790b5.

| Task | Participants and simultaneous turns | Disjoint operations | Domain calls | Inclusive tokens |
| --- | ---: | ---: | ---: | ---: |
| 9E | 3 | 1 eat | 6 | 247716 |
| 9M | 5 | 2 eats | 10 | 390114 |
| 9H | 7 | 3 eats | 14 | 552141 |
| 2M | 5 | 2 writes | 20 | 551222 |

All four protocol and original domain outcomes are complete, with zero resource
violations. Both acquisition requests precede the first grant and both returns
precede the first acknowledgement. Model commands and actual guest replies match
exactly. Raw requested/returned model settings, tool events, usage, original
simulator effects and delivered frames passed independent checks. Inclusive
tokens include input and output, including cached input. Participant turns are
not counts of the backend's individual inference requests.

Each case rejects eight altered copies after updating their outer hashes:
model substitution, invalid usage, forged commands, missing replies, false
completion, changed upstream source, an unqualified executable and a changed
participant set. The mutation reports pin both verification executables and the
original event hash. These checks do not modify the actual run files.

Each raw archive includes prompts, model events, results, version information,
the simulator journal and frame records. Execution source archives are stored
per case because their selected qualification evidence differs. Complete
binary-inclusive archives remain under output/ with recorded hashes. The
qualified executables are frozen under output/qualified-direct-wasi-20261016-v2.
The current model audit requires the qualification record in the execution's
Git commit and the exact qualified protocol verifier. Older version-1 data uses
its archived verifier source instead of being reinterpreted as version 2.

From the source of the verification tools, run:

```sh
cargo +1.95.0 build --locked --manifest-path tools/runner/Cargo.toml
tools/runner/target/debug/choreofix-experiments --audit output/direct-wasi-preflight/philosophers-luna-20261016-001/9E output/qualified-direct-wasi-20261016-v2/philosophers-boundary
```

Use the analogous 9M/9H directories for the other philosophers. The 2M trial is
under output/direct-wasi-preflight/writing-qualified-luna-20261016-001/2M and
uses the qualified writing-boundary executable. These are qualification trials,
not the 2592-episode comparison matrix or comparative success-rate estimates.
