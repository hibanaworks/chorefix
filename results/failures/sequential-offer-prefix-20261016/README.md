# Runtime admission rejected causally early input

These are retained development failures, without model calls or scored episodes.
The reproducing global is legal. Its projection contains the required
predecessors; this defect is in runtime event admission and passive offer
classification, not in compiling the global.

`regression.rs` is the strengthened four-role test used for both negative
baselines and the final repair. A resolver-driven rolled fragment is inside a
parallel composition, followed by another rolled resolver. The sender performs
only permitted Endpoint operations. One receiver's accepted prefix on lane 1 is
delayed while two ordered later receives on lane 0 share a wire key.

- `baseline.log`: exact e500c7e39931ff6736b63d2227fd90fba0a497ea runtime plus
  this test and its integration-module hook. It fails when pending is expected.
- `head-only.patch.gz` and `head-only.log`: checking the projected predecessor
  before current lane-head membership is necessary but insufficient. This
  partial repair still fails because multiple ordered future matches are
  mistaken for ambiguous enabled input.
- `lane-head-diagnostic.log` and `pending-alias-diagnostic.log`: temporary
  diagnostic builds establishing those two causes. Diagnostic traces were
  removed before the final core verification and commit.

Final core repair: 594cac066d8b99b5caf852d061b8ee7121dd8698. The passing source
and checks are recorded in the adjacent validation directory. No application
permission flag, helper flow or extra acknowledgement is needed for this fix.
The separate 1H global exclusion defect is not fixed by this runtime change.
