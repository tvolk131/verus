From ModelNestedBorrowContext Require Export protocol.
From NativeSemanticBridge Require Import borrow_handoff.
From lrust.typing Require Import uniq_util inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section writes.
  Context `{!typeG Σ}.
  Lemma bytes_head_physical (z : Z) zs tid :
    ty_phys (byte_ty * bytes_ty (length zs))%T (z,bytes_of_list zs) tid =
      FVal <$> ((λ z : Z, LitV (LitInt z)) <$> (z::zs)).
  Proof. simpl. by rewrite bytes_current_physical. Qed.
  Lemma bytes_head_ghost (z : Z) zs d g tid :
    ty_gho (byte_ty * bytes_ty (length zs))%T (z,bytes_of_list zs) d g tid ⊣⊢
      ⌜Forall (λ z, 0 ≤ z < 256) (z::zs)⌝.
  Proof. simpl. rewrite bytes_current_ghost. iSplit.
    - iIntros "[%Head %Tail]". iPureIntro. constructor; done.
    - iIntros "%Range". inversion Range; subst. by iSplit.
  Qed.
  Lemma bytes_head_mapsto p z zs tid :
    (p,repeat [] (S (length zs))) #↦!∗ ty_phys (byte_ty * bytes_ty (length zs))%T (z,bytes_of_list zs) tid ⊣⊢
    p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> (z::zs)).
  Proof.
    rewrite bytes_head_physical heap_complete_mapsto_fancy_fmap_eq.
    replace (S (length zs)) with (length ((λ z : Z, LitV (LitInt z)) <$> (z::zs)))
      by (rewrite length_fmap; done).
    by rewrite -heap_cloc_mapsto_val_empty.
  Qed.
  Lemma byte_child_write E L α p (old new : Z) tail c tid :
    lctx_lft_alive E L α → 0 ≤ new < 256 →
    uniq_bor_loc c = (p, repeat [] (S (length tail))) →
    uniq_bor_current c = bytes_of_list (old::tail) →
    llft_ctx -∗ time_ctx -∗ uniq_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (S (length tail)))) c -∗
    WP (Skip;; Write ScOrd #p #new) {{ _, ∃ c', llctx_interp L ∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (S (length tail)))) c' ∗
      ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
           vπ (uniq_bor_current c') π = vπ (bytes_of_list (new::tail)) π⟩ }}.
  Proof.
    intros Alive Range Loc Cur. destruct c as [[[[[l x] ξi] di] gi] idx].
    simpl in Loc, Cur. subst l x.
    iIntros "#LFT #TIME #UNIQ #E L Parent".
    iDestruct "Parent" as (v d) "[%Path [#Td [G %Phys]]]".
    simpl in Path. inversion Path; subst v.
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt1 with "TIME"); first solve_ndisj.
    iIntros "Ticket".
    iMod (cumulative_persistent_time_receipt_get_credits with "TIME Ticket Td") as "[#TdS Credits]"; first solve_ndisj.
    iDestruct (lc_weaken 2 with "Credits") as "[CreditOpen CreditWrite]"; first (unfold advance_credits; lia).
    wp_pure Skip. wp_pure (Let _ _ _).
    iDestruct "G" as "[#Incl [%Bounds [Body [#Base #Pers]]]]".
    iMod (llctx_interp_make_guarded with "L") as (γ) "[H1 [H2 [#GuardL #Restore]]]"; first solve_ndisj.
    iMod (lctx_lft_alive_get_guards _ _ _ _ _ Alive with "GuardL E H1") as "[H1 #GuardK]"; first solve_ndisj.
    iDestruct (guards_transitive with "GuardL GuardK") as "#Guard".
    iMod (uniq_body_acc with "LFT UNIQ Guard Body H1") as "[Payload [Ticket Close]]"; first solve_ndisj.
    iMod (lc_fupd_elim_later with "CreditOpen Payload") as "[#Time [G P]]".
    iEval (rewrite bytes_head_mapsto /= heap_mapsto_vec_cons) in "P".
    iDestruct "P" as "[First Rest]".
    iDestruct (bytes_head_ghost old tail di gi tid with "G") as "%Ranges".
    inversion Ranges as [|? ? OldRange TailRange]; subst.
    iAssert (□ ty_gho (bytes_ty (S (length tail))) (bytes_of_list (new::tail)) di gi tid)%I as "#Gnew".
    { iModIntro. rewrite bytes_head_ghost. iPureIntro. constructor; done. }
    iApply wp_fupd. iApply (wp_write_sc with "[First CreditWrite]"); first done.
    { iFrame. }
    iNext. iIntros "First".
    iPoseProof ("Close" $! (bytes_of_list (new::tail)) di gi with "[First Rest] Ticket") as "Update".
    {
      iNext. iFrame "Time". iSplitR "First Rest"; first iFrame "Gnew".
      - iEval (rewrite bytes_head_mapsto /= heap_mapsto_vec_cons). iFrame. }
    iMod (fupd_mask_mono with "Update") as "[Body H1]"; first solve_ndisj.
    iPoseProof ("Restore" with "H1 H2") as "Update".
    iMod (fupd_mask_mono with "Update") as "L"; first solve_ndisj.
    iModIntro. iExists ((p,repeat [] (S (length tail))),bytes_of_list (new::tail),ξi,di,gi,idx).
    iFrame "L". iSplit.
    - iExists #p, d. iFrame "Td". iSplit; first done.
      rewrite /ty_own /=. iSplit; last done. iFrame "Incl Base Body". iSplit; first done.
      iRight. iNext. iApply (ty_gho_pers_impl _ (bytes_ty (S (length tail))) (bytes_of_list (new::tail)) di gi tid with "Gnew").
    - iApply proph_obs_true. intros π. simpl. split; last done.
      unfold uniq_bor_future. simpl.
      exact (uniq_prvar_app_irrel (bytes_of_list (new::tail)) (bytes_of_list (old::tail)) ξi π).
  Qed.
End writes.

Section contextual_write.
  Context `{!typeG Σ}.
  Lemma shorter_write_changes E L I tid iκs p κ (old new : Z) tail m (Frame : iProp Σ) :
    lctx_lft_alive E L κ →
    lctx_lft_alive E L (ty_lft (&uniq{κ} (bytes_ty (S (length tail))))%T) →
    0 ≤ new < 256 → old ≠ new →
    uniq_bor_loc m = (p,repeat [] (S (length tail))) →
    uniq_bor_current m = bytes_of_list (old::tail) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
    tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty (S (length tail)))) m -∗ Frame -∗
    WP (Newlft;; ReBor;; (Skip;; Write ScOrd #p #new);; Endlft;; Skip) {{ _,
      Frame ∗ llctx_interp L ∗ invctx_interp tid ⊤ iκs I ∗
      ∃ m', tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty (S (length tail)))) m' ∗
      ⟨π, vπ m' π = (vπ (bytes_of_list (new::tail)) π, uniq_bor_future m π) ∧
        vπ (uniq_bor_current m') π ≠ vπ (uniq_bor_current m) π⟩ }}.
  Proof.
    iIntros (Alive ParentAlive Range Changed Loc Cur) "#LFT #TIME #PROPH #UNIQ #E L I Parent Frame".
    iApply (wp_wand with "[L I Parent Frame]").
    { iApply (shorter_run E L I tid iκs #p (bytes_ty (S (length tail))) κ m
        (bytes_of_list (new::tail)) (Skip;; Write ScOrd #p #new)%E Frame
        with "LFT TIME PROPH UNIQ E L I Parent Frame"); try done.
      - intros α c ChildLoc ChildCur.
        iIntros "#LFT #TIME #PROPH #UNIQ #E L I Child".
        iApply (wp_wand with "[L Child]").
        { iApply (byte_child_write E (α ⊑ₗ [κ] :: L) α p old new tail c tid
            (child_alive E L κ α Alive) Range with "LFT TIME UNIQ E L Child").
          - exact (eq_trans ChildLoc Loc).
          - exact (eq_trans ChildCur Cur). }
        iIntros (w) "[%c' [L [Child Obs]]]". iExists c'. iFrame. }
    iIntros (w) "[Frame [L [I [%m' [Parent Obs]]]]]".
    iFrame "Frame L I". iExists m'. iFrame "Parent".
    iApply (proph_obs_impl with "Obs"). intros π Eq. split; first done.
    intro Unchanged.
    rewrite uniq_observation in Eq.
    have Current : vπ (uniq_bor_current m') π = vπ (bytes_of_list (new::tail)) π := f_equal fst Eq.
    rewrite Cur in Unchanged.
    rewrite Current in Unchanged. simpl in Unchanged.
    injection Unchanged as Head. congruence.
  Qed.
End contextual_write.
