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
End raw_view.

Print Assumptions typed_source_raw_view.
