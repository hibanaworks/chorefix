The same Endpoint reproduction fails on remote main
f1c4dd4bab211b005687cfec15bb53a1f77500ed and on the ordinary-ingress repair
c03c7af337de6d00c150039450ddb0825b39e3a8. `repro.patch.gz` adds the test to the
latter revision's `nested_resolver_reentry` suite, using its production test
transport and Tokio current_thread without an OS I/O driver.

After Start, D must first observe logical label 4 (BeginDocB). The controller
has also queued label 6 (BeginCombine) on another lane. Offer instead previews
label 6 before the visible prefix. This is an existing preview defect, separate
from the service-rework regression repaired by c03c7af3. The first reproduction
here tests the returned preview, rather than claiming an out-of-order effect.

The original four-role ChoreoFix carrier reproduction is retained under
`probes/research-report-20261016/tests/nested.rs`; its second test separately
checks that trying to consume the later step is rejected. This issue is still
open. Keep the core PR in draft and retain the failure when reporting passing
component or QUIC checks. No actual model or scored episode ran in these tests.
