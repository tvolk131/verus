From ModelPairReader Require Export handoff.
From lrust.typing Require Import borrow type_context programs inv_context soundness.
From lrust.lang Require Import proofmode notation lifting.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section constructive_share.
  Context `{!typeG Σ}.

  (* Specialize the constructive core of type_share_instr before tctx existential
     packaging. resolve_uniq_body retains the initialized 0/0 payload indices. *)
  Lemma model_key_share_fixed key q tid κ :
    Forall (λ z, 0 ≤ z < 256) key →
    llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗
    model_reference key q tid κ -∗
    WP Share {{ _, llctx_interp [κ ⊑ₗ []] ∗
      ModelSharedByteRead.shared_read.shared_key_view
        (length key) κ q (bytes_of_list key) 1 0 tid }}.
  Proof.
    iIntros (Range) "#LFT #PROPH #UNIQ #TIME #E L Ref".
    iDestruct "Ref" as (ξi ζi idx) "[Own _]".
    iDestruct "Own" as "[[#Incl [%Ineq [Body [#Base #Pers]]]] _]".
    iMod (llctx_interp_make_guarded with "L") as (γ) "[H1 [H2 [#Guard #Close]]]";
      first solve_ndisj.
    iMod (resolve_uniq_body (bytes_ty (length key)) (bytes_of_list key) ξi 0 0 idx κ tid
      (q, repeat [] (length key)) [] [κ ⊑ₗ []] with
      "LFT PROPH UNIQ TIME Incl E Guard H1 Body") as "[_ [H1 #Shared]]";
      [apply fresh_local_alive|solve_ndisj|].
    iDestruct ("Close" with "H1 H2") as "Restore".
    iMod (fupd_mask_mono with "Restore") as "L"; first solve_ndisj.
    iAssert (ModelSharedByteRead.shared_read.shared_key_view
      (length key) κ q (bytes_of_list key) 1 0 tid) as "#View".
    { rewrite /ModelSharedByteRead.shared_read.shared_key_view /ty_own /=.
      iSplit; last done. iSplit.
      - iApply (guard_cloc_combine_fancy _ _ _ (q, repeat [] (length key)) _).
        + iFrame "Base".
        + iApply (guards_weaken_rhs_sep_r with "Shared").
      - iSplit.
        + iApply (guards_weaken_rhs_sep_l with "Shared").
        + iNext. iApply ty_gho_pers_impl. rewrite bytes_current_ghost. done. }
    wp_pure Skip. wp_pure (Let _ _ _). wp_pure Skip.
    iFrame "L View".
  Qed.

  Lemma fixed_view_pair_context key q tid κ :
    ⧖1 -∗ ModelSharedByteRead.shared_read.shared_key_view
      (length key) κ q (bytes_of_list key) 1 0 tid -∗
    ModelPairInput.setup.shared_key_view key q tid κ.
  Proof.
    rewrite ModelPairInput.setup.shared_key_view_own.
    iIntros "#T #View". iExists 1%nat. iFrame "T".
    iPoseProof "View" as "Own".
    rewrite /ModelSharedByteRead.shared_read.shared_key_view /ty_own.
    iDestruct "Own" as "[G P]". iFrame "P".
    iDestruct (ty_gho_depth_mono _ 1 0 1 1 with "G") as "[$ _]"; lia.
  Qed.
End constructive_share.
