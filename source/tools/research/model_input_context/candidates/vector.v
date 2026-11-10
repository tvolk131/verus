From lrust.typing Require Export int product uniq_bor.
From NativeSemanticBridge Require Export byte_array.
Set Default Proof Using "Type".

Fixpoint bytes_of_list (zs : list Z) : ~~(bytes_sort (length zs)) :=
  match zs with [] => () | z :: tail => (z, bytes_of_list tail) end.

Lemma byte_list_of_list zs : byte_list (length zs) (bytes_of_list zs) = zs.
Proof. induction zs; simpl; congruence. Qed.

Lemma byte_list_injective n (x y : ~~(bytes_sort n)) :
  byte_list n x = byte_list n y → x = y.
Proof.
  induction n as [|n IH]; destruct x, y; simpl; intros H; [done|].
  inversion H. f_equal. by apply IH.
Qed.

Lemma bytes_of_list_unique zs (x : ~~(bytes_sort (length zs))) :
  byte_list (length zs) x = zs → x = bytes_of_list zs.
Proof. intros H. apply byte_list_injective. by rewrite H byte_list_of_list. Qed.

Lemma bytes_observation_independent zs (π π' : proph_asn) :
  @vπ (bytes_sort (length zs)) (bytes_of_list zs) π =
  @vπ (bytes_sort (length zs)) (bytes_of_list zs) π'.
Proof. induction zs; simpl; [done|]. f_equal. exact IHzs. Qed.
