# QUIC address-rebinding failure

Run: https://github.com/hibanaworks/hibana-quic/actions/runs/38807924806

Actual qualification source: `2af1e696b3d2192ddf25656b4774c6e239a4360a`.
Core: `c03c7af337de6d00c150039450ddb0825b39e3a8`.
All 44 cells executed: 43 passed; `neqo / rebind-addr / client` failed.
The workflow and qualification are failed. Successful cells are not retagged
as a full matrix pass.

The failure archive was successfully decrypted with the retained operator key.
Only the synthetic client log and the final server lines are published here.
The client exited 1 with `connection idle-expired after resource retirement`.
The server had reported failed path probing, retirement of its last path and
`NoAvailablePath`. Later simulator termination and server exit 137 are cleanup,
not evidence of the initiating fault. The sanitizer's certificate category is
not sufficient to diagnose a certificate failure. The retained logs show no
kernel PhaseInvariant or panic, but do not determine whether the root cause
belongs to QUIC, the core, the peer or network behavior.

Only that failed cell was retried on the same immutable source in run
38812020642. It passed: one executed result, zero unexecuted, process exit 0 and
cleanup exit 0. The retained retry source-pair and environment receipts match
QUIC 2af1e69, core c03c7af3 and the original paired source trees. This is a
separate attempt and does not turn the original failed workflow into a pass.
Diagnostic key/TLS secret material is not included in this directory.

A newer source c440479 selects core ef65 and adds failure-only application
counter output. It does not change the old verdict. Its full matrix run
38813319581 finished 42/44, with this same rebinding cell and server-side quiche
handshakecorruption failing. That source has a separate failure record. It is
not qualification of the newer core 594cac06. Linux runtime run 38813324006 is
still pending at this update.
