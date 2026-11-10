From ModelStateObservation Require Export observation.
From lrust.lang Require Import proofmode notation lifting.
Set Default Proof Using "Type".

(* Every list, including arbitrary mathematical byte values at lengths 10/128.
   Only original raw-map lookups are supplied; the initializer supplies resources. *)
Theorem read_interval_final h p first tail :
  (∀ (i : nat) v, (first :: tail) !! i = Some v → h !! (p +ₗ i) = Some v) →
  adequate NotStuck (Read ScOrd #p) (initialized_state h)
    (λ w σ, ∃ vs, length vs = length (first :: tail) ∧
      (w = first ∧ vs = first :: tail) ∧ interval_observed σ p vs ∧
      frame_observed σ (h ∖ interval_map p (first :: tail))).
Proof.
  intros Cells. apply (initialized_interior_observation (Σ:=lrustΣ) _ _ _ _ _ _ 0 Cells).
  intros ?. rewrite heap_mapsto_vec_cons. iIntros "[Head Tail] Frame _ _".
  iApply (wp_read_sc with "Head"); first done.
  iNext. iIntros "Head". iExists (first :: tail).
  iSplit; first done. rewrite heap_mapsto_vec_cons. iFrame.
Qed.

Theorem write_interval_final h p first tail (written : val) :
  (∀ (i : nat) v, (first :: tail) !! i = Some v → h !! (p +ₗ i) = Some v) →
  adequate NotStuck (Write ScOrd #p (of_val written)) (initialized_state h)
    (λ w σ, ∃ vs, length vs = length (first :: tail) ∧
      (w = LitV LitPoison ∧ vs = written :: tail) ∧ interval_observed σ p vs ∧
      frame_observed σ (h ∖ interval_map p (first :: tail))).
Proof.
  intros Cells. apply (initialized_interior_observation (Σ:=lrustΣ) _ _ _ _ _ _ 1 Cells).
  intros ?. rewrite heap_mapsto_vec_cons. iIntros "[Head Tail] Frame Credit _".
  iApply (wp_write_sc ⊤ p (of_val written) written first with "[Head Credit]"); [done|by iFrame|].
  iNext. iIntros "Head". iExists (written :: tail).
  iSplit; first by simpl. rewrite heap_mapsto_vec_cons. iFrame.
Qed.

Theorem write_read_interval_final h p first tail (written : val) :
  (∀ (i : nat) v, (first :: tail) !! i = Some v → h !! (p +ₗ i) = Some v) →
  adequate NotStuck (Write ScOrd #p (of_val written);; Read ScOrd #p) (initialized_state h)
    (λ w σ, ∃ vs, length vs = length (first :: tail) ∧
      (w = written ∧ vs = written :: tail) ∧ interval_observed σ p vs ∧
      frame_observed σ (h ∖ interval_map p (first :: tail))).
Proof.
  intros Cells. apply (initialized_interior_observation (Σ:=lrustΣ) _ _ _ _ _ _ 1 Cells).
  intros ?. rewrite heap_mapsto_vec_cons. iIntros "[Head Tail] Frame Credit _".
  wp_bind (Write _ _ _).
  iApply (wp_write_sc ⊤ p (of_val written) written first with "[Head Credit]"); [done|by iFrame|].
  iNext. iIntros "Head". wp_pure (Let _ _ _).
  iApply (wp_read_sc with "Head"); first done.
  iNext. iIntros "Head". iExists (written :: tail).
  iSplit; first by simpl. rewrite heap_mapsto_vec_cons. iFrame.
Qed.

(* No execution assumption about all threads is hidden in these diagnostics. *)
Lemma head_return_execution e σ v σ' :
  head_step e σ [] (of_val v) σ' [] →
  rtc erased_step ([e], σ) ([of_val v], σ').
Proof.
  intros Step. apply rtc_once. exists [].
  eapply (step_atomic _ _ _ _ [] [] []); [done|done|].
  by apply base_prim_step.
Qed.

Definition stored_state (h : gmap loc val) (p : loc) (v : val) : state :=
  (<[p := (RSt 0,v)]> (initialized_map h), false).

Lemma store_execution (h : gmap loc val) (p : loc) (old written : val) : h !! p = Some old →
  rtc erased_step ([Write ScOrd #p (of_val written)], initialized_state h)
    ([of_val (LitV LitPoison)], stored_state h p written).
Proof.
  intros Cell. apply head_return_execution.
  eapply WriteScS; first by rewrite to_of_val.
  by apply initialized_lookup.
Qed.

Lemma stored_state_lookup h p v : heap_values (stored_state h p v) !! p = Some v.
Proof. by rewrite /heap_values /stored_state /= lookup_fmap lookup_insert. Qed.
