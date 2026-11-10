From ModelMutableWriter Require Export initialized.
From lrust.typing Require Import soundness inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section physical_transformation.
  Context `{!typeG Σ}.

  Lemma initialized_nested_transform zs p tid κ
      (x1 : ~~(bytes_sort (length zs))) e (Rin Rout Frame : iProp Σ) :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) (byte_list (length zs) x1) →
    (Forall (λ z, 0 ≤ z < 256) zs →
      p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> zs) -∗ Rin -∗
      WP e {{ _, p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) ∗ Rout }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ £1 -∗ Rin -∗ Frame -∗
    WP ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft) {{ _,
      Frame ∗ Rout ∗ p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) }}.
  Proof.
    iIntros (ClosedE RangeNew Transform) "#LFT #TIME #PROPH #UNIQ #E L I Ref Open Rin Frame".
    unshelve iApply (initialized_nested_payload_run zs p tid κ x1 e (£1 ∗ Rin)%I Rout Frame
      ClosedE _ with "LFT TIME PROPH UNIQ E L I Ref [Open Rin] Frame").
    intros α c Loc Cur.
    iIntros "#LFT' #TIME' #PROPH' #UNIQ' #E' L I Child [Open Rin]".
    iApply (wp_wand with "[L Child Open Rin]").
    { unshelve iApply (byte_child_transform (length zs) [] (α ⊑ₗ [κ] :: [κ ⊑ₗ []])
        α p (bytes_of_list zs) x1 c tid e Rin Rout
        (child_alive [] [κ ⊑ₗ []] κ α (fresh_local_alive κ))
        Loc Cur RangeNew _ with "LFT' TIME' UNIQ' E' L Child Open Rin").
      intros RangeOld. rewrite byte_list_of_list in RangeOld |- *.
      exact (Transform RangeOld). }
    iIntros (w) "[Rout [%c' [L [Child #Updated]]]]".
    iFrame "Rout". iExists c'. iFrame "L I Child Updated".
    Unshelve.
    iFrame "Open Rin".
  Qed.
End physical_transformation.
