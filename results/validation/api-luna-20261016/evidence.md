# API task through the actual WASI boundary

Execution consumer: `50ce492fab130c6c8fdaeb4ad0b58e08cf372148`.
Runtime source: `29344a8a4c6800ff583a3d24143fa592de6f6a17`.
External verification source: `7a3074868c78f1eb98ab364186ec75e0ca228e8c`.

Three unscored trials run the original 8E task at failure depths0/1/2, with
seed0 and original delay multiplier1.0. No artificial barrier is used. Each
trial has three real GPT-6 Luna xhigh participants. Their actual model turns
overlap, and all shared effects pass through the actual Rust std WASI shell,
projected requests, ChoreoFS owners and unchanged simulator. Prompts do not
supply a command recipe. Private shell computation remains independent.

| Depth | Subjects | Domain operations | Shell calls | Inclusive tokens | Seconds | Domain complete | Early result opens |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| 0 | 3 | 3 | 11 | 215,036 | 22.217 | true | 1 |
| 1 | 3 | 3 | 12 | 231,064 | 25.240 | false | 1 |
| 2 | 3 | 3 | 12 | 230,883 | 23.045 | false | 1 |

All three protocols complete with no resource violation. Each trial delivers
the actual test result to both developers through their projected read endpoints.
In each case one open arrives before the test result and waits for the required
notification. The other arrives later. Both complete reads match the exact
original result. The two false domain outcomes remain terminal failures,
without invented retries or successful-task claims.

Totals: nine subjects, nine domain operations,35 shell calls and676,983 inclusive
tokens. Inclusive usage includes cached input tokens and output tokens.
Participant turns are App Server turns, not counts of internal inference calls.
These trials do not estimate comparative success rates or speed.

The execution-time protocol checker and all original binaries remain pinned.
Its early_result_opens output name counted all opens. The separate corrected
timing audit compares the actual open and test-result positions and is the
source of the table's early-open counts. It does not change runtime admission.
Each saved trial includes both the qualified protocol/model audit and corrected
timing audit, with their separate source versions. Do not use the older metric
name as evidence that both model requests arrived early.

Eight model-evidence mutations per trial,24 total, are rejected after recomputing
outer hashes. The first negative-trial mutation test exposed an external test
generator defect: setting false to false did not alter a failed score. The fixed
generator inverts the saved Boolean and rejects the changed result. No model
trial or original raw log was rerun or rewritten for this correction.

The raw archives include participant events, prompts, commands, replies, usage,
carrier frames, simulator journals and run results. Execution source snapshots
and the verification source archive are separate. The local complete archives
also retain executables. Checksums identify those archives and verifier binaries.
Runner unit tests and strict Clippy pass after the audit corrections.

Checklist O2 still conflicts with the task description and simulator's terminal
failure rule. Full task acceptance and all scored comparisons remain pending.
The remaining families and native1H integration are not covered by these trials.
