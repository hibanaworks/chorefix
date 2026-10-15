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

Only that failed cell is retried on the same immutable source in run
38812020642. The retry outcome is pending at this checkpoint and must be kept
separate from the original failed workflow. Diagnostic key/TLS secret material
is not included in this directory.

A newer source c440479 selects core ef65 and adds failure-only application
counter output. It does not change the old verdict. Its new full matrix is run
38813319581; Linux runtime checks are run 38813324006. Both are pending.
