These are fixed-input actual-WASI qualification runs for 9E/M/H. There are no
model calls or scored episodes in this directory.

| Task | Participants | Original domain calls | Simultaneous eating | Violations |
| --- | ---: | ---: | ---: | ---: |
| 9E | 3 | 6 | 1 | 0 |
| 9M | 5 | 10 | 2 | 0 |
| 9H | 7 | 14 | 3 | 0 |

The default original simulator has one think and one eat step per participant.
The global connects the real WASI caller, materializing owner and adjacent fork
roles in one session. Both acquisition requests precede the first grant, and
both returns precede the first acknowledgement. Each physical fork has two
explicit visits. The frame audit checks conservation, requeues, exact payloads,
fresh WASI closes, ownership intervals, original effects and delivered replies.
All participants exit only after returning their forks. The outer test supplies
first-eater barriers to challenge independence. These are not performance data.

The original simulator uses agent_id to identify its participant and ignores
the philosopher string. The final port preserves this rule. Tests submit
ANOTHER_DISPLAY_NAME, but every effect and reply still names its projected
caller. Invalid JSON and non-string input are rejected before an effect. Early
and duplicate effects, chunked I/O and private computation are also exercised.
The initial unnecessary exact-name restriction is kept separately under
development-strict-argument, with its source and observations.

Each task rejects seven altered copies, including a serialized acquisition with
all frame identities retained. The exact application source is archived, with
source revision and binary hashes in the build qualification record. Rust 27,
Clippy, unchanged Python 43, upstream input and baseline byte checks passed.
The standalone model transport also passes four tests and an explicitly run
process integration against the newly built writing runtime. Frozen executables
are under output/qualified-direct-wasi-20261016-v2. Models use the qualification
record to select an executable before a trial. That record never participates
in a guest's permissions or task progress.

The expected model-trial settings use the original delay multiplier 1.0 and no
artificial barriers. Those trials and full task-fidelity acceptance remain
separate. The source still uses Hibana 0.9.9 patched at c2001ec. No core change
was made here. The paper, comparison implementations and original inputs were
not changed.
