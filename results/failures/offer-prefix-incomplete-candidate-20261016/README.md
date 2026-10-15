# Rejected partial prefix repair

The c03 core fails three of six native 3E tests when the two workaround
`DocumentReviewReady` acknowledgements are removed. The observed unexpected
document branch is the later combine notification (logical label 127).

The first prefix-repair candidate used the candidate's projected parent arm only
when committed history also had no choice. It passed the standalone prefix
example and broad core checks, but still failed one of six actual native 3E
tests after a completed parent revision. The unexpected document message was
the B revision notification (logical label 94).

The source patch and both failed logs are retained. These are rejected
development candidates, without model calls or scored episodes. Neither their
passing subsets nor unrelated core passes count as final integration evidence.

The final ef65 repair uses the candidate's projected path whenever the preview
has no arm, including masked completed parent choices. Reentry still reads
committed history. Its strengthened regression and actual consumer checks are
in `../../validation/offer-prefix-dependency-20261016/`.
