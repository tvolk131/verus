From lrust.lang Require Import proofmode notation lifting heap.
From ModelBufferTransform Require Import transform.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Compile-time finite data, not an extra runtime operation. *)
Definition byte_indices : list nat := seq 0 256.
Definition xor_row (a : nat) : list expr :=
  (λ b : nat, Lit (LitInt (Z.lxor (Z.of_nat a) (Z.of_nat b)))) <$> byte_indices.
Definition xor_rows (b : expr) : list expr :=
  (λ a : nat, Case b (xor_row a)) <$> byte_indices.
Definition xor_body : expr := Case (Var "a") (xor_rows (Var "b")).

Lemma xor_body_closed : Closed ["a"; "b"] xor_body.
Proof.
  rewrite /Closed /xor_body /xor_rows /is_closed -/is_closed.
  apply andb_True. split; first done.
  apply forallb_True, Forall_fmap, Forall_forall. intros a _.
  simpl only [is_closed]. apply andb_True. split; first done.
  rewrite /xor_row. apply forallb_True, Forall_fmap, Forall_forall.
  intros b _. done.
Qed.
Definition byte_xor : val := @RecV BAnon ["a"; "b"] xor_body xor_body_closed.
Definition xor_call (a b : Z) : expr :=
  App (of_val byte_xor) [Lit (LitInt a); Lit (LitInt b)].

Lemma subst_xor_row x e a : (subst x e) <$> xor_row a = xor_row a.
Proof. rewrite /xor_row !list_fmap_compose. apply list_fmap_ext. intros b. done. Qed.
Lemma subst_xor_rows x e b :
  (subst x e) <$> xor_rows b = xor_rows (subst x e b).
Proof.
  rewrite /xor_rows !list_fmap_compose. apply list_fmap_ext. intros a.
  simpl only [subst]. by rewrite subst_xor_row.
Qed.
Lemma xor_beta a b :
  subst_l [BAnon; "a"; "b"]
    [of_val byte_xor; Lit (LitInt a); Lit (LitInt b)] xor_body =
  Some (Case (Lit (LitInt a)) (xor_rows (Lit (LitInt b)))).
