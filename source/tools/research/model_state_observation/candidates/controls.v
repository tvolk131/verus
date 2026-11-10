Require Import guarding.internal.na_invariants_fork.
From lrust.util Require Import atomic_lock_counter.
From ModelStateObservation Require Export instances.
From lrust.lang Require Import proofmode notation lifting.
From lrust.util Require Import non_atomic_cell_map.
Set Default Proof Using "Type".

Section resource_controls.
  Context `{!heapGS Σ, !invGS Σ, !na_invG Σ, !alc_logicG Σ}.
  Lemma returned_value_cannot_disagree σ p v w : v ≠ w →
    heap_ctx σ -∗ p ↦ v -∗ ⌜heap_values σ !! p = Some w⌝ -∗ False.
  Proof.
    iIntros (Different) "Heap Point %Wrong".
    iDestruct "Heap" as (hF) "(Auth & _)".
    rewrite /heap_mapsto.
    iDestruct (points_to_heap_agree with "Point Auth") as (st) "%Actual".
    iPureIntro. rewrite /heap_values lookup_fmap Actual in Wrong. naive_solver.
  Qed.
End resource_controls.

Lemma explicit_length_refuses_empty n : n ≠ 0%nat →
  ¬ (length (@nil val) = n).
Proof. simpl. lia. Qed.

(* Two one-step executions, identical return, different actual final memory. *)
Theorem same_return_different_memory (h : gmap loc val) (p : loc) (old a b : val) :
  h !! p = Some old → a ≠ b →
  rtc erased_step ([Write ScOrd #p (of_val a)], initialized_state h)
    ([of_val (LitV LitPoison)], stored_state h p a) ∧
  rtc erased_step ([Write ScOrd #p (of_val b)], initialized_state h)
    ([of_val (LitV LitPoison)], stored_state h p b) ∧
  heap_values (stored_state h p a) !! p ≠ heap_values (stored_state h p b) !! p.
Proof.
  intros Cell Different. split; first exact (store_execution h p old a Cell).
  split; first exact (store_execution h p old b Cell).
  rewrite !stored_state_lookup. congruence.
Qed.

Theorem value_only_does_not_imply_unchanged (h : gmap loc val) (p : loc) (old written : val) :
  h !! p = Some old → old ≠ written →
  adequate NotStuck (Write ScOrd #p (of_val written)) (initialized_state h)
    (λ w _, w = LitV LitPoison) ∧
  ¬ adequate NotStuck (Write ScOrd #p (of_val written)) (initialized_state h)
    (λ w σ, w = LitV LitPoison ∧ heap_values σ !! p = Some old).
Proof.
  intros Cell Different. split.
  - apply (initialized_heap_adequacy (Σ:=lrustΣ) _ _ _ 1).
    intros ?. iIntros "F Credit _". rewrite /initial_fragments.
    iDestruct (big_sepM_lookup with "F") as "Point"; first done.
    iApply (wp_write_sc ⊤ p (of_val written) written old with "[Point Credit]"); [done|by iFrame|].
    iNext. iIntros "_". done.
  - intros Adeq.
    have Exec := store_execution h p old written Cell.
    have Result := adequate_result _ _ _ _ Adeq [] (stored_state h p written) (LitV LitPoison) Exec.
    destruct Result as [_ Same]. rewrite stored_state_lookup in Same. naive_solver.
Qed.
