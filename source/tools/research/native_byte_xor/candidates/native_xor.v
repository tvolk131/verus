From iris.program_logic Require Import lifting.
From lrust.lang Require Import proofmode notation lifting heap.
From ModelBufferTransform Require Import transform.
From ModelByteXor Require Import byte_xor.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Little-endian words. Neither Boolean XOR nor positional evaluation uses lxor. *)
Fixpoint bits (width : nat) (z : Z) : list bool :=
  match width with
  | O => []
  | S n => Z.odd z :: bits n (Z.div2 z)
  end.
Fixpoint word_xor (xs ys : list bool) : list bool :=
  match xs,ys with
  | x::xs,y::ys => xorb x y :: word_xor xs ys
  | _,_ => []
  end.
Fixpoint word_value (xs : list bool) : Z :=
  match xs with
  | [] => 0
  | x::xs => (if x then 1 else 0) + 2 * word_value xs
  end.
Definition scalar_xor8 (a b : Z) : Z :=
  word_value (word_xor (bits 8 a) (bits 8 b)).

Lemma bits_length n z : length (bits n z) = n.
Proof. revert z. induction n; intros z; simpl; auto. Qed.
Lemma word_xor_length xs ys :
  length (word_xor xs ys) = Nat.min (length xs) (length ys).
Proof. revert ys. induction xs; intros [|y ys]; simpl; auto. Qed.
Lemma word_value_range xs : 0 ≤ word_value xs < 2 ^ Z.of_nat (length xs).
Proof.
  induction xs as [|x xs IH]; simpl; first lia.
  rewrite Nat2Z.inj_succ Z.pow_succ_r; last lia.
  destruct x; simpl; lia.
Qed.
Lemma bits_value n a : 0 ≤ a < 2 ^ Z.of_nat n → word_value (bits n a) = a.
Proof.
  revert a. induction n as [|n IH]; intros a Bound; simpl in *.
  - assert (a = 0) by lia. subst. done.
  - rewrite Nat2Z.inj_succ Z.pow_succ_r in Bound; last lia.
    pose proof (Z.div2_odd a) as Decompose.
    assert (Half : 0 ≤ Z.div2 a < 2 ^ Z.of_nat n).
    { destruct (Z.odd a); simpl in Decompose; lia. }
    rewrite IH; last exact Half. destruct (Z.odd a); simpl in *; lia.
Qed.
Lemma odd_lxor a b : Z.odd (Z.lxor a b) = xorb (Z.odd a) (Z.odd b).
Proof. rewrite -!Z.bit0_odd Z.lxor_spec. done. Qed.
Lemma div2_lxor a b : Z.div2 (Z.lxor a b) = Z.lxor (Z.div2 a) (Z.div2 b).
Proof. rewrite !Z.div2_spec Z.shiftr_lxor. done. Qed.
Lemma word_xor_bits n a b : word_xor (bits n a) (bits n b) = bits n (Z.lxor a b).
Proof.
  revert a b. induction n; intros a b; simpl; first done.
  rewrite odd_lxor div2_lxor IHn. done.
Qed.
Lemma scalar_xor8_lxor a b :
  byte_value a → byte_value b → scalar_xor8 a b = Z.lxor a b.
Proof.
  intros RA RB. rewrite /scalar_xor8 word_xor_bits.
  apply bits_value. exact (byte_xor_range a b RA RB).
Qed.
Lemma scalar_xor8_range a b : byte_value (scalar_xor8 a b).
Proof.
  rewrite /scalar_xor8 /byte_value.
  pose proof (word_value_range (word_xor (bits 8 a) (bits 8 b))) as Bound.
  rewrite word_xor_length !bits_length in Bound. exact Bound.
Qed.

(* Only the concrete scalar rvalue computation is pure, for exactly three steps. *)
Lemma scalar_xor8_pure a b : byte_value a → byte_value b →
  PureExec True 3 (xor_call a b) (Lit (LitInt (scalar_xor8 a b))).
Proof. intros RA RB. rewrite scalar_xor8_lxor; auto using byte_xor_pure. Qed.
Lemma scalar_xor8_normalizes a b (σ : state) : byte_value a → byte_value b →
  nsteps (λ e e', @language.prim_step lrust_lang e σ [] e' σ []) 3
    (xor_call a b) (Lit (LitInt (scalar_xor8 a b))).
Proof. intros RA RB. rewrite scalar_xor8_lxor; auto using byte_xor_normalizes. Qed.

