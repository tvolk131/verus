From ModelOrdinaryFullWriter Require Export interface.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Local transport only; no root vectors source/object/failed receipt is used. *)
Definition bytes_cast n m (Eq : n = m) (x : ~~(bytes_sort n)) : ~~(bytes_sort m) :=
  eq_rect n (λ k, ~~(bytes_sort k)) x m Eq.
Lemma byte_list_cast n m Eq x : byte_list m (bytes_cast n m Eq x) = byte_list n x.
Proof. destruct Eq. done. Qed.
Definition final_bytes pad key : ~~(bytes_sort (length pad)) :=
  bytes_cast _ _ (xor_result_length pad key) (bytes_of_list (xor_result pad key)).
Lemma final_bytes_exact pad key :
  byte_list (length pad) (final_bytes pad key) = xor_result pad key.
Proof. rewrite /final_bytes byte_list_cast byte_list_of_list. done. Qed.
Lemma final_bytes_range pad key :
  Forall byte_value pad → Forall byte_value key →
  Forall byte_value (byte_list (length pad) (final_bytes pad key)).
Proof. rewrite final_bytes_exact. apply xor_result_bytes. Qed.
Lemma ordinary_loop_closed p q pad key : Closed [] (ordinary_loop p q pad key).
Proof. rewrite /ordinary_loop. solve_closed. Qed.
Lemma ordinary_mutation_closed p q pad key : Closed [] (ordinary_mutation p q pad key).
Proof. rewrite /ordinary_mutation /ordinary_loop. solve_closed. Qed.
Lemma ordinary_budget_split pad key :
  ordinary_budget pad key =
  (2 + (1 + shared_xor_na_cost (Nat.min (length pad) (length key)) 0 + length key))%nat.
Proof. rewrite /ordinary_budget /shared_xor_na_cost. lia. Qed.
