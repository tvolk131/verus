From ModelInputContext Require Export composition.
From lrust.typing Require Import soundness inv_context.
From lrust.lifetime Require Import lifetime_full.
Set Default Proof Using "Type".
Open Scope Z_scope.

Theorem raw_model_read_adequacy h p (zs : list Z) (z : Z) :
  Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  adequate NotStuck (Skip;; Read ScOrd #p) (initialized_state h) (λ w _, w = #z).
Proof.
  intros Range Cells Head.
  apply (raw_model_context_adequacy (Σ:=typeΣ) (Read ScOrd #p) h p zs _ 2);
    [solve_closed|done|done|].
  intros ? tid κ Nonempty.
  iIntros "LFT PROPH UNIQ TIME E L _ Ref Frame Credits".
  iApply (wp_wand with "[LFT PROPH UNIQ TIME E L Ref Frame Credits]").
  { iApply (initialized_reference_read_resolve with "LFT PROPH UNIQ TIME E L Ref Frame Credits"); done. }
  iIntros (w) "[%Eq _]". done.
Qed.

(* Explicit original lengths, no constraints on offset, block, other cells or bytes. *)
Corollary raw_model_read10 h p (zs : list Z) (z : Z) :
  length zs = 10%nat →
  Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  adequate NotStuck (Skip;; Read ScOrd #p) (initialized_state h) (λ w _, w = #z).
Proof. intros _ Range Cells Head. by apply (raw_model_read_adequacy _ _ zs). Qed.

Corollary raw_model_read128 h p (zs : list Z) (z : Z) :
  length zs = 128%nat →
  Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  adequate NotStuck (Skip;; Read ScOrd #p) (initialized_state h) (λ w _, w = #z).
Proof. intros _ Range Cells Head. by apply (raw_model_read_adequacy _ _ zs). Qed.

Lemma strict_interior_byte_lookup base (prefix suffix : list val) (zs : list Z) f :
  interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ##ₘ f →
  ∀ (i : nat) (z : Z), zs !! i = Some z →
    (interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ∪ f)
      !! ((base +ₗ length prefix) +ₗ i) = Some #z.
Proof.
  intros Disjoint i z Hz. rewrite shift_loc_assoc.
  apply lookup_union_Some_l.
  replace (base +ₗ (length prefix + i)) with (base +ₗ (length prefix + i)%nat)
    by (rewrite Nat2Z.inj_add; done).
  rewrite interval_lookup lookup_app_r; last lia.
  replace (length prefix + i - length prefix)%nat with i by lia. apply lookup_app_l_Some.
  by apply list_lookup_fmap_Some; exists z.
Qed.

Theorem strict_interior_model_read base (prefix suffix : list val) (zs : list Z) f (z : Z) :
  prefix ≠ [] → suffix ≠ [] →
  interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ##ₘ f →
  Forall (λ z, 0 ≤ z < 256) zs → zs !! 0%nat = Some z →
  adequate NotStuck (Skip;; Read ScOrd #(base +ₗ length prefix))
    (initialized_state
      (interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ∪ f))
    (λ w _, w = #z).
Proof.
  intros _ _ Disjoint Range Head.
  apply (raw_model_read_adequacy _ _ zs); [done| |done].
  by apply strict_interior_byte_lookup.
Qed.
