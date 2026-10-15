# Independent service endpoints in one 6E global

The extra real-WASI tests failed in all six old cases. After one review service
completed, a duplicate request kept its rejection loop live. The shared review
caller endpoint could offer that service's next branch while the guest expected
the other service's close response. The guest incorrectly assumed that its
previous request scoped `offer()`. The full failed traces and source are retained.

The repair projects two caller roles from the same task global. One original
reviewer guest directly owns both service endpoints and its private-I/O endpoint.
Each actual path/fd selects that service's caller endpoint. Both use the public
send/recv/offer operations. All service and task edges remain in the same
global/session. No second graph, forwarding channel, response dispatcher state,
progress flag, permission table, or hidden localside wrapper is added. The task
still has three model participants and three Wasm guests.

The six repaired cases preserve both review orders and failure depths 0/1/2.
They include invalid arguments followed by valid work, wrong-target rejection,
premature merge rejection, chunked JSON writes, duplicate implement/merge/review
rejection after completion, continued work on the other feature, both merges,
and clean guest exit. The unchanged original simulator replay and resource/
frame audit passed. Rust 26, Python 43, Clippy, formatting, source inventories,
and the unchanged 2,592-episode matrix checks passed.

This is a ChoreoFix application correction. These observations do not establish
a new Hibana defect or justify another core change. The previous Luna source
checkpoint remains separate. Full-task benchmark acceptance and scored counts
remain zero.
