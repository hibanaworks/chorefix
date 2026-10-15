# Direct WASI readiness preflight, 16E

This is an unscored integration run, not a completed comparative benchmark.
Three GPT-6 Luna xhigh subjects chose their own wash commands. The actual
published Hibana 0.9.9, pinned hibana-wasip1-runtime, and pinned std hibana-wash
were used. The task inputs and simulator were not changed. Original 16E has
no injected domain failure at any scenario depth, so depth 0/1/2 must not be
reported as three different failure conditions for this task.

The source archive is the exact dirty-source snapshot from before model calls.
The consumer HEAD alone does not identify that snapshot. Runtime and guest
binaries remain in the local frozen run directory; result.json records hashes.
The source manifest, locked dependencies, compiler, and raw model/carrier events
are retained here. Gzip files preserve raw bytes without normalizing whitespace.

Independent persisted-event audit passed: 3 participants, 11 shell calls,
384 frames / 768 send/receive observations, 3 original simulator calls,
220,215 inclusive total tokens, and 27.127 elapsed seconds. This measures one
integration trial. It does not establish a comparative completion rate, speed
advantage, or acceptance of the other 47 tasks. No development subagents ran.

All actual shared requests originated at the WASI role Endpoint and reached
the service owner before the simulator effect; required resource grants and
cross-role notifications preceded dependent effects. The received replies
match the simulator results. Private file operations remained within ChoreoFS.
The companion deterministic real-guest tests also cover malformed data followed
by valid work, private isolation, premature dependent operations, duplicate
rejection after completion, and premature guest exit.
