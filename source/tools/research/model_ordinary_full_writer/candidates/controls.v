From ModelOrdinaryFullWriter Require Export writer.
From lrust.typing Require Import soundness.
From ModelPairInput Require Import controls.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* A concrete public instance, with no ownership or physical-WP premise. *)
Theorem raw_ordinary_full_writer_observed h p pad q key :
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
  byte_map p pad ##ₘ byte_map q key →
  ordinary_writer_claim h p pad q key.
Proof. apply (ordinary_writer_observed (Σ:=typeΣ)). Qed.

Theorem ordinary_empty_key_same_address h p pad :
  Forall (λ z, 0 ≤ z < 256) pad →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  ordinary_writer_claim h p pad p [].
Proof.
  intros Range Cells. apply raw_ordinary_full_writer_observed; try done.
  apply byte_map_empty_alias.
Qed.

Theorem ordinary_same_block_interiors (base : loc) pad key gap h :
  (0 ≤ gap)%Z →
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! ((base +ₗ 1) +ₗ i) = Some #z) →
  (∀ (i : nat) (z : Z), key !! i = Some z →
    h !! ((base +ₗ (1 + length pad + gap)) +ₗ i) = Some #z) →
  ordinary_writer_claim h (base +ₗ 1) pad (base +ₗ (1 + length pad + gap)) key.
Proof.
  intros Gap PadRange KeyRange PadCells KeyCells.
  apply raw_ordinary_full_writer_observed; try done.
  by apply byte_map_same_block_gap.
Qed.

(* This 128-byte instance has no bound on key length, including 0 and >128. *)
Theorem ordinary_128_observed h p pad q key :
  length pad = 128%nat →
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
  byte_map p pad ##ₘ byte_map q key →
  ordinary_writer_claim h p pad q key.
Proof. intros _. apply raw_ordinary_full_writer_observed. Qed.

Lemma ordinary_128_post_exact h p pad q key v σ :
  length pad = 128%nat → ordinary_post h p pad q key v σ →
  length (xor_result pad key) = 128%nat ∧
  take (Nat.min 128 (length key)) (xor_result pad key) = xor_prefix pad key ∧
  drop (Nat.min 128 (length key)) (xor_result pad key) = drop (Nat.min 128 (length key)) pad.
Proof. rewrite /ordinary_post. intros <-. tauto. Qed.

Lemma ordinary_empty_key_result pad :
  xor_result pad [] = pad ∧ ordinary_budget pad [] = 3%nat.
Proof.
  destruct pad; done.
Qed.

Theorem ordinary_empty_pad h p q key :
  Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
  ordinary_writer_claim h p [] q key.
Proof.
  intros Range Cells. apply raw_ordinary_full_writer_observed; try done.
  apply byte_map_disjoint_iff. simpl. lia.
Qed.
