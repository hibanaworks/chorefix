# QUIC qualification with the repaired core

External run [38735739939](https://github.com/hibanaworks/hibana-quic/actions/runs/38735739939)
completed successfully on QUIC 44d291a with Hibana 0.9.9 Git c2001ec and TLS
6e5ece5. All 44 distinct registered case/direction cells passed in this single
run. This includes handshakeloss, connectionmigration, IPv6, and address/port
rebinding in both candidate directions.

The independent saved audit compares every phase report with its verdict,
checks all expected cells without duplicates or missing results, and verifies
the source commit, source-pair trees, and exact core/TLS pins in every group.
Sanitized phase reports (lossless `.json.gz`), verdicts, pins, environment records, workflow status,
runner patches, and the aggregate qualification report are retained here.

Reference self-tests were omitted. The runner has the disclosed patches in
`runner-patch.json`, and capture shutdown forwards termination and waits for
capture children. The scenarios are unchanged. This is one 44-cell qualification,
not three release attempts, hardware execution, or a whole-system proof. The
core's own full CI run 38734656111 also completed successfully at c2001ec, with
both final-form and Kani successful. Its status and full gate log are retained
separately from the external interop results. No private keys or TLS secrets
are included.
