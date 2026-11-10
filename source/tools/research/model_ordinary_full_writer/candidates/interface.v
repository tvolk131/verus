From ModelOrdinarySharedXor Require Export adapter.
From ModelMutableWriter Require Export transform.
From ModelPairInput Require Import controls.
From lrust.typing Require Import soundness.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Frozen runtime: the checked Na loop inside the original mutable protocol. *)
Definition ordinary_loop p q (pad key : list Z) : expr :=
  App (of_val zip_transform_na)
    [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min (length pad) (length key))));
     Lit (LitLoc p); Lit (LitLoc q)].
Definition ordinary_mutation p q pad key : expr :=
  ((Skip;; (Newlft;; ReBor;; ordinary_loop p q pad key;; Endlft;; Skip);;
    Skip);; Endlft)%E.
Definition ordinary_writer p q pad key : expr :=
  pair_setup (ordinary_mutation p q pad key).

(* Exact final heap, without fixing the shared key's current reader count. *)
Definition ordinary_post h p pad q key (_ : val) (σ : state) : Prop :=
  length (xor_result pad key) = length pad ∧
  take (Nat.min (length pad) (length key)) (xor_result pad key) = xor_prefix pad key ∧
  drop (Nat.min (length pad) (length key)) (xor_result pad key) =
    drop (Nat.min (length pad) (length key)) pad ∧
  Forall byte_value (xor_result pad key) ∧
  interval_observed σ p (zvalues (xor_result pad key)) ∧
  (∀ (i : nat) (z : Z), key !! i = Some z →
    ∃ r, σ.1 !! (q +ₗ i) = Some (RSt r, #z) ∧ 0 ≤ z < 256) ∧
  frame_observed σ (pair_outside h p pad q key).

Definition ordinary_budget (pad key : list Z) : nat :=
  3 + 3 * Nat.min (length pad) (length key) + length key.

(* A typed target, not a premise or an assumed proof. *)
Definition ordinary_writer_claim h p pad q key : Prop :=
  adequate NotStuck (ordinary_writer p q pad key) (initialized_state h)
    (ordinary_post h p pad q key).
