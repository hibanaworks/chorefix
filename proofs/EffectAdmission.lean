namespace ChoreoFix

structure Artifact where
  target : Nat
  revision : Nat
  value : Nat
  deriving DecidableEq

structure Verification where
  artifact : Artifact
  passed : Bool
  deriving DecidableEq

/-- A missing receive has no effect. Once both requests have arrived, the
receiver compares them with its actual verification and current artifact. -/
def materialize (current : Artifact) (checked : Verification)
    (first second : Option Verification) : Option Artifact :=
  match first, second with
  | some left, some right =>
      if checked.passed = true ∧ checked.artifact = current ∧
          left = checked ∧ right = checked then some current else none
  | _, _ => none

theorem materialize_spec (current effect : Artifact) (checked : Verification)
    (first second : Option Verification) :
    materialize current checked first second = some effect ↔
      effect = current ∧ checked.passed = true ∧ checked.artifact = current ∧
      first = some checked ∧ second = some checked := by
  cases first with
  | none => simp [materialize]
  | some left =>
    cases second with
    | none => simp [materialize]
    | some right =>
      simp only [materialize]
      by_cases valid : checked.passed = true ∧ checked.artifact = current ∧
          left = checked ∧ right = checked
      · simp [valid, eq_comm]
      · rw [if_neg valid]
        constructor
        · intro impossible
          cases impossible
        · intro matched
          exact False.elim (valid ⟨matched.2.1, matched.2.2.1,
            Option.some.inj matched.2.2.2.1, Option.some.inj matched.2.2.2.2⟩)

theorem missing_first_has_no_effect (current : Artifact) (checked : Verification)
    (second : Option Verification) : materialize current checked none second = none := rfl

theorem missing_second_has_no_effect (current : Artifact) (checked : Verification)
    (first : Option Verification) : materialize current checked first none = none := by
  cases first <;> rfl

theorem a_stale_verification_has_no_effect (current : Artifact) (checked : Verification)
    (first second : Option Verification) (stale : checked.artifact ≠ current) :
    materialize current checked first second = none := by
  cases output : materialize current checked first second with
  | none => rfl
  | some effect =>
    exact False.elim (stale ((materialize_spec current effect checked first second).mp output).2.2.1)

#print axioms materialize_spec
#print axioms missing_first_has_no_effect
#print axioms missing_second_has_no_effect
#print axioms a_stale_verification_has_no_effect

end ChoreoFix
