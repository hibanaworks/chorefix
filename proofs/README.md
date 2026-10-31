# Effect admission

`EffectAdmission.lean` specifies the data checks at the handoff case's physical
publication boundary. It proves, for arbitrary natural-valued targets, revisions
and contents, that publication requires both received requests, the verifier's
actual positive result, and the same current artifact. The equivalence also
checks that an exactly matching valid request is accepted. It uses Lean's kernel
with `propext`; there are no custom axioms or native evaluation.

Run with the Lean toolchain selected by the pinned Hibana proof package:

```sh
lean +leanprover/lean4:v4.30.0 proofs/EffectAdmission.lean
```

The [native case](../src/tasks/handoff) drives actual projected Endpoints and the
publication receiver. The receiver owns the repository mutably and has no await
between checking the requests and publishing. The model's natural values include
the decoded Rust `u64` values; it does not prove the Rust byte decoder.

This is the data-admission component of T4. The effect-to-receipt correspondence
is specified separately in `EffectReceipts.lean`; arbitrary host Rust remains
outside these proofs.

`endpoint-handoff/` compiles the same handoff, transport, bus and receiver sources
directly. It avoids instantiating unrelated task protocols during diagnosis and
does not contain a copied protocol, a replacement transport or a different runtime.
Its normal manifest uses the application's dependency pins; record any explicit
local core patch separately. The full application tests remain a separate gate.

```sh
cargo +stable test --manifest-path proofs/endpoint-handoff/Cargo.toml
```

The fixture uses hibana-runtime and an in-process transport. It checks
handoff after a failed candidate, further reconsideration, two-resource admission,
old approval, changed targets and arguments, and resuming a pending receive after
the missing request arrives. It is not a remote-network or model-call experiment.

B sends its final verification to A and returns R before A forwards that
verification to the publication receiver. A then uses R for the first publication
request while B uses S for the second. This transfer removes B's ability to
repeat its earlier R operations. The receiver continues to check both requests
against the actual verification and current artifact before publishing.

The native tests use the host event queue. Miri runs the same Endpoint and receiver
scenarios with a polling validation clock because it cannot execute macOS kqueue.
These checks cover Rust memory safety; OS wake delivery is checked by native tests.

```sh
MIRIFLAGS=-Zmiri-strict-provenance cargo +nightly-2026-10-12 miri test \
  --locked --manifest-path proofs/endpoint-handoff/Cargo.toml --lib handoff:: \
  -- --test-threads=1
```

`tests/nested_choice.rs` checks that a nested choice neither previews nor consumes
a later receive before its visible predecessor. It uses the current dependency
pins and the application's actual transport.

`SourceOccurrences.lean` imports Hibana's actual `SourceOccurrenceExecution` and
proves that every executed point remains observed and that no dynamic source
point occurs twice. Its permitted axioms are `propext` and `Quot.sound`.
Build `Hibana.SourceOccurrenceTrace` in the selected Hibana proof package, then
check this module in that package's `lake env lean` environment. The module
introduces no runtime occurrence counter or effect ledger.

`EffectReceipts.lean` composes the same source execution with the receiver
contract. Each invocation consumes two actual resource receives since the
preceding invocation, compares their data with the actual verification and
current artifact, and publishes once if those checks pass. Independent source
actions and either receive order are permitted between invocations. The relation
is erased proof data; the application stores no occurrence IDs or effect ledger.

The proofs establish that every publication has both correctly bound source
receipts and matching data, that distinct resource roles require distinct
receipts, and that neither receipt can authorize a second publication. Receipt
uniqueness follows from Hibana's source execution; an injective effect assignment
is not assumed. `resources_and_effect_receipts_compose` applies custody to both
distinct resource roles and ties each request's sender to its resource. The two
custody histories must project to the same source word as the receiver trace.
It gives an authority bound for each resource, distinct receipts for the two
requests, and no receipt reuse across publications. The permitted axioms remain
`propext` and `Quot.sound`.

After building the pinned Hibana Lean package, check all three modules together:

```sh
export HIBANA_PROOFS=/path/to/pinned/hibana/proofs/lean
mkdir -p target/effect-proofs
export LEAN_PATH="$PWD/target/effect-proofs:$HIBANA_PROOFS/.lake/build/lib/lean"
for module in EffectAdmission SourceOccurrences EffectReceipts; do
  lean +leanprover/lean4:v4.30.0 -j 1 -R proofs \
    -o "target/effect-proofs/$module.olean" "proofs/$module.lean"
done
```

T3 reuses Hibana's `OwnedCallHistory`, `OccurrenceResourceCustody` and source
occurrence results under their binding and transport assumptions. A legitimate
request can remain executable after its sender drops its Endpoint; authorization
comes from that admitted operation. Requiring a live sender Endpoint at effect
time would incorrectly discard such pending work. Local errors neither undo an
effect nor authorize initial reissuance.

The adapter obligation is explicit: each modeled receive must be the actual
Endpoint receive used by the physical call, with its exact decoded arguments,
verification and current artifact. The theorem does not derive that obligation
from arbitrary Rust or an unqualified frame log. The native receiver tests cover
the concrete call path; qualification of the remaining receivers and their
projection/trace correspondence is still required. The artifact model covers
target, revision and contents; other simulator arguments need their own bindings.
