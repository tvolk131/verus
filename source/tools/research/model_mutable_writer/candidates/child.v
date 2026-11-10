From ModelNestedBorrowContext Require Export writes.
From lrust.typing Require Import soundness inv_context uniq_util programs_util borrow.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section child_transform.
  Context `{!typeG Σ}.

  Lemma bytes_vector_mapsto n (x : ~~(bytes_sort n)) tid p :
    (p,repeat [] n) #↦!∗ ty_phys (bytes_ty n) x tid ⊣⊢
    p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list n x).
  Proof.
    rewrite bytes_physical. set (zs := byte_list n x).
    change (((p,repeat [] n) #↦!∗ ((FVal ∘ (λ z : Z, LitV (LitInt z))) <$> zs))
      ⊣⊢ (p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> zs))).
    rewrite list_fmap_compose heap_complete_mapsto_fancy_fmap_eq.
    have Len : length ((λ z : Z, LitV (LitInt z)) <$> zs) = n.
    { rewrite length_fmap /zs. apply byte_list_length. }
    rewrite -Len. by rewrite -heap_cloc_mapsto_val_empty.
  Qed.

  Lemma byte_child_transform n E L α p (x0 x1 : ~~(bytes_sort n)) c tid e
      (Rin Rout : iProp Σ) :
    lctx_lft_alive E L α →
    uniq_bor_loc c = (p,repeat [] n) → uniq_bor_current c = x0 →
    Forall (λ z, 0 ≤ z < 256) (byte_list n x1) →
    (Forall (λ z, 0 ≤ z < 256) (byte_list n x0) →
      p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list n x0) -∗ Rin -∗
      WP e {{ _, p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list n x1) ∗ Rout }}) →
    llft_ctx -∗ time_ctx -∗ uniq_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty n)) c -∗ £1 -∗ Rin -∗
    WP e {{ _, Rout ∗ ∃ c', llctx_interp L ∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty n)) c' ∗
      ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
           vπ (uniq_bor_current c') π = vπ x1 π⟩ }}.
  Proof.
    intros Alive Loc Cur RangeNew Transform.
    destruct c as [[[[[l x] ξi] di] gi] idx]. simpl in Loc, Cur. subst l x.
    iIntros "#LFT #TIME #UNIQ #E L Child CreditOpen Rin".
    iDestruct "Child" as (v d) "[%Path [#Td [G %Phys]]]".
    simpl in Path. inversion Path; subst v.
    iDestruct "G" as "[#Incl [%Bounds [Body [#Base #Pers]]]]".
    iMod (llctx_interp_make_guarded with "L") as (γ) "[H1 [H2 [#GuardL #Restore]]]";
      first solve_ndisj.
    iMod (lctx_lft_alive_get_guards _ _ _ _ _ Alive with "GuardL E H1")
      as "[H1 #GuardK]"; first solve_ndisj.
    iDestruct (guards_transitive with "GuardL GuardK") as "#Guard".
    iMod (uniq_body_acc with "LFT UNIQ Guard Body H1")
      as "[Payload [Ticket Close]]"; first solve_ndisj.
    iMod (lc_fupd_elim_later with "CreditOpen Payload") as "[#Time [G P]]".
    iDestruct (bytes_initialized n x0 di gi tid with "G") as "%RangeOld".
    iAssert (□ ty_gho (bytes_ty n) x1 di gi tid)%I as "#Gnew".
    { iModIntro. rewrite bytes_initialized. done. }
    iEval (rewrite bytes_vector_mapsto) in "P".
    iApply wp_fupd. iApply (wp_wand with "[P Rin]").
    { iApply (Transform RangeOld with "P Rin"). }
    iIntros (w) "[P Rout]".
    iPoseProof ("Close" $! x1 di gi with "[P] Ticket") as "Update".
    { iNext. iFrame "Time Gnew". by rewrite bytes_vector_mapsto. }
    iMod (fupd_mask_mono with "Update") as "[Body H1]"; first solve_ndisj.
    iPoseProof ("Restore" with "H1 H2") as "Update".
    iMod (fupd_mask_mono with "Update") as "L"; first solve_ndisj.
    iModIntro. iFrame "Rout".
    iExists ((p,repeat [] n),x1,ξi,di,gi,idx). iFrame "L". iSplit.
    - iExists #p,d. iFrame "Td". iSplit; first done.
      rewrite /ty_own /=. iSplit; last done. iFrame "Incl Base Body".
      iSplit; first done. iRight. iNext.
      iApply (ty_gho_pers_impl _ (bytes_ty n) x1 di gi tid with "Gnew").
    - iApply proph_obs_true. intros π. simpl. split; last done.
      unfold uniq_bor_future. simpl. exact (uniq_prvar_app_irrel x1 x0 ξi π).
  Qed.
End child_transform.
