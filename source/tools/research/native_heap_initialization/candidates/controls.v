From NativeHeapInitialization Require Export partition.
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
  Lemma duplicate_frame_refused h a l v : a ⊆ h → a !! l = Some v →
    initial_fragments a -∗ initial_fragments (h ∖ a) -∗ l ↦ v -∗ False.
  Proof.
    iIntros (Ha Hl) "A F P".
    rewrite /initial_fragments.
    iDestruct (big_sepM_lookup with "A") as "Q"; first exact Hl.
    rewrite /heap_mapsto. iApply (duplicate_same_namespace with "P Q").
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
