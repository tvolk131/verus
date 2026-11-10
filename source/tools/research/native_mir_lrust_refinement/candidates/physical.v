From NativeMirLrust Require Export simulation.
From ModelInputContext Require Import composition controls.
From lrust.lang Require Import proofmode.
From lrust.typing Require Import soundness inv_context uniq_util.
From lrust.lifetime Require Import lifetime_full.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section physical.
Context `{!typeG Σ}.
Lemma word_execution_frame e v (R : iProp Σ) : word_execution e v →
  R -∗ WP e {{ w, ⌜w = v⌝ ∗ R }}.
Proof.
  intros W. inversion W; subst; iIntros "R";
    rewrite /select_data /select_len /select_thin /false_branch;
    first [wp_rec | wp_op | wp_case]; iFrame; done.
Qed.

Theorem supported_operation_physical_frame op ds σ outs σ' (R : iProp Σ) :
  source_step op ds σ outs σ' →
  ∃ es, translate op (input_words ds) = Some es ∧
  Forall2 (λ e v, ⊢ R -∗ WP e {{ w, ⌜w = v⌝ ∗ R }}) es (input_words outs).
Proof.
  intros Step. destruct (supported_operation_simulation _ _ _ _ _ Step)
    as [_ [es [Tr Sim]]]. exists es. split; first done.
  eapply Forall2_impl; first exact Sim. intros e v W. by apply word_execution_frame.
Qed.

Lemma offset_read_prefix (p : loc) (n : Z) (Φ : val → iProp Σ) :
  (WP Read ScOrd #p {{ Φ }}) -∗
  WP (let: "end" := (BinOp OffsetOp #p #n) in Read ScOrd #p) {{ Φ }}.
Proof. iIntros "Next". wp_op. wp_let. iExact "Next". Qed.

(* The descriptor grants no physical read. This theorem instead requires the
   SAME model reference, uses its existing open/close/release protocol, and
   reads the ORIGINAL first cell, never the one-past pointer. *)
Theorem offset_then_original_read_resolve zs p (z : Z) (n : Z) tid κ (Frame : iProp Σ) :
  zs !! 0%nat = Some z →
  llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗
  elctx_interp [] -∗ llctx_interp [κ ⊑ₗ []] -∗
  model_reference zs p tid κ -∗ Frame -∗ £2 -∗
  WP (let: "end" := (BinOp OffsetOp #p #n) in Read ScOrd #p) {{ w,
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
  iIntros (Head) "LFT PROPH UNIQ TIME E L Ref Frame Credits".
  iApply offset_read_prefix.
  iApply (initialized_reference_read_resolve zs p z tid κ Frame Head with "LFT PROPH UNIQ TIME E L Ref Frame Credits").
Qed.

Lemma descriptor_does_not_duplicate_payload p (z : Z) zs :
  p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> (z :: zs)) -∗
  p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> (z :: zs)) -∗ False.
Proof. apply cannot_duplicate_nonempty_payload. Qed.
End physical.

(* No owner/permission premise: the accepted initializer constructs the single
   reference internally from arbitrary raw model bytes and keeps the same heap. *)
Theorem interior128_offset_read_adequacy h w start (zs : list Z) (z : Z) :
  window_valid w → 0 < start → start + 128 < window_extent w →
  length zs = 128%nat → Forall (λ z, 0 ≤ z < 256) zs →
  (∀ (i : nat) (z : Z), zs !! i = Some z →
    h !! (pointer_loc (Pointer w start) +ₗ i) = Some #z) →
  zs !! 0%nat = Some z →
  offset_valid (Pointer w start) 128 ∧
  adequate NotStuck
    (Skip;; let: "end" := (BinOp OffsetOp #(pointer_loc (Pointer w start)) #128)
      in Read ScOrd #(pointer_loc (Pointer w start)))
    (initialized_state h) (λ v _, v = #z).
Proof.
  intros W S E Len Range Cells Head. split.
  { exact (proj1 (interior128_offset w start W S E)). }
  apply (raw_model_context_adequacy (Σ:=typeΣ)
    (let: "end" := (BinOp OffsetOp #(pointer_loc (Pointer w start)) #128)
      in Read ScOrd #(pointer_loc (Pointer w start)))%E h
    (pointer_loc (Pointer w start)) zs _ 2); [solve_closed|done|done|].
  intros ? tid κ Nonempty.
  iIntros "LFT PROPH UNIQ TIME E L _ Ref Frame Credits".
  iApply (wp_wand with "[LFT PROPH UNIQ TIME E L Ref Frame Credits]").
  { iApply (offset_then_original_read_resolve zs (pointer_loc (Pointer w start)) z 128 tid κ _ Head
      with "LFT PROPH UNIQ TIME E L Ref Frame Credits"). }
  iIntros (v) "[%Eq _]". done.
Qed.
