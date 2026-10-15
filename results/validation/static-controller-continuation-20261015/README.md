# Continuation and parallel-resolver validation

This records the final core source c2001ec. A two-role rolled send-choice/reply
protocol fails with PhaseInvariant at 5440370 and passes with c2001ec. The earlier
change incorrectly dropped a send-selected controller choice while locating its
receive continuation. Keeping that committed choice does not retain completed
observer/resolver decisions or prevent enabled parallel resolver work.

The two current QUIC ingress failures also pass. Default tests/doctests passed 593 and hq passed 598, with zero failures, using
the clean local core checkout at c2001ec. The final QUIC revision 44d291a pins
the exact Git source in all six consumers. Its three ingress tests also passed,
and the dependency tree selects that same source directly and through TLS.
Source-pin checks (4), matrix checks (13) and driver tests passed. New external
peer qualification and final GitHub CI remain separate pending gates.
No main merge or registry publication is claimed.

The first receive-entry diagnosis was disproved by probes 001/002. Probe 003
restored all historical choices and passed QUIC but reintroduced the native
parallel-resolver stall. Those probes are not the adopted fix. The final source
keeps only a send-selected controller's committed choice alongside live choices.

Rust 883 passed, zero failed, 12 explicit ignored. Full Lean, supplemental Lean
10 claims, Z3 7 SAT/6 UNSAT, two Miri regressions and four relevant Kani harnesses
passed. Kani and supplemental models are scoped evidence, not verification of
every Rust execution or QUIC behavior.

The 6E boundary tests use real wash/WASIp1/ChoreoFS and unchanged original
simulator effects. All three depths passed five stress sets and one exact Git
source build. The independent auditor replays domain results, checks resource
intervals and frame identity, and confirms both merges. Sampled delays are checked
against their original range rather than required to match a new random replay.
Removing the final domain result from a copy of the trace is rejected.

These are deterministic component checks with no model calls. They do not count
as accepted full ports or scored runs of the planned 2592-case comparison.
