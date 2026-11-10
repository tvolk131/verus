From ModelMutableWriter Require Export resource_protocol child.
From ModelNestedBorrowContext Require Export recovery.
From NativeSemanticBridge Require Import borrow_handoff.
From lrust.typing Require Import soundness inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section changing_initialized_reference.
  Context `{!typeG Σ}.

  Lemma initialized_nested_run zs p tid κ (x1 : ~~(bytes_sort (length zs)))
      e (Rin Rout Frame : iProp Σ) :
    Closed [] e →
    (∀ α c,
      uniq_bor_loc c = (p,repeat [] (length zs)) →
      uniq_bor_current c = bytes_of_list zs →
      llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
      llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c -∗ Rin -∗
      WP e {{ _, Rout ∗ ∃ c', llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) ∗
        invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) ∗
        tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c' ∗
        ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
             vπ (uniq_bor_current c') π = vπ x1 π⟩ }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ Rin -∗ Frame -∗
    WP (Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip) {{ _,
      Frame ∗ Rout ∗ llctx_interp [κ ⊑ₗ []] ∗
      ∃ ξi ζi idx,
      let x := bytes_of_list zs in
      let l := (p, repeat [] (length zs)) in
      let m := (l,x,ξi,0%nat,0%nat,idx) in
      let ξ := uniq_prvar x ξi in
      let ζ := PrVar (at_locₛ (bytes_sort (length zs)) ↾
                      prval_to_inh (@vπ (at_locₛ (bytes_sort (length zs))) (p,x))) ζi in
      ⟨π, uniq_bor_future m π = vπ x1 π⟩ ∗
      ⟨π, π ζ = (p,(π ξ).1)⟩ ∗
      ([†κ] ={↑Nllft}=∗ ▷ ∃ x' (d' g' : nat),
        ((λ π, (π ξ).1) :== vπ x') ∗ ⧖(S d' `max` g') ∗
        ty_gho (bytes_ty (length zs)) x' d' g' tid ∗
        l #↦!∗ ty_phys (bytes_ty (length zs)) x' tid) }}.
  Proof.
    iIntros (ClosedE RunChild) "#LFT #TIME #PROPH #UNIQ #E L I Ref Rin Frame".
    iDestruct "Ref" as (ξi ζi idx) "[Own [#Obs Back]]".
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt1 with "TIME"); first solve_ndisj.
    iIntros "Ticket". iMod persistent_time_receipt_0 as "T0".
    iMod (cumulative_persistent_time_receipt _ 1 0 with "TIME Ticket T0") as "#T1";
      first solve_ndisj.
    wp_pure Skip. wp_pure (Let _ _ _).
    iAssert (tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty (length zs)))
       ((p,repeat [] (length zs)),bytes_of_list zs,ξi,0%nat,0%nat,idx))
       with "[Own]" as "Parent".
    { iExists #p, 1%nat. iSplit; first done. iFrame "T1".
      iDestruct "Own" as "[G P]". iFrame "P".
      iDestruct "G" as "[#Incl [%Bounds [Body [#Base #Pers]]]]".
      rewrite /ty_gho /=. iSplit; first iFrame "Incl".
      iSplit; first (iPureIntro; lia). iFrame "Body". iSplit.
      - leaf_goal laters to 1%nat; first lia. iFrame "Base".
      - iDestruct "Pers" as "[Dead|Pers]".
        + iLeft. iFrame "Dead".
        + iRight. iNext. iFrame "Pers". }
    wp_bind (Newlft;; ReBor;; e;; Endlft;; Skip)%E.
    iApply (wp_wand with "[L I Parent Rin Frame]").
    { iApply (shorter_run_resources [] [κ ⊑ₗ []] (InvCtx [] static AtomicClosed)
        tid [] #p (bytes_ty (length zs)) κ _ x1 e Rin Rout Frame
        with "LFT TIME PROPH UNIQ E L I Parent Rin Frame").
      - apply fresh_local_alive.
      - apply bytes_parent_alive, fresh_local_alive.
      - done. }
    iIntros (v) "[Frame [Rout [L [I [%m' [Parent #Recovered]]]]]]".
    wp_seq.
    iApply (wp_wand with "[L I Parent]").
    { iApply (type_resolve_instr #p (&uniq{κ} (bytes_ty (length zs)))%T
        (λ m π, uniq_bor_future m π = vπ (uniq_bor_current m) π)
        [] [κ ⊑ₗ []] (InvCtx [] static AtomicClosed)
        (source_loan_release _ _ _ _ (fresh_local_alive κ))
        tid (λ _ _ π, uniq_bor_future
          ((p,repeat [] (length zs)),bytes_of_list zs,ξi,0%nat,0%nat,idx) π = vπ x1 π)
        ⊤ [] -[m'] with "LFT TIME PROPH UNIQ E L I [Parent] []").
      - simpl. iFrame.
      - iApply (proph_obs_impl with "Recovered"). intros π Eq Resolved.
        destruct m' as [[[[[l y] yi] dy] gy] i].
        unfold uniq_bor_future, uniq_bor_current in Resolved. simpl in Eq, Resolved.
        inversion Eq. congruence. }
    iIntros (v') "[%xl [L [_ [_ #Resolved]]]]".
    iFrame "Frame Rout L". iExists ξi, ζi, idx. iFrame "Resolved Obs Back".
  Qed.

  Lemma initialized_nested_payload_run zs p tid κ
      (x1 : ~~(bytes_sort (length zs))) e (Rin Rout Frame : iProp Σ) :
    Closed [] e →
    (∀ α c,
      uniq_bor_loc c = (p,repeat [] (length zs)) →
      uniq_bor_current c = bytes_of_list zs →
      llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
      llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c -∗ Rin -∗
      WP e {{ _, Rout ∗ ∃ c', llctx_interp (α ⊑ₗ [κ] :: [κ ⊑ₗ []]) ∗
        invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) ∗
        tctx_elt_interp tid (#p ◁ &uniq{α} (bytes_ty (length zs))) c' ∗
        ⟨π, uniq_bor_future c' π = uniq_bor_future c π ∧
             vπ (uniq_bor_current c') π = vπ x1 π⟩ }}) →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ Rin -∗ Frame -∗
    WP ((Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip);; Endlft) {{ _,
      Frame ∗ Rout ∗ p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> byte_list (length zs) x1) }}.
  Proof.
    iIntros (ClosedE RunChild) "#LFT #TIME #PROPH #UNIQ #E L I Reference Rin Frame".
    wp_bind (Skip;; (Newlft;; ReBor;; e;; Endlft;; Skip);; Skip)%E.
    iApply (wp_wand with "[L I Reference Rin Frame]").
    { iApply (initialized_nested_run zs p tid κ x1 e Rin Rout Frame ClosedE RunChild
        with "LFT TIME PROPH UNIQ E L I Reference Rin Frame"). }
    iIntros (w) "[Frame [Rout [L Result]]]".
    iDestruct "Result" as (ξi ζi idx) "[#Resolved [_ Back]]".
    wp_seq.
    iDestruct "L" as "[Root _]".
    iDestruct "Root" as (κ0) "[%Eq [Alive %Nonempty]]".
    simpl in Eq. rewrite left_id_L in Eq. subst κ0.
    iDestruct (llftl_end' with "LFT Alive") as "ToDead"; first done.
    wp_bind Skip.
    iApply wp_mask_mono; [|iApply (wp_step_fupd with "ToDead")]; [set_solver..|].
    wp_seq. iIntros "#Dead !>".
    iPoseProof ("Back" with "Dead") as "Update".
    iMod (fupd_mask_mono with "Update") as "Payload"; first solve_ndisj.
    wp_seq. wp_pure Skip.
    iDestruct "Payload" as (x' d' g') "[Eqz [_ [G P]]]".
    iMod ("Eqz" $! ⊤ [] 1%Qp with "[] []") as "[#Equal _]".
    { iPureIntro. split; first solve_ndisj.
      rewrite <-(bytes_vars_nil (length zs) x'). apply syn_indep. }
    { done. }
    iCombine "Resolved Equal" as "Both".
    iMod (proph_obs_sat with "PROPH Both") as %[π [Resolved Equal]]; first solve_ndisj.
    unfold uniq_bor_future in Resolved. simpl in Resolved, Equal.
    have Current : x' = x1.
    { apply (bytes_observed_injective _ _ _ π). exact (eq_trans (eq_sym Equal) Resolved). }
    subst x'.
    wp_seq. wp_seq. iFrame "Frame Rout".
    by rewrite bytes_vector_mapsto.
  Qed.
End changing_initialized_reference.
