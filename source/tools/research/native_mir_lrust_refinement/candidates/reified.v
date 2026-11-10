From NativeMirLrust Require Export simulation.
Definition selected_operations : list operation := [Metadata; WrapWide; UnwrapWide; Thin; WrapThin; UnwrapThin; ByteOffset; U8ZstBranch].
Lemma selected_operations_exact : selected_operations =
 [Metadata; WrapWide; UnwrapWide; Thin; WrapThin; UnwrapThin; ByteOffset; U8ZstBranch].
Proof. reflexivity. Qed.
Lemma selected_operations_supported : Forall (λ op, ∀ why, op ≠ Unsupported why) selected_operations.
Proof. repeat constructor; intros why H; discriminate. Qed.
