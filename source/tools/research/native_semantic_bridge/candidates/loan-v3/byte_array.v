From lrust.typing Require Import int product uniq_bor.
From NativeSemanticBridge Require Import borrow_handoff.
Set Default Proof Using "Type".
Open Scope Z_scope.

Fixpoint bytes_sort (n : nat) : syn_type :=
  match n with O => unitₛ | S k => (Zₛ * bytes_sort k)%ST end.

Fixpoint byte_list (n : nat) : ~~(bytes_sort n) → list Z :=
  match n as k return ~~(bytes_sort k) → list Z with
  | O => λ _, []
  | S k => λ x, x.1 :: byte_list k x.2
  end.

Section bytes.
  Context `{!typeG Σ}.

  (* Every initialized byte value, rather than mathematical int without range. *)
  Program Definition byte_ty : type Zₛ := {|
    pt_size := 1;
    pt_gho (z : ~~Zₛ) _ := ⌜0 ≤ z < 256⌝%I;
    pt_phys (z : ~~Zₛ) _ := [FVal #z];
  |}.
  Next Obligation. intros. apply _. Qed.
  Next Obligation. done. Qed.
  Next Obligation. done. Qed.
  Next Obligation. done. Qed.
  Next Obligation. done. Qed.
  Next Obligation. done. Qed.
  Next Obligation. done. Qed.

  Fixpoint bytes_ty (n : nat) : type (bytes_sort n) :=
    match n with O => unit_ty | S k => prod_ty byte_ty (bytes_ty k) end.

  Lemma byte_list_length n (x : ~~(bytes_sort n)) : length (byte_list n x) = n.
  Proof. induction n as [|n IH]; simpl; [done|]. by rewrite IH. Qed.

  Lemma bytes_size n : ty_size (bytes_ty n) = n.
  Proof. induction n as [|n IH]; simpl; [done|]. by rewrite IH. Qed.

  Lemma bytes_physical n (x : ~~(bytes_sort n)) tid :
    ty_phys (bytes_ty n) x tid = (λ z, FVal #z) <$> byte_list n x.
  Proof. induction n as [|n IH]; simpl; [done|]. by rewrite IH. Qed.

  Lemma bytes_initialized n (x : ~~(bytes_sort n)) d g tid :
    ty_gho (bytes_ty n) x d g tid ⊣⊢
      ⌜Forall (λ z, 0 ≤ z < 256) (byte_list n x)⌝.
  Proof.
    induction n as [|n IH]; simpl.
    - iSplit; iIntros "_"; done.
    - rewrite IH. iSplit.
      + iIntros "[%Head %Tail]". iPureIntro. constructor; done.
      + iIntros "%Range". inversion Range; subst. by iSplit.
  Qed.

  (* No allocator premise and no restriction to the allocation base. The actual
     source resource occurs exactly once in source_loan_access's input. *)
  Definition byte_array128_ty := bytes_ty 128.

  Lemma byte_array128_size : ty_size byte_array128_ty = 128%nat.
  Proof. apply bytes_size. Qed.

  Lemma byte_array128_release E L κ :
    lctx_lft_alive E L κ →
    resolve E L (&uniq{κ} byte_array128_ty)
      (λ m π, uniq_bor_future m π = vπ (uniq_bor_current m) π).
  Proof. apply source_loan_release. Qed.
End bytes.

Print Assumptions bytes_initialized.
Print Assumptions byte_array128_release.