Proof.
  cbn [subst_l subst' xor_body]. rewrite !subst_xor_rows.
  done.
Qed.
Lemma byte_indices_lookup z : byte_value z → byte_indices !! Z.to_nat z = Some (Z.to_nat z).
Proof.
  intros [Lo Hi]. rewrite /byte_indices lookup_seq; [by rewrite Nat.add_0_l|lia].
Qed.
Lemma xor_rows_lookup a b : byte_value a →
  xor_rows b !! Z.to_nat a = Some (Case b (xor_row (Z.to_nat a))).
Proof. intros H. by rewrite /xor_rows list_lookup_fmap (byte_indices_lookup a H). Qed.
Lemma xor_row_lookup a b : byte_value a → byte_value b →
  xor_row (Z.to_nat a) !! Z.to_nat b = Some (Lit (LitInt (Z.lxor a b))).
Proof.
  intros [LoA HiA] RangeB.
  rewrite /xor_row list_lookup_fmap (byte_indices_lookup b RangeB) /=.
  rewrite !Z2Nat.id; [done|exact (proj1 RangeB)|done].
Qed.

(* Three actual, deterministic, state-preserving pure steps. *)
Lemma byte_xor_pure a b : byte_value a → byte_value b →
  PureExec True 3 (xor_call a b) (Lit (LitInt (Z.lxor a b))).
Proof.
  intros RangeA RangeB _.
  eapply nsteps_l.
  - apply (pure_exec (φ:=True) (n:=1)
      (e2:=Case (Lit (LitInt a)) (xor_rows (Lit (LitInt b))))); last done.
    apply (pure_rec _ BAnon ["a"; "b"] xor_body); try apply _.
    rewrite /DoSubstL. apply xor_beta.
  - eapply nsteps_l.
    + apply (pure_exec (φ:=0 ≤ a ∧ _ = _) (n:=1)); [apply pure_case|].
      split; [exact (proj1 RangeA)|apply xor_rows_lookup; done].
    + apply nsteps_once. apply (pure_exec (φ:=0 ≤ b ∧ _ = _) (n:=1)); [apply pure_case|].
      split; [exact (proj1 RangeB)|apply xor_row_lookup; done].
Qed.
Lemma byte_xor_range a b : byte_value a → byte_value b → byte_value (Z.lxor a b).
Proof.
  intros [LoA HiA] [LoB HiB]. split.
  - apply Z.lxor_nonneg. lia.
  - assert (Bound : Z.log2 (Z.lxor a b) < 8).
    { eapply Z.le_lt_trans; first apply Z.log2_lxor; try done.
      apply Z.max_lub_lt; apply Z.log2_lt_pow2; lia. }
    destruct (Z.eq_dec (Z.lxor a b) 0); first lia.
    apply (proj1 (Z.log2_lt_pow2 _ 8)); lia.
Qed.

(* Independent list semantics: min-length XOR, then the original suffix. *)
Definition xor_prefix (xs ys : list Z) : list Z :=
  (λ ab : Z * Z, Z.lxor ab.1 ab.2) <$> zip xs ys.
Definition xor_result (xs ys : list Z) : list Z :=
  xor_prefix xs ys ++ drop (Nat.min (length xs) (length ys)) xs.
Lemma transform_xor_result xs ys :
  transform_prefix (Nat.min (length xs) (length ys)) Z.lxor xs ys = xor_result xs ys.
Proof.
  revert ys. induction xs as [|x xs IH]; intros [|y ys]; simpl; try done.
  by rewrite IH.
Qed.
Lemma xor_result_length xs ys : length (xor_result xs ys) = length xs.
Proof. rewrite -transform_xor_result. apply transform_prefix_length; lia. Qed.
Lemma xor_result_prefix xs ys :
  take (Nat.min (length xs) (length ys)) (xor_result xs ys) = xor_prefix xs ys.
Proof.
  rewrite /xor_result take_app_alt; first done.
  rewrite /xor_prefix fmap_length zip_length. done.
Qed.
Lemma xor_result_suffix xs ys :
  drop (Nat.min (length xs) (length ys)) (xor_result xs ys) =
  drop (Nat.min (length xs) (length ys)) xs.
Proof. rewrite -transform_xor_result. apply transform_prefix_suffix; lia. Qed.
Lemma xor_result_bytes xs ys :
  Forall byte_value xs → Forall byte_value ys → Forall byte_value (xor_result xs ys).
Proof.
  rewrite -transform_xor_result. revert ys.
  induction xs as [|x xs IH]; intros [|y ys] RX RY; simpl; try done.
  inversion RX; inversion RY; subst. constructor; [by apply byte_xor_range|by apply IH].
Qed.

Section operational.
  Context `{!lrustGS Σ}.
  Lemma wp_byte_xor : ⊢ binary_operation_spec Z.lxor byte_xor (Σ:=Σ).
  Proof.
    iModIntro. iIntros (a b) "%RA %RB".
    iApply (wp_pure_step_later (φ:=True) (n:=3)); [by apply byte_xor_pure|done|].
    iNext 3. by iApply wp_value.
  Qed.
  (* Internal physical resources, not original Rust/application requirements. *)
  Lemma wp_xor_memory p q xs ys :
    Forall byte_value xs → Forall byte_value ys →
    {{{ p ↦∗ zvalues xs ∗ q ↦∗ zvalues ys ∗ £(Nat.min (length xs) (length ys)) }}}
      (App (of_val zip_transform)
        [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min (length xs) (length ys))));
         Lit (LitLoc p); Lit (LitLoc q)])
    {{{ RET LitV LitPoison; p ↦∗ zvalues (xor_result xs ys) ∗ q ↦∗ zvalues ys ∗
        ⌜length (xor_result xs ys) = length xs ∧
          take (Nat.min (length xs) (length ys)) (xor_result xs ys) = xor_prefix xs ys ∧
          drop (Nat.min (length xs) (length ys)) (xor_result xs ys) =
            drop (Nat.min (length xs) (length ys)) xs ∧
          Forall byte_value (xor_result xs ys)⌝ }}}.
  Proof.
    intros RX RY. iIntros (Φ) "Resources Post".
    iApply (wp_zip_transform with "[] Resources"); [lia|lia|done|done|apply wp_byte_xor|].
    iNext. iIntros "[Pad Key]". rewrite transform_xor_result. iApply "Post". iFrame.
    iPureIntro. repeat split; auto using xor_result_length, xor_result_prefix, xor_result_suffix, xor_result_bytes.
  Qed.
  Lemma wp_xor_memory_128 p q xs ys :
    length xs = 128%nat → Forall byte_value xs → Forall byte_value ys →
    {{{ p ↦∗ zvalues xs ∗ q ↦∗ zvalues ys ∗ £(Nat.min 128 (length ys)) }}}
      (App (of_val zip_transform)
        [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min 128 (length ys))));
         Lit (LitLoc p); Lit (LitLoc q)])
    {{{ RET LitV LitPoison; p ↦∗ zvalues (xor_result xs ys) ∗ q ↦∗ zvalues ys ∗
        ⌜length (xor_result xs ys) = 128%nat ∧
          take (Nat.min 128 (length ys)) (xor_result xs ys) = xor_prefix xs ys ∧
          drop (Nat.min 128 (length ys)) (xor_result xs ys) = drop (Nat.min 128 (length ys)) xs⌝ }}}.
  Proof.
    intros Len RX RY. rewrite -Len.
    iIntros (Φ) "Resources Post". iApply (wp_xor_memory with "Resources"); [done|done|].
    iNext. iIntros "[Pad [Key %Facts]]". iApply "Post". iFrame. iPureIntro. tauto.
  Qed.
End operational.
