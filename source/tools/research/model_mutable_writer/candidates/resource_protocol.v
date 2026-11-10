From ModelNestedBorrowContext Require Export protocol.
From NativeSemanticBridge Require Import borrow_handoff.
From lrust.typing Require Import soundness inv_context programs_util borrow uniq_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.
Section resources.
  Context `{!typeG Σ}.
  Lemma shorter_run_resources {𝔄} E L I tid iκs p (ty : type 𝔄) κ m (x_final : ~~𝔄) e (Rin Rout Frame : iProp Σ) :
    lctx_lft_alive E L κ → lctx_lft_alive E L (ty_lft (&uniq{κ} ty)%T) →
    Closed [] e →
    (∀ α c, uniq_bor_loc c = uniq_bor_loc m → uniq_bor_current c = uniq_bor_current m →
      llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
      llctx_interp (α ⊑ₗ [κ] :: L) -∗ invctx_interp tid ⊤ iκs I -∗
      tctx_elt_interp tid (p ◁ &uniq{α} ty) c -∗ Rin -∗
      WP e {{ _, Rout ∗ ∃ c', llctx_interp (α ⊑ₗ [κ] :: L) ∗ invctx_interp tid ⊤ iκs I ∗
        tctx_elt_interp tid (p ◁ &uniq{α} ty) c' ∗
        ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
             vπ (uniq_bor_current c') π = vπ x_final π⟩ }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
    tctx_elt_interp tid (p ◁ &uniq{κ} ty) m -∗ Rin -∗ Frame -∗
    WP (Newlft;; ReBor;; e;; Endlft;; Skip) {{ _,
      Frame ∗ Rout ∗ llctx_interp L ∗ invctx_interp tid ⊤ iκs I ∗
      ∃ m', tctx_elt_interp tid (p ◁ &uniq{κ} ty) m' ∗
      ⟨π, vπ m' π = (vπ x_final π, uniq_bor_future m π)⟩ }}.
  Proof.
    iIntros (Alive ParentAlive ClosedE RunChild) "#LFT #TIME #PROPH #UNIQ #E L I Parent Rin Frame".
    iMod (llftl_begin' with "LFT") as (Λ) "[Λ #Nonempty]"; first done.
    set α := lft_intersect_list [κ] ⊓ Λ.
    iAssert (llctx_interp (α ⊑ₗ [κ] :: L)) with "[Λ L]" as "L".
    { rewrite /llctx_interp. iSplitL "Λ"; last done. iExists Λ. iFrame "Λ". by iSplit. }
    wp_seq. wp_bind ReBor.
    iApply (wp_wand with "[L I Parent]").
    { iApply (tctx_reborrow_uniq E (α ⊑ₗ [κ] :: L) I p ty κ α (child_inclusion E L κ α)
        tid (λ xl _ π, let '(c -:: b -:: _) := xl in
          uniq_bor_loc c = uniq_bor_loc m ∧ uniq_bor_current c = uniq_bor_current m ∧
          blockedπ b π = (uniq_bor_future c π, uniq_bor_future m π)) ⊤ iκs -[m]
        with "LFT TIME PROPH UNIQ E L I [Parent] []").
      - simpl. by iFrame.
      - iApply proph_obs_true. intros π c b Loc Cur Block. by repeat split. }
    iIntros (v) "[%xl [L [I [T #Obs]]]]". destruct xl as [c [b []]].
    wp_seq.
    iMod (proph_obs_sat with "PROPH Obs") as %[π0 [Loc [Cur Block0]]]; first solve_ndisj.
    simpl in Loc, Cur, Block0.
    iDestruct "T" as "[Child [Blocked _]]".
    wp_bind e.
    iApply (wp_wand with "[L I Child Rin]").
    { iApply (RunChild α c Loc Cur with "LFT TIME PROPH UNIQ E L I Child Rin"). }
    iIntros (w) "[Rout [%c' [L [I [Child #Updated]]]]]".
    wp_seq.
    iApply (end_child_context E L α [κ] tid
      +[p ◁ &uniq{α} ty; p ◁{α} blocked_type_ctor _ (&uniq{κ} ty)]
      +[p ◁{α} blocked_type_ctor _ (&uniq{κ} ty)] +[p ◁ &uniq{κ} ty]
      -[c';b] _ _ (λ xl _ π, let '(b -:: _) := xl in
        blockedπ b π = (vπ x_final π, uniq_bor_future m π))
      Skip _ with "LFT TIME PROPH UNIQ E L [Child Blocked] [] [Frame Rout I]").
    - eapply resolve_unblock_tctx_cons_resolve.
      + simpl. by left.
      + apply resolve'_post. apply (source_loan_release _ _ _ _ (child_alive E L κ α Alive)).
      + apply resolve_unblock_tctx_cons_keep. apply resolve_unblock_tctx_nil.
    - apply unblock_tctx_cons_unblock; [done|apply unblock_tctx_nil].
    - simpl. iFrame.
    - iCombine "Obs Updated" as "Both".
      iApply (proph_obs_impl with "Both").
      move=> π [[Loc' [Cur' Block']] [Future Current]] /= Resolved.
      by rewrite Block' -Future Resolved Current.
    - iIntros (yl) "L T #Recovered". destruct yl as [m' []].
      wp_seq. iFrame "Frame Rout L I". iExists m'.
      simpl. iDestruct "T" as "[$ _]".
      iApply (proph_obs_impl with "Recovered"). intros π [[b' []] [Block [Eq _]]].
      simpl in *. by rewrite -Eq Block.
  Qed.
End resources.
