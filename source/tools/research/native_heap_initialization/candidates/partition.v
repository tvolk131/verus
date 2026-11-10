From NativeHeapInitialization Require Export initialization.
Set Default Proof Using "Type".

(* Any finite exact chosen submap, with all unrelated cells retained. *)
Section partition.
  Context `{!heapGS Σ}.
  Lemma fragments_partition (a f : gmap loc val) :
    a ##ₘ f → initial_fragments (a ∪ f) ⊣⊢ initial_fragments a ∗ initial_fragments f.
  Proof. intros H. by rewrite /initial_fragments big_sepM_union. Qed.

  Lemma fragments_extract h a : a ⊆ h →
    initial_fragments h ⊣⊢ initial_fragments a ∗ initial_fragments (h ∖ a).
  Proof.
    intros Ha. rewrite -(map_difference_union _ _ Ha).
    apply fragments_partition. apply map_disjoint_difference_r. done.
  Qed.
End partition.

Fixpoint interval_map (l : loc) (vs : list val) : gmap loc val :=
  match vs with [] => ∅ | v :: vs => <[l:=v]>(interval_map (l +ₗ 1) vs) end.

Lemma interval_lookup l vs i v :
  interval_map l vs !! (l +ₗ i) = Some v ↔ vs !! i = Some v.
Proof.
  induction vs as [|w vs IH] in l, i |- *.
  - by rewrite /interval_map lookup_empty lookup_nil.
  - destruct i as [|i].
    + simpl. rewrite shift_loc_0 lookup_insert. naive_solver.
    + simpl. rewrite lookup_insert_ne.
      * replace (l +ₗ S i) with (l +ₗ 1 +ₗ i) by (rewrite shift_loc_assoc; f_equal; lia).
        apply IH.
      * intros H. destruct l as [b z]. inversion H. lia.
Qed.

Lemma interval_before l vs q : q.2 < l.2 → interval_map l vs !! q = None.
Proof.
  induction vs as [|v vs IH] in l |- *; first done.
  intros Hlt. simpl. rewrite lookup_insert_ne.
  - apply IH. simpl. lia.
  - intros ->. lia.
Qed.

Lemma interval_fresh l vs : interval_map (l +ₗ 1) vs !! l = None.
Proof. apply interval_before. simpl. lia. Qed.

Lemma interval_lookup_witness l vs q v :
  interval_map l vs !! q = Some v → ∃ i : nat, vs !! i = Some v ∧ q = l +ₗ i.
Proof.
  induction vs as [|w vs IH] in l |- *; first done.
  simpl. rewrite lookup_insert_Some. intros [[-> ->]|[_ H]].
  - exists 0%nat. by rewrite shift_loc_0.
  - destruct (IH _ H) as [i [Hi ->]]. exists (S i). split; first done.
    rewrite shift_loc_assoc. f_equal. lia.
Qed.

Lemma interval_submap h l vs :
  (∀ i v, vs !! i = Some v → h !! (l +ₗ i) = Some v) → interval_map l vs ⊆ h.
Proof.
  intros H. apply map_subseteq_spec. intros q v Hq.
  destruct (interval_lookup_witness _ _ _ _ Hq) as [i [Hi ->]]. by apply H.
Qed.

Section intervals.
  Context `{!heapGS Σ}.
  Lemma interval_fragments l vs : initial_fragments (interval_map l vs) ⊣⊢ l ↦∗ vs.
  Proof.
    induction vs as [|v vs IH] in l |- *.
    - by rewrite /initial_fragments /= big_sepM_empty heap_mapsto_vec_nil.
    - rewrite /initial_fragments /= big_sepM_insert; last apply interval_fresh.
      rewrite heap_mapsto_vec_cons. by rewrite -IH.
  Qed.

  Lemma interior_extract h l vs :
    (∀ i v, vs !! i = Some v → h !! (l +ₗ i) = Some v) →
    initial_fragments h ⊣⊢ l ↦∗ vs ∗ initial_fragments (h ∖ interval_map l vs).
  Proof.
    intros H. rewrite (fragments_extract h (interval_map l vs)); last by apply interval_submap.
    by rewrite interval_fragments.
  Qed.

  Lemma interior_v12_payload h l vs :
    (∀ i v, vs !! i = Some v → h !! (l +ₗ i) = Some v) →
    initial_fragments h ⊣⊢
      (l, repeat [] (length vs)) #↦!∗ (FVal <$> vs) ∗
      initial_fragments (h ∖ interval_map l vs).
  Proof.
    intros H. rewrite (interior_extract _ _ _ H).
    rewrite heap_cloc_mapsto_val_empty heap_complete_mapsto_fancy_fmap_eq. done.
  Qed.

  Lemma interior_split_recombine l prefix middle suffix :
    l ↦∗ (prefix ++ middle ++ suffix) ⊣⊢
      l ↦∗ prefix ∗ (l +ₗ length prefix) ↦∗ middle ∗
      (l +ₗ (length prefix + length middle)) ↦∗ suffix.
  Proof. rewrite !heap_mapsto_vec_app shift_loc_assoc Nat2Z.inj_add. by rewrite assoc. Qed.
End intervals.
