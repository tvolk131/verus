From NativeByteXor Require Export native_xor.
From ModelByteXor Require Import byte_xor.
From ModelBufferTransform Require Import transform.
From lrust.lang Require Import proofmode notation lifting heap.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition silent_step (s s' : expr * state) : Prop :=
  @language.prim_step lrust_lang s.1 s.2 [] s'.1 s'.2 [].

Lemma sc_read_one_step σ p r v :
  σ.1 !! p = Some (RSt r,v) →
  nsteps silent_step 1 (Read ScOrd (Lit (LitLoc p)),σ) (of_val v,σ).
Proof.
  intros Lookup. econstructor; last constructor.
  apply base_prim_step. eapply ReadScS. exact Lookup.
Qed.

Lemma na_read_two_steps σ p r v :
  σ.1 !! p = Some (RSt r,v) →
  nsteps silent_step 2 (Read Na1Ord (Lit (LitLoc p)),σ) (of_val v,σ).
Proof.
  destruct σ as [h f]. simpl. intros Lookup.
  econstructor.
  - apply base_prim_step. eapply ReadNa1S. exact Lookup.
  - econstructor.
    + apply base_prim_step. eapply ReadNa2S. by rewrite /= lookup_insert.
    + rewrite /= insert_insert (insert_id _ _ _ Lookup). constructor.
Qed.

Lemma sc_write_one_step σ p old v :
  σ.1 !! p = Some (RSt 0,old) →
  nsteps silent_step 1
    (Write ScOrd (Lit (LitLoc p)) (of_val v),σ)
    (Lit LitPoison,(<[p:=(RSt 0,v)]>σ.1,σ.2)).
Proof.
  intros Lookup. econstructor; last constructor.
  apply base_prim_step. apply (WriteScS _ _ v old); [done|done].
Qed.

Lemma na_write_two_steps σ p old v :
  σ.1 !! p = Some (RSt 0,old) →
  nsteps silent_step 2
    (Write Na1Ord (Lit (LitLoc p)) (of_val v),σ)
    (Lit LitPoison,(<[p:=(RSt 0,v)]>σ.1,σ.2)).
Proof.
  destruct σ as [h f]. simpl. intros Lookup.
  econstructor.
  - apply base_prim_step. apply (WriteNa1S _ _ v old); [done|done].
  - econstructor.
    + apply base_prim_step. apply (WriteNa2S _ _ v old); [done|].
      by rewrite /= lookup_insert.
    + by rewrite /= insert_insert; constructor.
Qed.

Lemma na_read_phase_changes σ p r v :
  σ.1 !! p = Some (RSt r,v) →
  (<[p:=(RSt (S r),v)]>σ.1,σ.2) ≠ σ.
Proof.
  intros Lookup Eq.
  have At := f_equal (λ s : state, s.1 !! p) Eq.
  rewrite /= lookup_insert Lookup in At. injection At. lia.
Qed.

Lemma na_write_readers_blocked σ p r old v e' σ' efs :
  σ.1 !! p = Some (RSt (S r),old) →
  ¬ head_step (Write Na1Ord (Lit (LitLoc p)) (of_val v)) σ [] e' σ' efs.
Proof. intros Lookup Step. inversion Step; subst; congruence. Qed.

Definition cell_assignment_na (p : loc) (b : Z) : expr :=
  (let: "a" := !(Lit (LitLoc p)) in
   let: "c" := (of_val byte_xor) ["a"; Lit (LitInt b)] in
   Lit (LitLoc p) <- "c")%E.

Section ordinary_cell.
  Context `{!lrustGS Σ}.
  Lemma wp_cell_assignment_na p a b (R : iProp Σ) : byte_value a → byte_value b →
    {{{ p ↦ #a ∗ R }}}
      cell_assignment_na p b
    {{{ RET LitV LitPoison; p ↦ #(scalar_xor8 a b) ∗ R ∗
        ⌜byte_value (scalar_xor8 a b)⌝ }}}.
  Proof.
    intros RA RB. iIntros (Φ) "[Cell Frame] Post".
    rewrite /cell_assignment_na.
    remember byte_xor as op eqn:OpEq.
    wp_bind (Read Na1Ord (Lit (LitLoc p))).
    iApply (wp_read_na with "Cell"); first done.
    iNext. iIntros "Cell". wp_let.
    wp_bind (App (of_val op) [Lit (LitInt a); Lit (LitInt b)]).
    iApply (wp_wand with "[]").
    { rewrite OpEq. iApply (wp_scalar_xor8 a b RA RB). }
    iIntros (w) "%Eq". subst w. wp_let.
    iApply (wp_write_na ⊤ p _ _ _ with "Cell"); first done.
    iNext. iIntros "Cell". iApply "Post". iFrame.
    iPureIntro. apply scalar_xor8_range.
  Qed.
End ordinary_cell.
