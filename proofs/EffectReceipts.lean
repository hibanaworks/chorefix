import EffectAdmission
import SourceOccurrences

namespace ChoreoFix
open Hibana Hibana.EndpointDelegation

/-- An actual request receive, identified by its dynamic source occurrence.
Equal argument bytes do not identify equal receives. -/
structure ResourceRequest where
  occurrence : Nat
  value : Verification

def ResourceRequest.point (request : ResourceRequest) : SourcePoint :=
  .committed ⟨request.occurrence, .receive⟩

/-- The source binding is checked separately from the request's decoded data. -/
def ResourceRequest.matches (request : ResourceRequest)
    (plan : SourceOccurrenceCut source) (message : Message) : Prop :=
  ∃ binding,
    plan.path.bindings.find? (fun row => row.1 == request.occurrence) =
      some (request.occurrence, binding) ∧ binding.message = message

theorem different_resources_have_different_receipts
    {first second : ResourceRequest} {firstMessage secondMessage : Message}
    {plan : SourceOccurrenceCut source}
    (different : firstMessage.sender ≠ secondMessage.sender)
    (firstAt : first.matches plan firstMessage) (secondAt : second.matches plan secondMessage) :
    first.point ≠ second.point := by
  intro same
  have occurrence : first.occurrence = second.occurrence := by
    simpa [ResourceRequest.point] using same
  obtain ⟨left, leftAt, leftMessage⟩ := firstAt
  obtain ⟨right, rightAt, rightMessage⟩ := secondAt
  rw [occurrence] at leftAt
  have binding : left = right := congrArg Prod.snd (Option.some.inj (leftAt.symm.trans rightAt))
  have message : firstMessage = secondMessage :=
    leftMessage.symm.trans ((congrArg Binding.message binding).trans rightMessage)
  exact different (congrArg Message.sender message)

structure Publication where
  first : ResourceRequest
  second : ResourceRequest
  current : Artifact
  checked : Verification
  artifact : Artifact

/-- Interpret the receiver's one physical call after its two receives and data
checks. A rejected comparison emits no publication. -/
def publications (current : Artifact) (checked : Verification)
    (first second : ResourceRequest) : List Publication :=
  match materialize current checked (some first.value) (some second.value) with
  | none => []
  | some artifact => [⟨first, second, current, checked, artifact⟩]

/-- Erased adapter correspondence. Each invocation consumes both receives
since the preceding invocation. The segment may include arbitrary independent
source actions and either receive order. No freshness or injectivity premise
is imposed: those properties follow from the source execution below.

This relation describes the receiver contract, not application state. Relating
the actual Rust calls and decoded values to it remains the adapter obligation. -/
inductive ReceiverEffects (plan : SourceOccurrenceCut source)
    (firstMessage secondMessage : Message) : List SourcePoint → List Publication → Prop where
  | nil : ReceiverEffects plan firstMessage secondMessage [] []
  | pending (prior : ReceiverEffects plan firstMessage secondMessage points effects)
      (segment : List SourcePoint) :
      ReceiverEffects plan firstMessage secondMessage (points ++ segment) effects
  | invoke (prior : ReceiverEffects plan firstMessage secondMessage points effects)
      (segment : List SourcePoint) (current : Artifact) (checked : Verification)
      (first second : ResourceRequest)
      (firstReceived : first.point ∈ segment) (secondReceived : second.point ∈ segment)
      (firstMatches : first.matches plan firstMessage)
      (secondMatches : second.matches plan secondMessage) :
      ReceiverEffects plan firstMessage secondMessage (points ++ segment)
        (effects ++ publications current checked first second)

private theorem publication_anchors_are_received_once
    (receiver : ReceiverEffects plan firstMessage secondMessage points effects) :
    List.Sublist (effects.map fun effect => effect.first.point) points ∧
      List.Sublist (effects.map fun effect => effect.second.point) points := by
  induction receiver with
  | nil => exact ⟨.refl _, .refl _⟩
  | pending prior segment ih =>
      exact ⟨ih.1.trans (List.sublist_append_left _ _),
        ih.2.trans (List.sublist_append_left _ _)⟩
  | invoke prior segment current checked first second firstReceived secondReceived
      firstMatches secondMatches ih =>
      cases result : materialize current checked (some first.value) (some second.value) with
      | none =>
          simpa [publications, result] using
            And.intro (ih.1.trans (List.sublist_append_left _ segment))
              (ih.2.trans (List.sublist_append_left _ segment))
      | some artifact =>
          simpa [publications, result] using And.intro
            (ih.1.append (List.singleton_sublist.mpr firstReceived))
            (ih.2.append (List.singleton_sublist.mpr secondReceived))

