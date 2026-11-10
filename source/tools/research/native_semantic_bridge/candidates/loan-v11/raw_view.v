From lrust.typing Require Import borrow pptr programs type_context.
From NativeSemanticBridge Require Import borrow_handoff byte_array.
Set Default Proof Using "Type".

Section raw_view.
  Context `{!typeG Σ}.

  (* Evaluates the original pointer; keeps the ONE original loan interpretation.
     ptr_ty alone has no memory authority. No second &uniq is constructed. *)
  Lemma typed_source_raw_view {𝔄} E L I p κ (ty : type 𝔄) :
    typed_instr E L I +[p ◁ &uniq{κ} ty] p
      (λ v, +[p ◁ &uniq{κ} ty; v ◁ ptr_ty])
      (λ post '-[m], post -[m; (uniq_bor_loc m).1]).
  Proof.
    iIntros (tid post mask iκs [m []]) "_ _ _ _ _ $ $ [Src _] Obs".
    iApply (wp_hasty with "Src").
    iIntros (v d Path) "#Time [Loan %Phys]".
    destruct m as [[[[[l x] ξi] di] gi] idx].
    simpl in Phys. inversion Phys; subst v.
    iExists -[(l, x, ξi, di, gi, idx); l.1]. iFrame "Obs".
    iSplitL "Loan".
    - iExists #(l.1), d. iFrame "Time". iSplit; first done. iFrame "Loan". done.
    - rewrite right_id tctx_hasty_val. iExists d. iFrame "Time". done.
  Qed.
  Lemma typed_source_raw_offset {𝔄} E L I p κ (ty : type 𝔄) (n : Z) :
    typed_instr E L I +[p ◁ &uniq{κ} ty] (p +ₗ #n)
      (λ v, +[p ◁ &uniq{κ} ty; v ◁ ptr_ty])
      (λ post '-[m], post -[m; (uniq_bor_loc m).1 +ₗ n]).
  Proof.
    iIntros (tid post mask iκs [m []]) "_ _ _ _ _ $ $ [Src _] Obs".
    wp_bind p. iApply (wp_hasty with "Src").
    iIntros (v d Path) "#Time [Loan %Phys]".
    destruct m as [[[[[l x] ξi] di] gi] idx].
    simpl in Phys. inversion Phys; subst v. wp_op.
    iExists -[(l, x, ξi, di, gi, idx); l.1 +ₗ n]. iFrame "Obs".
    iSplitL "Loan".
    - iExists #(l.1), d. iFrame "Time". iSplit; first done. iFrame "Loan". done.
    - rewrite right_id tctx_hasty_val. iExists d. iFrame "Time". done.
  Qed.

  (* Model drop/resolution consumes the ORIGINAL loan and observes its prophecy.
     This is not a theorem about a Rust IterMut drop body. *)
  Lemma typed_source_loan_release {𝔄} E L I p κ (ty : type 𝔄) :
    lctx_lft_alive E L κ →
    typed_instr E L I +[p ◁ &uniq{κ} ty] Skip (λ _, +[])
      (λ post '-[m], λ mask π,
        uniq_bor_future m π = vπ (uniq_bor_current m) π → post -[] mask π).
  Proof. intros Alive. apply type_resolve_instr. by apply source_loan_release. Qed.
End raw_view.

Print Assumptions typed_source_raw_view.

Print Assumptions typed_source_raw_offset.
Print Assumptions typed_source_loan_release.
