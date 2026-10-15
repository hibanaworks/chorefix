# Direct WASI result reads and parallel module handoffs

Runtime source: `29344a8a4c6800ff583a3d24143fa592de6f6a17`.
All trials here use fixed external stimuli, with no model calls or scored episodes.
The source archive, qualification inputs and exact executables are pinned by
`results/qualified-direct-wasi-build.json`. The frozen executables stay locally
in `output/qualified-direct-wasi-20261016-api/`.

API task 8E connects all three real WASI guests, private I/O, service owners,
DATABASE, TEST_ENV and two result-file endpoints through one global/session28.
Each WASI service request reaches its owner through that global. The owner
consumes the request, resource grant and local effect step before ChoreoFS
materializes the original simulator operation. No relay graph or progress table
is used. The backend waits for the migration, and the test waits for the backend.

The developer's result-file open and the tester's actual result notification
are independent inputs in `par`. The result owner must receive both before
replying. The fixed stimulus sends both opens before the test result is released.
Both recipients read the complete actual result twice. Private shell work stays
independent. Pathnames and rights do not grant the test-result read by themselves.

| Failure depth | Operations | Guest exits | Result recipients | Domain complete | Violations |
| --- | ---: | ---: | ---: | --- | ---: |
| 0 | 3 | 3 | 2 | true | 0 |
| 1 | 3 | 3 | 2 | false | 0 |
| 2 | 3 | 3 | 2 | false | 0 |

Depth1/2 preserve the unchanged simulator's terminal test failure, with no retry.
A completed protocol is not a successful task in these cases. Checklist O2
conflicts with the task description and simulator; full acceptance remains open.
Eight persisted-event mutations per depth are rejected, including missing
notification/read, changed arguments/result, altered frames and false success.

The previous native 1E/1M broker path had a schedule-dependent circular wait.
Its CI failure, local failure and old source are retained in before-native-fix.
The repaired native path projects physical modules from one global and removes
broker_once and its separate sessions. Independent acquisition and return
requests still precede either reply wait. A regression requires independent
A/C commits to start together. All three terminal-path regressions pass in each
of32 repetitions (96 test executions). Native1H's earlier broker foundation is
still unqualified and needs its own integration; this repair does not cover it.

The same new build requalifies 1E/M, 2E/M/H and9E/M/H. Actual constrained work
overlaps1/2 for commits,1/2/3 for writes and1/2/3 for eating, with no resource
violations. The delayed tests/reviews complete after another participant commits
or writes, checking that private review does not retain the returned resources.

Validation: all28 root Rust tests, strict Clippy,43 Python tests, four runner
unit tests and the explicit real-WASI runner process test pass. Original48 tasks,
825 checklist items and baseline111/59 files are byte-identical. The process
test confirms that independent simulator calls can overlap while stdout is read.
The first incorrect sequential input layout was rejected by projection; its
source and diagnostic are retained separately. No Hibana core change is made.

Next: live Luna xhigh qualification, remaining task families and original-loop
baseline comparison. Full task acceptance and scored comparisons remain pending.