theorem every_publication_has_both_source_receipts_and_matching_data
    (receiver : ReceiverEffects plan firstMessage secondMessage points effects)
    (effect : Publication) (present : effect ∈ effects) :
    effect.first.point ∈ points ∧ effect.second.point ∈ points ∧
      effect.first.matches plan firstMessage ∧ effect.second.matches plan secondMessage ∧
      effect.artifact = effect.current ∧ effect.checked.passed = true ∧
      effect.checked.artifact = effect.current ∧
      effect.first.value = effect.checked ∧ effect.second.value = effect.checked := by
  induction receiver with
  | nil => simp at present
  | pending prior segment ih =>
      obtain ⟨first, second, rest⟩ := ih present
      exact ⟨List.mem_append_left _ first, List.mem_append_left _ second, rest⟩
  | invoke prior segment current checked first second firstReceived secondReceived
      firstMatches secondMatches ih =>
      rcases List.mem_append.mp present with earlier | actual
      · obtain ⟨left, right, rest⟩ := ih earlier
        exact ⟨List.mem_append_left _ left, List.mem_append_left _ right, rest⟩
      · cases result : materialize current checked (some first.value) (some second.value) with
        | none => simp [publications, result] at actual
        | some artifact =>
            have same : effect = ⟨first, second, current, checked, artifact⟩ := by
              simpa [publications, result] using actual
            subst effect
            obtain ⟨artifactAt, passed, checkedAt, firstAt, secondAt⟩ :=
              (materialize_spec current artifact checked _ _).mp result
            exact ⟨List.mem_append_right _ firstReceived,
              List.mem_append_right _ secondReceived, firstMatches, secondMatches,
              artifactAt, passed, checkedAt, Option.some.inj firstAt, Option.some.inj secondAt⟩

/-- One source execution and the receiver's consuming interpretation imply
that neither resource request can authorize two physical publications. -/
theorem source_execution_prevents_reusing_either_effect_request
    (execution : SourceOccurrenceExecution first last points)
    (receiver : ReceiverEffects plan firstMessage secondMessage points effects) :
    (effects.map fun effect => effect.first.point).Nodup ∧
      (effects.map fun effect => effect.second.point).Nodup := by
  have unique := source_points_are_consumed_once execution
  have received := publication_anchors_are_received_once receiver
  exact ⟨List.Nodup.sublist received.1 unique, List.Nodup.sublist received.2 unique⟩

/-- Custody and effect correspondence use the same source word. The adapter
contract supplies actual receives and values; the core supplies affine custody
and source occurrence uniqueness. No injective effect assignment is assumed. -/
theorem custody_and_effect_receipts_compose
    (custody : OwnedCallHistory.History images resource choreo value events cut)
    (trace : SourceOccurrenceTrace images choreo plan
      (OwnedCallHistory.sourceWord events) calls points)
    (receiver : ReceiverEffects plan firstMessage secondMessage points effects) :
    cut.custody.authorityCount ≤ 1 ∧
      (effects.map fun effect => effect.first.point).Nodup ∧
      (effects.map fun effect => effect.second.point).Nodup ∧
      ∀ effect ∈ effects,
        effect.first.point ∈ points ∧ effect.second.point ∈ points ∧
        effect.first.matches plan firstMessage ∧ effect.second.matches plan secondMessage ∧
        effect.artifact = effect.current ∧ effect.checked.passed = true ∧
        effect.checked.artifact = effect.current ∧
        effect.first.value = effect.checked ∧ effect.second.value = effect.checked := by
  have execution := a_source_occurrence_trace_is_its_reconstructed_execution trace
  have unique := source_execution_prevents_reusing_either_effect_request execution receiver
  exact ⟨(a_joint_call_history_preserves_affine_custody custody).1, unique.1, unique.2,
    every_publication_has_both_source_receipts_and_matching_data receiver⟩

#print axioms every_publication_has_both_source_receipts_and_matching_data
#print axioms different_resources_have_different_receipts
#print axioms source_execution_prevents_reusing_either_effect_request
#print axioms custody_and_effect_receipts_compose

end ChoreoFix
