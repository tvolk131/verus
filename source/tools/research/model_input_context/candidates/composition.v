From ModelInputContext Require Export context.
From NativeSemanticBridge Require Import borrow_handoff.
From lrust.typing Require Import soundness inv_context uniq_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".

Section operational_composition.
  Context `{!typeG Σ}.

  Lemma initialized_reference_read_resolve zs p (z : Z) tid κ (Frame : iProp Σ) :
    zs !! 0%nat = Some z →
    llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗
    elctx_interp [] -∗ llctx_interp [κ ⊑ₗ []] -∗
    model_reference zs p tid κ -∗ Frame -∗ £2 -∗
    WP Read ScOrd #p {{ w,
      ⌜w = #z⌝ ∗ Frame ∗ llctx_interp [κ ⊑ₗ []] ∗
      ∃ ξi ζi idx,
      let x := bytes_of_list zs in
      let l := (p, repeat [] (length zs)) in
      let m := (l,x,ξi,0%nat,0%nat,idx) in
      let ξ := uniq_prvar x ξi in
      let ζ := PrVar (at_locₛ (bytes_sort (length zs)) ↾
                      prval_to_inh (@vπ (at_locₛ (bytes_sort (length zs))) (p,x))) ζi in
      ⟨π, uniq_bor_future m π = vπ (uniq_bor_current m) π⟩ ∗
      ⟨π, π ζ = (p,(π ξ).1)⟩ ∗
      ([†κ] ={↑Nllft}=∗ ▷ ∃ x' (d' g' : nat),
        ((λ π, (π ξ).1) :== vπ x') ∗ ⧖(S d' `max` g') ∗
        ty_gho (bytes_ty (length zs)) x' d' g' tid ∗
        l #↦!∗ ty_phys (bytes_ty (length zs)) x' tid)
    }}.
  Proof.
    iIntros (Head) "#LFT #PROPH #UNIQ #TIME #E L Ref Frame [CreditRead CreditResolve]".
    iDestruct "Ref" as (ξi ζi idx) "[Own [#Obs Back]]".
    iMod (source_context_access _ κ [] [κ ⊑ₗ []] with "LFT UNIQ E L Own")
      as "[_ [Payload [Ticket Close]]]"; first apply fresh_local_alive.
    iMod (lc_fupd_elim_later with "CreditRead Payload") as "[#T [G P]]".
    iEval (rewrite bytes_current_physical heap_complete_mapsto_fancy_fmap_eq
      -(length_fmap (λ z : Z, LitV (LitInt z)) zs) -heap_cloc_mapsto_val_empty) in "P".
    destruct zs as [|z0 tail]; first done. simpl in Head. inversion Head; subst z0.
    iEval (rewrite /= heap_mapsto_vec_cons) in "P".
    iDestruct "P" as "[First Rest]".
    iApply wp_fupd. iApply (wp_read_sc with "First"); first done.
    iNext. iIntros "First".
    iMod ("Close" with "[G First Rest] Ticket") as "[Own L]".
    { iNext. iFrame "T G". iEval (rewrite bytes_current_physical
        heap_complete_mapsto_fancy_fmap_eq
        -(length_fmap (λ z : Z, LitV (LitInt z)) (z :: tail))
        -heap_cloc_mapsto_val_empty /= heap_mapsto_vec_cons).
      iFrame. }
    iDestruct "Own" as "[Gho _]".
    iMod (llctx_interp_make_guarded with "L") as (γ) "[H1 [H2 [#Guard #RestoreL]]]";
      first solve_ndisj.
    iMod (source_loan_release [] [κ ⊑ₗ []] κ _ (fresh_local_alive κ)
      _ ⊤ with "LFT PROPH UNIQ TIME E Guard H1 Gho") as "Resolve"; first solve_ndisj.
    iMod (lc_step_fupdN_elim_later with "CreditResolve Resolve") as "Resolve".
    iMod "Resolve" as "[#Resolved H1]".
    iMod (fupd_mask_mono with "[H1 H2]") as "L"; first solve_ndisj.
    { iApply ("RestoreL" with "H1 H2"). }
    iModIntro. iSplit; first done. iFrame "Frame L".
    iExists ξi, ζi, idx. iFrame "Resolved Obs Back".
  Qed.
End operational_composition.
