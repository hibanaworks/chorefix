Core revision: 64d4f4966403c967f2b80393acec2c36556bfc5c.

The two-role original reproduction fails on remote main f1c4dd4 and predecessor
0a296df. Resolver preview used completion alone to restart a containing choice
while its child reentered. Commit preflight kept the containing choice, and
the inconsistent fresh ranges caused a post-send invariant panic.

The repair shares the existing descriptor-based fresh-visit calculation between
preview and commit. The strengthened regression checks three containing visits,
two child revisions per visit, and changing the already committed containing
resolver input. Its legitimate next containing visit remains possible.

Local checks: workspace 886 passed / 12 explicitly ignored, strict all-target
Clippy, no_std/thumb, full Lean, related Lean/Z3 supplements, six new Lean kernel
theorems, five new UNSAT obligations with one historical SAT witness, strict
Miri, three selected production route-row Kani harnesses, and size/layout gates.
The selected Kani checks cover row bounds and lane identity, not all runtime
freshness calculations. The Lean model also assumes descriptor-supplied facts.

Measured checked programs: aggregate flash 94,694 bytes, SRAM 5,418 bytes,
maximum stack 2,751 bytes. These values do not bound arbitrary consumer globals.

Remote CI and QUIC checks for this commit are recorded separately. Earlier
0a296df validation is not evidence for this revision. The separate offer preview
failure is not repaired or qualified by this commit.
