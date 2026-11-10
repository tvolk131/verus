From NativeSemanticBridge Require Export byte_array.
Set Default Proof Using "Type".

Fixpoint bytes_of_list (zs : list Z) : ~~(bytes_sort (length zs)) :=
  match zs with [] => () | z :: tail => (z, bytes_of_list tail) end.

Lemma byte_list_of_list zs : byte_list (length zs) (bytes_of_list zs) = zs.
Proof. induction zs; simpl; congruence. Qed.

Lemma bytes_of_byte_list n (x : ~~(bytes_sort n)) :
  ∃ y : ~~(bytes_sort (length (byte_list n x))),
    y = bytes_of_list (byte_list n x) ∧ byte_list _ y = byte_list n x.
Proof. exists (bytes_of_list (byte_list n x)). split; [done|apply byte_list_of_list]. Qed.
