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

This is the data-admission component of T4. It does not prove that arbitrary host
Rust follows the receiver contract, that a host call belongs to a particular
source occurrence, or that an operation cannot be invoked twice. Those require
the adapter's correspondence with the actual received source operations.

`endpoint-handoff/` compiles the same handoff, transport, bus and receiver sources
directly. It avoids instantiating unrelated task protocols during diagnosis and
does not contain a copied protocol, a replacement transport or a different runtime.
Its normal manifest uses the application's dependency pins; record any explicit
local core patch separately. The full application tests remain a separate gate.

```sh
cargo +stable test --manifest-path proofs/endpoint-handoff/Cargo.toml
```

The fixture uses one native Tokio executor and an in-process transport. It checks
handoff after a failed candidate, further reconsideration, two-resource admission,
old approval, changed targets and arguments, and resuming a pending receive after
the missing request arrives. It is not a remote-network or model-call experiment.
The auxiliary computation's 20 ms delay is a labeled scheduling fixture.

`SourceOccurrences.lean` imports Hibana's actual `SourceOccurrenceExecution` and
proves that every executed point remains observed and that no dynamic source
point occurs twice. Its permitted axioms are `propext` and `Quot.sound`.
Build `Hibana.SourceOccurrenceTrace` in the selected Hibana proof package, then
check this module in that package's `lake env lean` environment. The module
introduces no runtime occurrence counter or effect ledger.

Source-point uniqueness is not yet a one-to-one theorem for Rust effects. That
requires binding each effect invocation and both resource receipts to the same
designated operation. Different source points may carry equal payload bytes.
The remaining composition must preserve that distinction, rather than treating
equal bytes or a locally successful call as an occurrence identity.

T3 reuses Hibana's `OwnedCallHistory`, `OccurrenceResourceCustody` and source
occurrence results under their binding and transport assumptions. A legitimate
request can remain executable after its sender drops its Endpoint; authorization
comes from that admitted operation. Requiring a live sender Endpoint at effect
time would incorrectly discard such pending work. Local errors neither undo an
effect nor authorize initial reissuance. The complete T3/T4 composition must relate
the received requests to those admitted occurrences; this file alone is not that
composition or a proof of the complete Rust implementation.
