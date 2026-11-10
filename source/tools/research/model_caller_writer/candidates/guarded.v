From ModelCallerWriter Require Export interface.
From lrust.typing Require Import soundness inv_context programs_util borrow uniq_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section guarded_child_transform.
  Context `{!typeG Σ}.

  (* G is the other half of the SAME ordinary child context used by the reader.
     No copy of L is requested, and the caller retains R throughout this WP. *)
  Lemma byte_child_transform_guarded n E L α p (x0 x1 : ~~(bytes_sort n)) c tid e
      (G Rin Rout : iProp Σ) :
    Timeless G → lctx_lft_alive E L α →
    uniq_bor_loc c = (p,repeat [] n) → uniq_bor_current c = x0 →
    (Forall byte_value (byte_list n x0) → Forall byte_value (byte_list n x1)) →
    (Forall byte_value (byte_list n x0) →
      p ↦∗ zvalues (byte_list n x0) -∗ Rin -∗
      WP e {{ _, p ↦∗ zvalues (byte_list n x1) ∗ Rout }}) →
    llft_ctx -∗ time_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    (G &&{↑NllftG}&&> llctx_interp L) -∗ G -∗
    tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty n)) c -∗ £1 -∗ Rin -∗
    WP e {{ _, Rout ∗ ∃ c', G ∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty n)) c' ∗
      ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
           vπ (uniq_bor_current c') π = vπ x1 π⟩ }}.
  Proof.
    intros TG Alive Loc Cur RangeNew Transform.
    destruct c as [[[[[l x] ξi] di] gi] idx]. simpl in Loc, Cur. subst l x.
    iIntros "#LFT #TIME #UNIQ #E #GuardL Token Child CreditOpen Rin".
    iDestruct "Child" as (v d) "[%Path [#Td [Own %Phys]]]".
    simpl in Path. inversion Path; subst v.
    iDestruct "Own" as "[#Incl [%Bounds [Body [#Base #Pers]]]]".
    iMod (lctx_lft_alive_get_guards _ _ _ _ _ Alive with "GuardL E Token")
      as "[Token #GuardK]"; first solve_ndisj.
    iDestruct (guards_transitive with "GuardL GuardK") as "#Guard".
    iMod (uniq_body_acc with "LFT UNIQ Guard Body Token")
      as "[Payload [Ticket Close]]"; first solve_ndisj.
    iMod (lc_fupd_elim_later with "CreditOpen Payload") as "[#Time [Ghost P]]".
    iDestruct (bytes_initialized n x0 di gi tid with "Ghost") as "%RangeOld".
    iAssert (□ ty_gho (bytes_ty n) x1 di gi tid)%I as "#Gnew".
    { iModIntro. rewrite bytes_initialized. iPureIntro. by apply RangeNew. }
    iEval (rewrite bytes_vector_mapsto) in "P".
    iApply wp_fupd. iApply (wp_wand with "[P Rin]").
    { iApply (Transform RangeOld with "P Rin"). }
    iIntros (w) "[P Rout]".
    iPoseProof ("Close" $! x1 di gi with "[P] Ticket") as "Update".
    { iNext. iFrame "Time Gnew". by rewrite bytes_vector_mapsto. }
    iMod (fupd_mask_mono with "Update") as "[Body Token]"; first solve_ndisj.
    iModIntro. iFrame "Rout".
    iExists ((p,repeat [] n),x1,ξi,di,gi,idx). iFrame "Token". iSplit.
    - iExists #p,d. iFrame "Td". iSplit; first done.
      rewrite /ty_own /=. iSplit; last done. iFrame "Incl Base Body".
      iSplit; first done. iRight. iNext.
      iApply (ty_gho_pers_impl _ (bytes_ty n) x1 di gi tid with "Gnew").
    - iApply proph_obs_true. intros π. simpl. split; last done.
      unfold uniq_bor_future. simpl. exact (uniq_prvar_app_irrel x1 x0 ξi π).
  Qed.
End guarded_child_transform.
