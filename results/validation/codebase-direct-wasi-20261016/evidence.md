# Codebase direct WASI qualification

Source: `80a46eb86aaa7600acb59bb5d897c9c603ca0c83`.
No model was called in these boundary trials. No scored episode is claimed.

The 1E/1M guests, service owners, physical modules and private I/O use one global
and session 20. Each WASI caller sends its service request to the materializer's
Endpoint. The materializer consumes the request, both module grants and the
local effect step before calling the unchanged simulator. There is no separate
relay session or task progress table in this path.

Both acquisition requests precede the first grant send. Both returns precede
the first return acknowledgement. The audit rejects a serialized-acquisition
copy while retaining all its frame identities. Disjoint 1M commits overlap.
A's local test waits for B's commit, so completion also checks that testing does
not retain the modules or block a different developer's shared work.

| Task | Guests | Domain operations | Maximum concurrent commits | Violations |
| --- | ---: | ---: | ---: | ---: |
| 1E | 3 | 12 | 1 | 0 |
| 1M | 5 | 20 | 2 | 0 |

The fixed stimuli include private calculation, early and duplicate operations,
wrong objects, invalid JSON arguments, a third repeated module and chunked I/O.
The valid module pair is submitted in reverse order without comma whitespace.
All actual results agree with the original simulator's events and the frames
delivered back to the guest. Seven mutations per task are rejected.

Separate terminal-test-failure trials finish their protocols but preserve
`simulator_complete: false`, with four operations per developer and no retry.
They are negative boundary tests and are excluded from successful episodes.
1H needs its own result-driven reconsideration path and is not covered here.

The original checklist's sequential acquisition procedure is deliberately
replaced by parallel requests. Resource exclusion, both grants before commit,
returns before testing, participant identities and assigned work are preserved.
The original simulator and all baseline source files remain byte-identical.
The argument interface is stricter than the original simulator's permissive
matching. These differences are recorded in `benchmark/experiment-contract.json`.

Validation: 27 Rust tests, strict Clippy, 43 Python tests, four runner unit tests
and an explicitly invoked real-WASI process test pass. The new build also passes
the six writing/dining boundary configurations. Logs, raw events, source archive
and hashes are saved here. Executables are retained in the local frozen build
and complete archive. The previous CI success belongs to 5646c0b, not this source.

Full task acceptance, live model qualification and scored comparisons remain
pending. The paper is not being revised during this work.
