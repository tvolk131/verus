From ModelPairInput Require Export setup.
From lrust.typing Require Import type_context programs inv_context soundness.
From lrust.lang Require Import proofmode notation lifting.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Disjointness is exactly about the accessed cells, without a block or
   base-address restriction. In particular it is vacuous for either empty list. *)
Lemma byte_map_disjoint_iff p pad q key :
  byte_map p pad ##ₘ byte_map q key ↔
  ∀ (i j : nat), (i < length pad)%nat → (j < length key)%nat → p +ₗ i ≠ q +ₗ j.
Proof.
  split.
  - intros Disjoint i j Hi Hj Eq.
    destruct (lookup_lt_is_Some_2 pad i Hi) as [z Hz].
    destruct (lookup_lt_is_Some_2 key j Hj) as [w Hw].
    have P : byte_map p pad !! (p +ₗ i) = Some #z.
    { apply interval_lookup. by apply list_lookup_fmap_Some; exists z. }
    have Q : byte_map q key !! (q +ₗ j) = Some #w.
    { apply interval_lookup. by apply list_lookup_fmap_Some; exists w. }
    rewrite Eq in P.
    have NoneP := map_disjoint_Some_r _ _ _ _ Disjoint Q.
    congruence.
  - intros Separate. apply map_disjoint_spec. intros l v w P Q.
    destruct (interval_lookup_witness _ _ _ _ P) as [i [Hi Ei]].
    destruct (interval_lookup_witness _ _ _ _ Q) as [j [Hj Ej]].
    apply (Separate i j); [| |congruence].
    + apply lookup_lt_Some in Hi. by rewrite length_fmap in Hi.
    + apply lookup_lt_Some in Hj. by rewrite length_fmap in Hj.
Qed.

Lemma byte_map_empty_alias p pad : byte_map p pad ##ₘ byte_map p [].
Proof. apply byte_map_disjoint_iff. simpl. lia. Qed.

Lemma byte_map_same_block_gap (base : loc) pad key gap :
  (0 ≤ gap)%Z →
  byte_map (base +ₗ 1) pad ##ₘ
    byte_map (base +ₗ (1 + length pad + gap)) key.
Proof.
  intros Gap. apply byte_map_disjoint_iff.
  intros i j Hi Hj Eq. destruct base as [block off]. inversion Eq. simpl in *. lia.
Qed.

(* This duplicates a shared view, never a lifetime handle or exclusive payload. *)
Section aliases.
  Context `{!typeG Σ}.
  Lemma shared_key_aliases key q tid κ :
    shared_key_view key q tid κ ⊣⊢
    shared_key_view key q tid κ ∗ shared_key_view key q tid κ.
  Proof.
    rewrite !shared_key_view_own. iSplit.
    - iDestruct 1 as (d) "[#T #Own]". iSplit; iExists d; iFrame "T Own".
    - iIntros "[$ _]".
  Qed.
End aliases.

(* Execute all setup steps and observe the exact outside frame from its actual
   returned resources. The two input intervals remain borrowed and are not
   claimed as physical observations in this control. *)
Theorem pair_setup_outside_observed h p pad q key :
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
  byte_map p pad ##ₘ byte_map q key →
  adequate NotStuck (pair_setup #0) (initialized_state h)
    (λ v σ, v = #0 ∧ frame_observed σ (pair_outside h p pad q key)).
Proof.
  intros PadRange KeyRange PadCells KeyCells Disjoint.
  apply (raw_pair_model_observed (Σ:=typeΣ) #0 h p pad q key
    (λ Outer v, (⌜v = #0⌝ ∗ @initial_fragments typeΣ
      (@lrustGS_gen_heapGS typeΣ Outer) (pair_outside h p pad q key))%I) _ 0%nat); try done.
  - intros Outer TG tid κpad κkey SameOuter PadNonempty KeyNonempty. subst Outer.
    iIntros "_ _ _ _ _ _ _ _ _ _ Frame _".
    iApply wp_value. iFrame "Frame". done.
  - intros Outer σ v. iIntros "Heap [%Value Frame]".
    iDestruct (fragments_observe with "Heap Frame") as "%Outside".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done. done.
Qed.

Theorem empty_key_same_address_setup_observed h p pad :
  Forall (λ z, 0 ≤ z < 256) pad →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  adequate NotStuck (pair_setup #0) (initialized_state h)
    (λ v σ, v = #0 ∧ frame_observed σ (pair_outside h p pad p [])).
Proof.
  intros Range Cells. apply pair_setup_outside_observed; try done.
  apply byte_map_empty_alias.
Qed.

From ModelInputContext Require Import instances.

Theorem strict_interior_pair_setup_observed base (prefix middle suffix : list val)
    pad key f :
  prefix ≠ [] → middle ≠ [] → suffix ≠ [] →
  interval_map base (prefix ++ byte_values pad ++ middle ++ byte_values key ++ suffix) ##ₘ f →
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  let h := interval_map base (prefix ++ byte_values pad ++ middle ++ byte_values key ++ suffix) ∪ f in
  let p := base +ₗ length prefix in
  let q := base +ₗ length (prefix ++ byte_values pad ++ middle) in
  adequate NotStuck (pair_setup #0) (initialized_state h)
    (λ v σ, v = #0 ∧ frame_observed σ (pair_outside h p pad q key)).
Proof.
  intros _ _ _ Disjoint PadRange KeyRange. simpl.
  apply pair_setup_outside_observed; try done.
  - apply (strict_interior_byte_lookup base prefix (middle ++ byte_values key ++ suffix) pad f).
    exact Disjoint.
  - have Disjoint' : interval_map base
      ((prefix ++ byte_values pad ++ middle) ++ byte_values key ++ suffix) ##ₘ f.
    { by rewrite -!app_assoc. }
    have Lookup := strict_interior_byte_lookup base
      (prefix ++ byte_values pad ++ middle) suffix key f Disjoint'.
    by rewrite -!app_assoc in Lookup.
  - apply byte_map_disjoint_iff. intros i j Hi Hj Eq.
    rewrite !length_app /byte_values !length_fmap in Eq.
    destruct base as [block off]. inversion Eq. simpl in *. lia.
Qed.
