From ModelInputContext Require Export vector.
From ModelByteXor Require Export byte_xor.
Set Default Proof Using "Type".

Definition bytes_cast {n m : nat} (E : n = m) (x : ~~(bytes_sort n)) : ~~(bytes_sort m) :=
  eq_rect n (λ k, ~~(bytes_sort k)) x m E.

Lemma byte_list_cast n m (E : n = m) (x : ~~(bytes_sort n)) :
  byte_list m (bytes_cast E x) = byte_list n x.
Proof. destruct E. done. Qed.

Definition bytes_from_length {n : nat} (zs : list Z) (E : length zs = n) : ~~(bytes_sort n) :=
  bytes_cast E (bytes_of_list zs).

Lemma byte_list_from_length n zs (E : length zs = n) :
  byte_list n (bytes_from_length zs E) = zs.
Proof. rewrite /bytes_from_length byte_list_cast. apply byte_list_of_list. Qed.
