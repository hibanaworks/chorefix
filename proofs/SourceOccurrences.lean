import Hibana.SourceOccurrenceTrace

namespace ChoreoFix
open Hibana

theorem executed_source_points_are_observed
    {source : EndpointDelegation.Source} {first last : SourceOccurrenceCut source}
    {points : List SourcePoint} (execution : SourceOccurrenceExecution first last points) :
    ∀ point ∈ points, last.observed point = true := by
  induction execution with
  | nil => simp
  | @tail before points point after prior step ih =>
    cases step with
    | advance present pending ready =>
      intro candidate member
      simp only [List.mem_append, List.mem_singleton] at member
      by_cases same : candidate = point
      · simp [same]
      · have earlier : candidate ∈ points := member.resolve_right same
        simp [same, ih candidate earlier]

/-- Dynamic occurrence identity comes from the source execution. No runtime
counter or second effect ledger is introduced. -/
theorem source_points_are_consumed_once
    {source : EndpointDelegation.Source} {first last : SourceOccurrenceCut source}
    {points : List SourcePoint} (execution : SourceOccurrenceExecution first last points) :
    points.Nodup := by
  induction execution with
  | nil => simp
  | @tail before points point after prior step ih =>
    have fresh : point ∉ points := by
      intro earlier
      have marked := executed_source_points_are_observed prior point earlier
      cases step with
      | advance present pending ready =>
        rw [pending] at marked
        contradiction
    apply List.nodup_append.mpr
    refine ⟨ih, by simp, ?_⟩
    intro earlier member appended singleton same
    have appendedAt : appended = point := by simpa using singleton
    exact fresh ((same.trans appendedAt) ▸ member)

#print axioms executed_source_points_are_observed
#print axioms source_points_are_consumed_once

end ChoreoFix
