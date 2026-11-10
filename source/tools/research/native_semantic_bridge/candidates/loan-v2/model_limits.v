From lrust.typing Require Import pptr programs.
From lrust.lang Require Import lang notation.
Set Default Proof Using "Type".

(* A meaningful operational control: a pure raw pointer type contains no heap
   authority. This is a proved bad-model-claim counterexample, not a failed script. *)
Section pointer_value.
  Context `{!typeG Σ}.
  Lemma arbitrary_raw_pointer_typed (l : loc) : typed_val #l ptr_ty l.
  Proof.
    iIntros (????????) "_ _ _ _ _ $ $ _ Obs".
    iMod persistent_time_receipt_0 as "Time".
    iApply wp_value. iExists -[l]. iFrame "Obs". iSplit; last done.
    rewrite tctx_hasty_val'; last done. iExists 0%nat. iFrame "Time". done.
  Qed.
End pointer_value.

Lemma raw_read_empty_head_stuck l κ e σ efs :
  ¬ head_step (Read ScOrd (Lit (LitLoc l))) (∅, false) κ e σ efs.
Proof. intros Step. inversion Step; subst. naive_solver. Qed.

(* The actual language allows ANY mathematical pointer offset, independently of
   allocations. This rule cannot silently stand in for Rust's checked in-bounds
   Offset primitive: an explicit restricted simulation is needed. *)
Lemma model_offset_unrestricted σ (l : loc) (z : Z) :
  bin_op_eval σ OffsetOp (LitLoc l) (LitInt z) (LitLoc (l +ₗ z)).
Proof. constructor. Qed.

Print Assumptions arbitrary_raw_pointer_typed.
Print Assumptions raw_read_empty_head_stuck.
Print Assumptions model_offset_unrestricted.
