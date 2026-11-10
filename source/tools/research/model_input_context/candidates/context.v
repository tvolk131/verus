From NativeHeapInitialization Require Export partition.
From NativeSemanticBridge Require Export raw_view.
From ModelInputContext Require Export vector.
From lrust.typing Require Import soundness inv_context.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
From lrust.typing Require Import uniq_util.
Set Default Proof Using "Type".

Section construction.
  Context `{!typeG Σ}.

  Lemma bytes_lfts_nil n : ty_lfts (bytes_ty n) = [].
  Proof. induction n; simpl; congruence. Qed.

  Lemma bytes_current_physical zs tid :
    ty_phys (bytes_ty (length zs)) (bytes_of_list zs) tid =
      FVal <$> ((λ z : Z, LitV (LitInt z)) <$> zs).
  Proof. rewrite bytes_physical byte_list_of_list. induction zs; simpl; [done|]. f_equal. exact IHzs. Qed.

  Lemma bytes_current_ghost zs d g tid :
    ty_gho (bytes_ty (length zs)) (bytes_of_list zs) d g tid ⊣⊢
    ⌜Forall (λ z, 0 ≤ z < 256) zs⌝.
  Proof. by rewrite bytes_initialized byte_list_of_list. Qed.

  Lemma fresh_local_alive κ : lctx_lft_alive [] [κ ⊑ₗ []] κ.
  Proof. apply (lctx_lft_alive_local _ _ κ []); [by left|constructor]. Qed.

  Definition model_reference (zs : list Z) (p : loc) (tid : thread_id) (κ : lft) : iProp Σ :=
    ∃ ξi ζi idx,
      let x := bytes_of_list zs in
      let l := (p, repeat [] (length zs)) in
      let ξ := uniq_prvar x ξi in
      let ζ := PrVar (at_locₛ (bytes_sort (length zs)) ↾
                      prval_to_inh (@vπ (at_locₛ (bytes_sort (length zs))) (p,x))) ζi in
      ty_own (&uniq{κ} (bytes_ty (length zs))) (l,x,ξi,0%nat,0%nat,idx) 1 0 tid [FVal #p] ∗
      ⟨π, π ζ = (p,(π ξ).1)⟩ ∗
      ([†κ] ={↑Nllft}=∗ ▷ ∃ x' (d' g' : nat),
        ((λ π, (π ξ).1) :== vπ x') ∗ ⧖(S d' `max` g') ∗
        ty_gho (bytes_ty (length zs)) x' d' g' tid ∗
        l #↦!∗ ty_phys (bytes_ty (length zs)) x' tid).

  (* Private resource lemma: the public adequacy theorem derives P internally. *)
  Lemma bytes_reference_construct zs p tid κ F :
    Forall (λ z, 0 ≤ z < 256) zs →
    ↑Nllft ∪ ↑prophN ∪ ↑uniqN ⊆ F →
    llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ ⧖1 -∗ ⧗1 -∗
    (p, repeat [] (length zs)) #↦!∗ (FVal <$> ((λ z : Z, LitV (LitInt z)) <$> zs))
    ={F}=∗ model_reference zs p tid κ.
  Proof.
    rewrite /model_reference.
    iIntros (Range Mask) "#LFT #PROPH #UNIQ #T Ticket P".
    iAssert (ty_gho (bytes_ty (length zs)) (bytes_of_list zs) 0 0 tid) as "G".
    { rewrite bytes_current_ghost. done. }
    iDestruct (ty_gho_pers_impl with "G") as "#Pers".
    rewrite <-(bytes_current_physical zs tid).
    iMod (alloc_uniq_body _ _ 0 0 κ tid _ F with "LFT PROPH UNIQ T G P Ticket")
      as (ξi ζi idx) "[Body [Obs Back]]"; first done.
    iModIntro. iExists ξi, ζi, idx. iFrame "Obs Back".
    rewrite /ty_own /=. iSplit; last done.
    iSplit.
    { rewrite bytes_lfts_nil /=. iApply lft_incl_static. }
    iSplit; first done. iFrame "Body". iSplit; last by iRight; iNext; iFrame "Pers".
    rewrite <- heap_cloc_mapsto_empty. iApply guards_true.
  Qed.
End construction.


Section global_setup.
  Context {Σ : gFunctors} `{!typePreG Σ}.
  Theorem raw_model_context_adequacy e h p (zs : list Z) φ k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) zs →
    (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ `{!typeG Σ} tid κ, κ ≠ llft_empty →
      llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗
      elctx_interp [] -∗ llctx_interp [κ ⊑ₗ []] -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      model_reference zs p tid κ -∗
      initial_fragments (h ∖ interval_map p ((λ z : Z, LitV (LitInt z)) <$> zs)) -∗
      £k -∗ WP e @ ⊤ {{ v, ⌜φ v⌝ }}) →
    adequate NotStuck (Skip;; e) (initialized_state h) (λ v _, φ v).
  Proof using All.
    intros ClosedE Range Cells Callback.
    apply (initialized_heap_adequacy (Σ:=Σ) _ _ _ (1+k)).
    intros ?. iIntros "Fragments [LifeCredit Credits] #TIME".
    iMod (llft_alloc with "LifeCredit") as (?) "#LFT".
    iMod proph_init as (?) "#PROPH"; first done.
    iMod uniq_init as (?) "#UNIQ"; first done.
    set (Htype := TypeG Σ _ _ _ _ _ _ _).
    iMod (invctx_alloc ⊤) as (tid) "Invctx".
    iMod (llftl_begin' with "LFT") as (κ) "[Alive %Nonempty]"; first done.
    iDestruct (byte_interior_payload _ _ _ Range Cells with "Fragments")
      as "[_ [Payload Frame]]".
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt2 with "TIME"); first solve_ndisj.
    iIntros "Tickets". iDestruct (cumulative_time_receipt_sep 1 1 with "Tickets") as "[Ticket Receipt]".
    iMod persistent_time_receipt_0 as "T0".
    iMod (cumulative_persistent_time_receipt _ 1 0 with "TIME Receipt T0") as "#T1";
      first solve_ndisj.
    wp_pure Skip. wp_pure (Let _ _ _).
    iMod (@bytes_reference_construct Σ Htype zs p tid κ ⊤ with "LFT PROPH UNIQ T1 Ticket Payload")
      as "Reference"; [done|solve_ndisj|].
    iApply (Callback Htype tid κ Nonempty with "LFT PROPH UNIQ TIME [] [Alive] Invctx Reference Frame Credits").
    - by rewrite /elctx_interp.
    - rewrite /llctx_interp /llctx_elt_interp /= right_id.
      iExists κ. iFrame "Alive". by rewrite left_id.
  Qed.
End global_setup.
