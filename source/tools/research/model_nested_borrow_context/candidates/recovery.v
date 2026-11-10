From ModelNestedBorrowContext Require Export instances.
From lrust.typing Require Import soundness inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Lemma bytes_vars_nil n (x : ~~(bytes_sort n)) : ξl x = [].
Proof. induction n; destruct x; simpl; [done|]. apply IHn. Qed.

Lemma bytes_observed_injective n (x y : ~~(bytes_sort n)) π :
  vπ x π = vπ y π → x = y.
Proof.
  induction n; destruct x, y; simpl; intros Eq; [done|].
  injection Eq as Head Tail. subst. f_equal. by apply IHn.
Qed.

Section payload_recovery.
  Context `{!typeG Σ}.
  Lemma initialized_nested_payload zs p tid κ (Frame : iProp Σ) :
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference zs p tid κ -∗ Frame -∗
    WP ((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft) {{ _,
      Frame ∗ p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> zs) }}.
  Proof.
    iIntros "#LFT #TIME #PROPH #UNIQ #E L I Reference Frame".
    wp_bind (Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip)%E.
    iApply (wp_wand with "[L I Reference Frame]").
    { iApply (initialized_nested_no_write with "LFT TIME PROPH UNIQ E L I Reference Frame"). }
    iIntros (w) "[Frame [L Result]]".
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
    { iPureIntro. split; first solve_ndisj. rewrite <-(bytes_vars_nil (length zs) x'). apply syn_indep. }
    { done. }
    iCombine "Resolved Equal" as "Both".
    iMod (proph_obs_sat with "PROPH Both") as %[π [Resolved Equal]]; first solve_ndisj.
    unfold uniq_bor_future in Resolved. simpl in Resolved, Equal.
    have Current : x' = bytes_of_list zs.
    { apply (bytes_observed_injective _ _ _ π). exact (eq_trans (eq_sym Equal) Resolved). }
    subst x'.
    wp_seq. wp_seq. iFrame "Frame".
    iEval (rewrite bytes_current_physical heap_complete_mapsto_fancy_fmap_eq
      -(length_fmap (λ z : Z, LitV (LitInt z)) zs) -heap_cloc_mapsto_val_empty) in "P".
    iFrame "P".
  Qed.
End payload_recovery.

Theorem raw_model_nested_read_adequacy h p (zs : list Z) (z : Z) :
  Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  adequate NotStuck
    (Skip;; (((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft);; Read ScOrd #p))
    (initialized_state h) (λ w _, w = #z).
Proof.
  intros Range Cells Head.
  apply (raw_model_context_adequacy (Σ:=typeΣ) _ h p zs _ 0); [solve_closed|done|done|].
  intros TG tid κ Nonempty.
  iIntros "#LFT #PROPH #UNIQ #TIME #E L I Reference Frame _".
  wp_bind ((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft)%E.
  iApply (wp_wand with "[L I Reference Frame]").
  { iApply (initialized_nested_payload with "LFT TIME PROPH UNIQ E L I Reference Frame"). }
  iIntros (w) "[Frame Payload]". wp_seq.
  destruct zs as [|head tail]; first done. simpl in Head. inversion Head; subst head.
  iEval (rewrite /= heap_mapsto_vec_cons) in "Payload".
  iDestruct "Payload" as "[First Rest]".
  iApply (wp_read_sc with "First"); first done.
  iNext. iIntros "First". done.
Qed.

Corollary raw_model_nested_read10 h p (zs : list Z) (z : Z) :
  length zs = 10%nat → Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  adequate NotStuck
    (Skip;; (((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft);; Read ScOrd #p))
    (initialized_state h) (λ w _, w = #z).
Proof. intros _ Range Cells Head. by apply (raw_model_nested_read_adequacy h p zs). Qed.

Corollary raw_model_nested_read128 h p (zs : list Z) (z : Z) :
  length zs = 128%nat → Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  adequate NotStuck
    (Skip;; (((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft);; Read ScOrd #p))
    (initialized_state h) (λ w _, w = #z).
Proof. intros _ Range Cells Head. by apply (raw_model_nested_read_adequacy h p zs). Qed.

Theorem strict_interior_nested_read base (prefix suffix : list val) (zs : list Z) f (z : Z) :
  prefix ≠ [] → suffix ≠ [] →
  interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ##ₘ f →
  Forall (λ z, 0 ≤ z < 256) zs → zs !! 0%nat = Some z →
  adequate NotStuck
    (Skip;; (((Skip;; (Newlft;; ReBor;; Endlft;; Skip);; Skip);; Endlft);;
      Read ScOrd #(base +ₗ length prefix)))
    (initialized_state
      (interval_map base (prefix ++ ((λ z : Z, LitV (LitInt z)) <$> zs) ++ suffix) ∪ f))
    (λ w _, w = #z).
Proof.
  intros _ _ Disjoint Range Head.
  apply (raw_model_nested_read_adequacy _ _ zs); [done| |done].
  by apply strict_interior_byte_lookup.
Qed.
