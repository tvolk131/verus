From ModelMutableWriter Require Export transform.
From ModelInputObservation Require Import setup.
From lrust.typing Require Import soundness inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section public_model_transformation.
  Context {Σ : gFunctors} `{!typePreG Σ}.

  Theorem raw_model_transform_observed h p zs
      (x1 : ~~(bytes_sort (length zs))) e k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) zs →
    Forall (λ z, 0 ≤ z < 256) (byte_list (length zs) x1) →
    (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (TG : typeG Σ),
      Forall (λ z, 0 ≤ z < 256) zs →
      p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> zs) -∗ £k -∗
      WP e {{ _, p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) }}) →
    adequate NotStuck
      (Skip;; ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft))%E
      (initialized_state h)
      (λ _ σ, ∃ vs, length vs = length zs ∧
        vs = ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) ∧
        interval_observed σ p vs ∧
        frame_observed σ (h ∖ interval_map p ((λ z : Z, LitV (LitInt z)) <$> zs))).
  Proof using All.
    intros ClosedE RangeOld RangeNew Cells Transform.
    apply (raw_model_context_observed (Σ:=Σ)
      ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft)%E
      h p zs (length zs)
      (λ _ vs, vs = ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1)) (1+k));
      [solve_closed|done|done|].
    intros TG tid κ Nonempty.
    iIntros "#LFT #PROPH #UNIQ #TIME #E L I Ref Frame [Open Credits]".
    iApply (wp_wand with "[L I Ref Frame Open Credits]").
    { unshelve iApply (initialized_nested_transform zs p tid κ x1 e (£k)%I emp%I
        (initial_fragments (h ∖ interval_map p ((λ z : Z, LitV (LitInt z)) <$> zs)))
        ClosedE RangeNew _ with "LFT TIME PROPH UNIQ E L I Ref Open Credits Frame").
      intros RangeInput. iIntros "P Credits".
      iApply (wp_wand with "[P Credits]").
      { iApply (Transform TG RangeInput with "P Credits"). }
      iIntros (w) "P". by iFrame. }
    iIntros (w) "[Frame [_ P]]".
    iExists ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1).
    iFrame "P Frame". iPureIntro. split; last done.
    rewrite length_fmap. apply byte_list_length.
  Qed.
End public_model_transformation.