(* A conditional MODEL translation: read and write the SAME p. b is already
   obtained by moving local 19; no native dereference rule is postulated. *)
Definition cell_assignment (p : loc) (b : Z) : expr :=
  (let: "a" := !ˢᶜ #(LitLoc p) in
  let: "c" := (of_val byte_xor) ["a"; #(LitInt b)] in
  #(LitLoc p) <-ˢᶜ "c")%E.

(* The optional register adapter is ordinary data, not a language primitive.
   It removes 19 before returning the expression and does not access a heap. *)
Definition register_file := nat → option Z.
Definition without19 (rs : register_file) : register_file :=
  λ n, if Nat.eq_dec n 19%nat then None else rs n.
Definition prepare_assignment (p : loc) (rs : register_file) : option (expr * register_file) :=
  match rs 19%nat with
  | Some b => Some (cell_assignment p b, without19 rs)
  | None => None
  end.
Lemma prepare_assignment_some p rs b : rs 19%nat = Some b →
  prepare_assignment p rs = Some (cell_assignment p b, without19 rs).
Proof. intros H. rewrite /prepare_assignment H. done. Qed.
Lemma prepare_assignment_none p rs : rs 19%nat = None → prepare_assignment p rs = None.
Proof. intros H. rewrite /prepare_assignment H. done. Qed.
Lemma moved19_unavailable rs : without19 rs 19%nat = None.
Proof. rewrite /without19. destruct (Nat.eq_dec 19%nat 19%nat); done. Qed.
Lemma other_register_preserved rs n : n ≠ 19%nat → without19 rs n = rs n.
Proof. intros NE. rewrite /without19. destruct (Nat.eq_dec n 19%nat); done. Qed.

Section operational.
  Context `{!lrustGS Σ}.
  Lemma wp_scalar_xor8 a b : byte_value a → byte_value b →
    ⊢ WP (xor_call a b) {{ w, ⌜w = #(scalar_xor8 a b)⌝ }}.
  Proof.
    intros RA RB. rewrite scalar_xor8_lxor; [|done|done].
    iApply (wp_byte_xor $! a b with "[] []"); done.
  Qed.
  (* Cell ownership and one store credit are explicit internal WP resources.
     R is arbitrary and preserved, including all disjoint heap/key resources. *)
  Lemma wp_cell_assignment p a b (R : iProp Σ) : byte_value a → byte_value b →
    {{{ p ↦ #a ∗ £1 ∗ R }}}
      cell_assignment p b
    {{{ RET LitV LitPoison; p ↦ #(scalar_xor8 a b) ∗ R ∗
        ⌜byte_value (scalar_xor8 a b)⌝ }}}.
  Proof.
    intros RA RB. iIntros (Φ) "[Cell [Credit Frame]] Post".
    rewrite /cell_assignment.
    remember byte_xor as op eqn:OpEq.
    wp_bind (Read ScOrd (Lit (LitLoc p))).
    iApply (wp_read_sc with "Cell"); first done.
    iNext. iIntros "Cell". wp_let.
    wp_bind (App (of_val op) [Lit (LitInt a); Lit (LitInt b)]).
    iApply (wp_wand with "[]").
    { rewrite OpEq. iApply (wp_scalar_xor8 a b RA RB). }
    iIntros (w) "%Eq". subst w. wp_let.
    iApply (wp_write_sc ⊤ p _ _ _ with "[Cell Credit]"); [done|by iFrame|].
    iNext. iIntros "Cell". iApply "Post". iFrame.
    iPureIntro. apply scalar_xor8_range.
  Qed.
  Lemma wp_prepared_assignment p rs a b (R : iProp Σ) :
    rs 19%nat = Some b → byte_value a → byte_value b →
    ⊢ ∃ e rs', ⌜prepare_assignment p rs = Some (e, rs') ∧ rs' 19%nat = None ∧
      (∀ n, n ≠ 19%nat → rs' n = rs n)⌝ ∧
      ({{{ p ↦ #a ∗ £1 ∗ R }}} e
       {{{ RET LitV LitPoison; p ↦ #(scalar_xor8 a b) ∗ R ∗
           ⌜byte_value (scalar_xor8 a b)⌝ }}}).
  Proof.
    intros At19 RA RB. iExists (cell_assignment p b), (without19 rs).
    iSplit; first (iPureIntro; repeat split; auto using prepare_assignment_some,
      moved19_unavailable, other_register_preserved).
    iIntros (Φ) "!> Resources Post".
    iApply (wp_cell_assignment p a b R RA RB with "Resources Post").
  Qed.
End operational.
