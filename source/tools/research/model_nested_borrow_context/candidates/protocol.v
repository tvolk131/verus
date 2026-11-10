From ModelInputContext Require Export context.
From NativeSemanticBridge Require Import borrow_handoff.
From lrust.typing Require Import borrow programs_util soundness.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".

Section protocol.
  Context `{!typeG Σ}.

  Lemma child_inclusion E L κ α :
    lctx_lft_incl E (α ⊑ₗ [κ] :: L) α κ.
  Proof. eapply lctx_lft_incl_local; by left. Qed.

  Lemma child_alive E L κ α :
    lctx_lft_alive E L κ → lctx_lft_alive E (α ⊑ₗ [κ] :: L) α.
  Proof.
    intros Alive. apply (lctx_lft_alive_local _ _ α [κ]); first by left.
    constructor; last constructor. unfold lctx_lft_alive in *.
    iIntros "#L #E". iApply (Alive with "[L] E").
    rewrite /llctx_interp. by iDestruct "L" as "[_ $]".
  Qed.

  (* WP form of the existing type_endlft rule, with its exact three Skip
     expression and its two receipt-indexed step-fupd eliminations. *)
  Lemma end_child_context {𝔄l 𝔅l} E L κ κl tid
      (T : tctx 𝔄l) (T' T'' : tctx 𝔅l) xl tru f post e Φ :
    Closed [] e →
    resolve_unblock_tctx E (κ ⊑ₗ κl :: L) κ T T' tru →
    unblock_tctx E L κ T' T'' f →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp (κ ⊑ₗ κl :: L) -∗ tctx_interp tid T xl -∗
    ⟨π, tru post xl ⊤ π⟩ -∗
    (∀ yl, llctx_interp L -∗ tctx_interp tid T'' yl -∗
      ⟨π, ∃ xl', post xl' ⊤ π ∧ f π xl' yl⟩ -∗ WP e {{ Φ }}) -∗
    WP (Endlft;; e) {{ Φ }}.
  Proof.
    iIntros (ClosedE RslvU Un) "#LFT #TIME #PROPH #UNIQ #E L' T Obs Continue".
    wp_bind Skip.
    iMod (llctx_interp_make_guarded with "L'") as (γ) "[H1 [H2 [#Ghalf #Halfback]]]"; first solve_ndisj.
    iMod (RslvU with "LFT PROPH UNIQ TIME E Ghalf H1 T Obs") as (d xl') "[⧖ ToT']".
    iApply (wp_step_fupdN_persistent_time_receipt' _ _ ∅ with "TIME ⧖ [ToT']")=>//.
    { iApply step_fupdN_with_emp.
      iDestruct (step_fupdN_nmono with "ToT'") as "ToT'";
        last by rewrite difference_empty_L. nia. }
    wp_seq. iIntros "(H1 & T' & Obs) !>".
    iDestruct ("Halfback" with "H1 H2") as "L'".
    iMod (fupd_mask_mono with "L'") as "L'"; first solve_ndisj.
    iDestruct "L'" as "[(%&%& κ' & %ToDead) L]".
    iDestruct (llftl_end' with "LFT κ'") as "To†"; first apply ToDead.
    wp_seq. wp_bind Skip.
    iApply wp_mask_mono; [|iApply (wp_step_fupd with "To†")]; [set_solver..|].
    wp_seq. iIntros "† !>". wp_seq. wp_bind Skip.
    iMod (llctx_interp_make_guarded with "L") as (γ1) "[H1 [H2 [#Ghalf1 #Halfback1]]]"; first solve_ndisj.
    iMod (Un with "LFT TIME E Ghalf1 H1 [†] T'") as (d0 yl) "[⧖ ToT']".
    { simpl in *. subst. rewrite llftl_end_inter. by iRight. }
    iApply (wp_step_fupdN_persistent_time_receipt' _ _ ∅ with "TIME ⧖ [ToT']")=>//.
    { iApply step_fupdN_with_emp.
      iDestruct (step_fupdN_nmono _ (S d0 * S d0) with "ToT'") as "ToT'"; first nia.
      rewrite <- step_fupdN_addS. by rewrite difference_empty_L. }
    wp_seq. iIntros "(H1 & T'' & Obs') !>". wp_seq.
    iDestruct ("Halfback1" with "H1 H2") as "L".
    iMod (fupd_mask_mono with "L") as "L"; first solve_ndisj.
    iApply ("Continue" with "L T'' [Obs Obs']").
    iCombine "Obs Obs'" as "Obs". iApply (proph_obs_impl with "Obs").
    intros π [Ha Hf]. by exists xl'.
  Qed.

  (* The input ownership is explicitly MODEL-relative. Frame can contain an
     ancestor restoration continuation and any surrounding typing context. *)
  Lemma shorter_no_write {𝔄} E L I tid iκs p (ty : type 𝔄) κ m (Frame : iProp Σ) :
    lctx_lft_alive E L κ → lctx_lft_alive E L (ty_lft (&uniq{κ} ty)) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
    tctx_elt_interp tid (p ◁ &uniq{κ} ty) m -∗ Frame -∗
    WP (Newlft;; ReBor;; Endlft;; Skip) {{ _,
      Frame ∗ llctx_interp L ∗ invctx_interp tid ⊤ iκs I ∗
      ∃ m', tctx_elt_interp tid (p ◁ &uniq{κ} ty) m' ∗
      ⟨π, vπ m' π = (vπ (uniq_bor_current m) π, uniq_bor_future m π)⟩ }}.
  Proof.
    iIntros (Alive ParentAlive) "#LFT #TIME #PROPH #UNIQ #E L I Parent Frame".
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
      - rewrite /tctx_interp /=. by iFrame.
      - iApply proph_obs_true. intros π c b Loc Cur Block. by repeat split. }
    iIntros (v) "[%xl [L [I [T #Obs]]]]". destruct xl as [c [b []]].
    wp_seq.
    iApply (end_child_context E L α [κ] tid
      +[p ◁ &uniq{α} ty; p ◁{α} blocked_type_ctor _ (&uniq{κ} ty)]
      +[p ◁{α} blocked_type_ctor _ (&uniq{κ} ty)] +[p ◁ &uniq{κ} ty]
      -[c;b] _ _ (λ xl _ π, let '(b -:: _) := xl in
        blockedπ b π = (vπ (uniq_bor_current m) π, uniq_bor_future m π))
      Skip _ with "LFT TIME PROPH UNIQ E L T [] [Frame I]").
    - solve_closed.
    - eapply resolve_unblock_tctx_cons_resolve.
      + simpl. by left.
      + apply resolve'. apply (source_loan_release _ _ _ _ (child_alive E L κ α Alive)).
      + apply resolve_unblock_tctx_cons_keep. apply resolve_unblock_tctx_nil.
    - apply unblock_tctx_cons_unblock; [done|apply unblock_tctx_nil].
    - iApply (proph_obs_impl with "Obs"). intros π [Loc [Cur Block]] /= Resolved.
      rewrite Block Resolved Cur. done.
    - iIntros (yl) "L T #Recovered". destruct yl as [m' []].
      wp_seq. iFrame "Frame L I". iExists m'.
      rewrite /tctx_interp /=. iDestruct "T" as "[$ _]".
      iApply (proph_obs_impl with "Recovered"). intros π [[b' []] [Block [Eq _]]].
      simpl in *. by rewrite -Eq Block.
  Qed.
End protocol.
