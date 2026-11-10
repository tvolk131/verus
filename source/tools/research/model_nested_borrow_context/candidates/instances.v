From ModelNestedBorrowContext Require Export protocol.
From ModelInputContext Require Import instances.
From NativeSemanticBridge Require Import borrow_handoff.
From lrust.typing Require Import soundness inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section initialized.
  Context `{!typeG Σ}.
  Lemma bytes_parent_alive E L κ n :
    lctx_lft_alive E L κ → lctx_lft_alive E L (ty_lft (&uniq{κ} (bytes_ty n))%T).
  Proof. intros Alive. simpl. rewrite bytes_lfts_nil /= right_id_L. done. Qed.

  Lemma initialized_nested_no_write zs p tid κ (Frame : iProp Σ) :
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ Frame -∗
    WP (Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip) {{ _,
      Frame ∗ llctx_interp [κ ⊑ₗ []] ∗
      ∃ ξi ζi idx,
      let x := bytes_of_list zs in
      let l := (p, repeat [] (length zs)) in
      let m := (l,x,ξi,0%nat,0%nat,idx) in
      let ξ := uniq_prvar x ξi in
      let ζ := PrVar (at_locₛ (bytes_sort (length zs)) ↾
                      prval_to_inh (@vπ (at_locₛ (bytes_sort (length zs))) (p,x))) ζi in
      ⟨π, uniq_bor_future m π = vπ x π⟩ ∗
      ⟨π, π ζ = (p,(π ξ).1)⟩ ∗
      ([†κ] ={↑Nllft}=∗ ▷ ∃ x' (d' g' : nat),
        ((λ π, (π ξ).1) :== vπ x') ∗ ⧖(S d' `max` g') ∗
        ty_gho (bytes_ty (length zs)) x' d' g' tid ∗
        l #↦!∗ ty_phys (bytes_ty (length zs)) x' tid) }}.
  Proof.
    iIntros "#LFT #TIME #PROPH #UNIQ #E L I Ref Frame".
    iDestruct "Ref" as (ξi ζi idx) "[Own [#Obs Back]]".
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt1 with "TIME"); first solve_ndisj.
    iIntros "Ticket". iMod persistent_time_receipt_0 as "T0".
    iMod (cumulative_persistent_time_receipt _ 1 0 with "TIME Ticket T0") as "#T1"; first solve_ndisj.
    wp_pure Skip. wp_pure (Let _ _ _).
    iAssert (tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty (length zs)))
       ((p,repeat [] (length zs)),bytes_of_list zs,ξi,0%nat,0%nat,idx)) with "[Own]" as "Parent".
    { iExists #p, 1%nat. iSplit; first done. iFrame "T1".
      iDestruct "Own" as "[G P]". iFrame "P".
      iDestruct "G" as "[#Incl [%Bounds [Body [#Base #Pers]]]]".
      rewrite /ty_gho /=. iSplit; first iFrame "Incl".
      iSplit; first (iPureIntro; lia). iFrame "Body". iSplit.
      - leaf_goal laters to 1%nat; first lia. iFrame "Base".
      - iDestruct "Pers" as "[Dead|Pers]".
        + iLeft. iFrame "Dead".
        + iRight. iNext. iFrame "Pers". }
    wp_bind (Newlft;; ReBor;; Endlft;; Skip)%E.
    iApply (wp_wand with "[L I Parent]").
    { iApply (shorter_no_write [] [κ ⊑ₗ []] (InvCtx [] static AtomicClosed) tid [] #p
        (bytes_ty (length zs)) κ _ emp%I
        with "LFT TIME PROPH UNIQ E L I Parent []").
      - apply fresh_local_alive.
      - apply bytes_parent_alive, fresh_local_alive.
      - done. }
    iIntros (v) "[_ [L [I [%m' [Parent #Recovered]]]]]".
    wp_seq.
    iApply (wp_wand with "[L I Parent]").
    { iApply (type_resolve_instr #p (&uniq{κ} (bytes_ty (length zs)))%T
        (λ m π, uniq_bor_future m π = vπ (uniq_bor_current m) π)
        [] [κ ⊑ₗ []] (InvCtx [] static AtomicClosed)
        (source_loan_release _ _ _ _ (fresh_local_alive κ))
        tid (λ _ _ π, uniq_bor_future
          ((p,repeat [] (length zs)),bytes_of_list zs,ξi,0%nat,0%nat,idx) π = vπ (bytes_of_list zs) π)
        ⊤ [] -[m'] with "LFT TIME PROPH UNIQ E L I [Parent] []").
      - simpl. iFrame.
      - iApply (proph_obs_impl with "Recovered"). intros π Eq Resolved.
        destruct m' as [[[[[l y] yi] dy] gy] i].
        unfold uniq_bor_future, uniq_bor_current in Resolved. simpl in Eq, Resolved.
        inversion Eq. congruence. }
    iIntros (v') "[%xl [L [_ [_ #Resolved]]]]".
    iFrame "Frame L". iExists ξi, ζi, idx. iFrame "Resolved Obs Back".
  Qed.
End initialized.

Theorem raw_model_nested_adequacy h p (zs : list Z) :
  Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  adequate NotStuck (Skip;; (Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip))
    (initialized_state h) (λ _ _, True).
Proof.
  intros Range Cells.
  apply (raw_model_context_adequacy (Σ:=typeΣ) _ h p zs _ 0); [solve_closed|done|done|].
  intros ? tid κ Nonempty.
  iIntros "LFT PROPH UNIQ TIME E L I Ref Frame _".
  iApply (wp_wand with "[LFT PROPH UNIQ TIME E L I Ref Frame]").
  { iApply (initialized_nested_no_write with "LFT TIME PROPH UNIQ E L I Ref Frame"). }
  iIntros (w) "_". done.
Qed.

Corollary raw_model_nested10 h p (zs : list Z) :
  length zs = 10%nat → Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  adequate NotStuck (Skip;; (Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip))
    (initialized_state h) (λ _ _, True).
Proof. intros _ Range Cells. by apply (raw_model_nested_adequacy h p zs). Qed.

Corollary raw_model_nested128 h p (zs : list Z) :
  length zs = 128%nat → Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  adequate NotStuck (Skip;; (Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip))
    (initialized_state h) (λ _ _, True).
Proof. intros _ Range Cells. by apply (raw_model_nested_adequacy h p zs). Qed.

Theorem strict_interior_nested base (prefix suffix : list val) (zs : list Z) f :
  prefix ≠ [] → suffix ≠ [] →
  interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ##ₘ f →
  Forall (λ z, 0 ≤ z < 256) zs →
  adequate NotStuck
    (Skip;; (Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip))
    (initialized_state
      (interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ∪ f))
    (λ _ _, True).
Proof.
  intros _ _ Disjoint Range.
  apply (raw_model_nested_adequacy _ (base +ₗ length prefix) zs); first done.
  by apply strict_interior_byte_lookup.
Qed.
