From NativeHeapInitialization Require Export partition.
From lrust.util Require Import non_atomic_cell_map cellmap.
From lrust.lang Require Import notation.
Set Default Proof Using "Type".

Section controls.
  Context `{!invGS Σ, !na_logicG loc val Σ}.
  Lemma wrong_initial_value ι h l v w : h !! l = Some v → v ≠ w →
    non_atomic_map ι (initialized_map h) -∗ (inl l @↦[ι] inr w) -∗ False.
  Proof.
    iIntros (Hv Hne) "H P".
    iDestruct (points_to_heap_agree with "P H") as (st) "%Hst".
    rewrite /initialized_map lookup_fmap Hv in Hst. naive_solver.
  Qed.
  Lemma wrong_initial_location ι h l v : h !! l = None →
    non_atomic_map ι (initialized_map h) -∗ (inl l @↦[ι] inr v) -∗ False.
  Proof.
    iIntros (Hl) "H P".
    iDestruct (points_to_heap_agree with "P H") as (st) "%Hst".
    rewrite /initialized_map lookup_fmap Hl in Hst. done.
  Qed.
  Lemma duplicate_same_namespace ι l v w :
    (inl l @↦[ι] inr v) -∗ (inl l @↦[ι] inr w) -∗ False.
  Proof.
    rewrite non_atomic_cell_map.points_to_seq_unseal.
    iIntros "P Q". iDestruct (cmap_locs_ne with "P Q") as "%Hne". done.
  Qed.
End controls.

Section frame_control.
  Context `{!heapGS Σ, !invGS Σ, !na_invG Σ, !alc_logicG Σ}.
  Lemma overlapping_frame_refused (a f : gmap loc val) l v w :
    a !! l = Some v → f !! l = Some w →
    initial_fragments a -∗ initial_fragments f -∗ False.
  Proof.
    iIntros (Ha Hf) "A F".
    rewrite /initial_fragments.
    iDestruct (big_sepM_lookup with "A") as "P"; first exact Ha.
    iDestruct (big_sepM_lookup with "F") as "Q"; first exact Hf.
    rewrite /heap_mapsto. iApply (duplicate_same_namespace with "P Q").
  Qed.
  Lemma wrong_initial_frame (h f : gmap loc val) l v w :
    h !! l = Some v → f !! l = Some w → v ≠ w →
    non_atomic_map heap_name (initialized_map h) -∗ initial_fragments f -∗ False.
  Proof.
    iIntros (Hv Hw Hne) "H F". rewrite /initial_fragments.
    iDestruct (big_sepM_lookup with "F") as "P"; first exact Hw.
    rewrite /heap_mapsto. iApply (wrong_initial_value with "H P"); done.
  Qed.
End frame_control.

(* Checked concrete strict interior, arbitrary block/offset and unrelated frame.
   Incorrect bytes, shifted absent location and overlapping frame are refuted by
   the general resource countertheorems above, not by unsuccessful tactics. *)
Lemma interval_retains_frame l vs f : interval_map l vs ##ₘ f →
  (interval_map l vs ∪ f) ∖ interval_map l vs = f.
Proof.
  intros H. apply map_eq. intros q.
  rewrite lookup_difference lookup_union.
  destruct (interval_map l vs !! q) as [v|] eqn:Hv.
  - have Hf : f !! q = None by eapply map_disjoint_Some_l.
    by rewrite Hf.
  - by destruct (f !! q).
Qed.
