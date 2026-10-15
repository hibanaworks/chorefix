# QUIC full matrix on the previous core ef65

Run: https://github.com/hibanaworks/hibana-quic/actions/runs/38813319581

Actual QUIC source: `c4404792eaba89494847456d9def8611d33400c5`.
Core: `ef65f0956104441a7bcf2dc4eabbc62930e4c30d`.
TLS: `6e5ece518c30a1478f75f17d23aec72b10208fd8`.
The workflow checks out the immutable source input, distinct from its main
workflow-definition revision. The paired-tree receipt is retained.

All 44 candidate cells executed, with 42 passed, two failed and none unexecuted:

- quiche / handshakecorruption / server
- neqo / rebind-addr / client

Both handshakeloss directions passed. The previous c03 rebinding-cell-only
retry passed on its own unchanged source and is recorded separately.

The failed server archive was decrypted with the retained operator diagnostic
key after clearing finished temporary build caches. The first decryption
attempt failed because the local disk filled; repeating it with the same key
after freeing space succeeded. No key replacement was needed.

The synthetic server log reports several closed connections and several idle
expirations. The last quiche client connection reports receiving zero packets
and a timeout after roughly 30 seconds. These logs do not show a kernel
PhaseInvariant or panic, and do not identify the initiating cause. Later
simulator termination and forced server cleanup are not proof of that cause.
Only selected synthetic text logs are published. Private diagnostic keys, TLS
keylogs and packet captures are excluded.

The rebinding client diagnostic reports a confirmed handshake, one submitted
request, zero completed responses and no application acknowledgement, then
idle expiration. The reference server reports failed path probing and
`NoAvailablePath`. No protocol behavior was changed to turn those errors into
success.

The earlier broad macOS CI-helper test failure was partly a temporary-directory
path issue. Retrying only the 24 unique failed tests with canonical
`TMPDIR=/private/tmp` gives 23 passes and one failure. The remaining encrypted
capture path uses Linux `/proc/self/fd`; it still needs Linux verification.
The failed test list and raw retry output are retained. No checker was relaxed.

This failed source does not qualify core 594cac06, whose consumer checks are
tracked in the new sequential-prefix validation checkpoint.
