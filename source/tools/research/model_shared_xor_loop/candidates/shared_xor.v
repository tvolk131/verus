From lrust.lang Require Import proofmode notation lifting heap.
From lrust.typing Require Import shr_bor.
From lrust.lifetime Require Import lifetime_full.
From ModelBufferTransform Require Export transform.
From ModelByteXor Require Export byte_xor.
From ModelSharedByteRead Require Export session.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Proof data only: this does not define or replace an executable loop. *)
Definition shared_xor_result count start pad key :=
  transform_prefix count Z.lxor pad (drop start key).
Definition shared_xor_cost (count g : nat) : nat := count * (g+2).

Lemma shared_xor_cost_step count g :
  shared_xor_cost (S count) g = ((g+1) + (1 + shared_xor_cost count g))%nat.
Proof. rewrite /shared_xor_cost. lia. Qed.
Lemma shared_xor_result_bytes count start pad key :
  Forall byte_value pad → Forall byte_value key →
  Forall byte_value (shared_xor_result count start pad key).
Proof.
  rewrite /shared_xor_result. intros RP RK.
  have RD : Forall byte_value (drop start key) := Forall_drop _ _ _ RK.
  remember (drop start key) as ys eqn:Eq in RD |- *.
  clear Eq. revert pad ys RP RD.
  induction count; intros [|a pad] [|b key'] RP RD; simpl; try done.
  inversion RP; inversion RD; subst. constructor; [by apply byte_xor_range|by apply IHcount].
Qed.
Lemma shared_xor_result_facts count start pad key :
  (count ≤ length pad)%nat → (start+count ≤ length key)%nat →
  Forall byte_value pad → Forall byte_value key →
  length (shared_xor_result count start pad key) = length pad ∧
  drop count (shared_xor_result count start pad key) = drop count pad ∧
  Forall byte_value (shared_xor_result count start pad key).
Proof.
  intros BP BK RP RK. split.
  - apply transform_prefix_length; rewrite ?length_drop; lia.
  - split; last by apply shared_xor_result_bytes.
    apply transform_prefix_suffix; rewrite ?length_drop; lia.
Qed.

Section contract.
  Context `{!typeG Σ}.
  (* Exact typed milestone, an assertion to prove, not an axiom or assumed summary.
     R is internal and linear; persistent Cap is independent of any read. *)
  Definition shared_xor_contract n κ q (x : ~~(bytes_sort n)) d g tid
      (R : iProp Σ) p (pad : list Z) (start count : nat) (Frame : iProp Σ) : iProp Σ :=
    (key_reader n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_cost count g) -∗
    WP (App (of_val zip_transform)
      [of_val byte_xor; Lit (LitInt (Z.of_nat count));
       Lit (LitLoc p); Lit (LitLoc (q +ₗ start))]) @ ⊤ {{ v,
      ⌜v = LitV LitPoison⌝ ∗
      p ↦∗ zvalues (shared_xor_result count start pad (byte_list n x)) ∗
      R ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜length (shared_xor_result count start pad (byte_list n x)) = length pad ∧
        drop count (shared_xor_result count start pad (byte_list n x)) = drop count pad ∧
        Forall byte_value (shared_xor_result count start pad (byte_list n x))⌝ }})%I.
End contract.
