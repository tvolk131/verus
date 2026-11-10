From NativeMirLrust Require Export simulation.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Empty endpoints have no model block and no readable-cell premise. *)
Inductive endpoint :=
| InAllocation (p : pointer)
| EmptyAddress (address : Z).
Definition address e := match e with
  | InAllocation p => pointer_address p | EmptyAddress a => a end.
Definition endpoint_valid e : Prop := match e with
  | InAllocation p => pointer_valid p
  | EmptyAddress a => 1 ≤ a ≤ usize_max end.
Inductive span : endpoint → endpoint → Z → Prop :=
| SpanAllocation p n : slice_valid p n →
    span (InAllocation p) (InAllocation (shifted p n)) n
| SpanEmpty e : endpoint_valid e → span e e 0.

Definition modulus : Z := 2^64.
(* Explicit finite-width scalar semantics for the pinned LLVM lowering.
   This does NOT add a Rust intrinsic rule to lrust. *)
Definition unsigned_sub (bits a b : Z) : option Z :=
  if bool_decide (0 ≤ b ≤ a ∧ a < 2^bits)
  then Some ((a-b) mod (2^bits)) else None.
Definition exact_udiv (d size : Z) : option Z :=
  if bool_decide (0 < size ∧ d mod size = 0)
  then Some (d / size) else None.
Definition lowered_distance (bits size a b : Z) : option Z :=
  match unsigned_sub bits a b with
  | Some d => exact_udiv d size | None => None end.

Lemma span_bounds p q n : span p q n →
  1 ≤ address p ∧ address p ≤ address q ∧ address q ≤ usize_max ∧
  0 ≤ n ≤ isize_max ∧ address q - address p = n.
Proof.
  intros H. destruct H as [r len H|r H].
  - destruct r as [[b a e] k].
    rewrite /slice_valid /pointer_valid /window_valid /pointer_window
      /pointer_cursor /window_address /window_extent /= in H.
    rewrite /address /pointer_address /shifted /pointer_window
      /pointer_cursor /window_address /=. intuition lia.
  - destruct r as [[[b a e] k]|a].
    + rewrite /endpoint_valid /pointer_valid /window_valid /pointer_window
        /pointer_cursor /window_address /window_extent /= in H.
      rewrite /address /pointer_address /pointer_window /pointer_cursor
        /window_address /= /isize_max. intuition lia.
    + rewrite /endpoint_valid /= in H. rewrite /address /= /isize_max. intuition lia.
Qed.
Lemma allocated_formation p n : slice_valid p n →
  offset_valid p n ∧ pointer_valid (shifted p n) ∧
  (pointer_loc (shifted p n)).1 = (pointer_loc p).1 ∧
  pointer_loc (shifted p n) = pointer_loc p +ₗ n.
Proof.
  intros H. assert (O : offset_valid p n).
  { destruct p as [[b a e] k].
    rewrite /slice_valid /pointer_valid /window_valid /pointer_window
      /pointer_cursor /window_address /window_extent /= in H.
    rewrite /offset_valid /pointer_valid /pointer_window /pointer_cursor
      /window_valid /window_address /window_extent /=.
    intuition lia. }
  destruct (offset_result_bounds _ _ O) as [V [_ [_ B]]].
  exact (conj O (conj V (conj B (pointer_loc_shift p n)))).
Qed.
Lemma unsigned_sub_exact a b :
  0 ≤ b ≤ a → a ≤ usize_max →
  unsigned_sub 64 a b = Some (a-b).
Proof.
  intros B A. rewrite /unsigned_sub.
  case_bool_decide; last (rewrite /usize_max in A; cbn in *; lia).
  rewrite Z.mod_small; first done. rewrite /usize_max in A; cbn; lia.
Qed.
Lemma lowered_u8_exact p q n : span p q n →
  lowered_distance 64 1 (address q) (address p) = Some n.
Proof.
  intros H. destruct (span_bounds _ _ _ H) as [P [O [Q [N E]]]].
  rewrite /lowered_distance unsigned_sub_exact; try lia.
  rewrite E /exact_udiv Z.mod_1_r.
  case_bool_decide; last lia. by rewrite Z.div_1_r.
Qed.
Lemma strict_interior_pad w k : window_valid w →
  0 < k → k+128 < window_extent w →
  span (InAllocation (Pointer w k))
       (InAllocation (shifted (Pointer w k) 128)) 128.
Proof.
  intros W K E. pose proof (interior128_offset w k W K E) as [O _].
  constructor. unfold offset_valid in O. unfold slice_valid. intuition lia.
Qed.
Lemma dangling_empty a : 1 ≤ a ≤ usize_max →
  span (EmptyAddress a) (EmptyAddress a) 0.
Proof. intros H. apply SpanEmpty. exact H. Qed.
Lemma empty_at_pad p : pointer_valid p →
  span (InAllocation p) (InAllocation p) 0.
Proof. intros H. apply SpanEmpty. exact H. Qed.
Lemma nonempty_same_origin p q n : span p q n → 0 < n →
  ∃ r, p = InAllocation r ∧ q = InAllocation (shifted r n) ∧ slice_valid r n.
Proof. intros H N. inversion H; subst; last lia. eauto. Qed.
Lemma distinct_dangling_rejected a b n : a ≠ b →
  ¬ span (EmptyAddress a) (EmptyAddress b) n.
Proof. intros N H. inversion H; subst. congruence. Qed.
Lemma wrong_distance_rejected p q n d : span p q n → d ≠ n →
  lowered_distance 64 1 (address q) (address p) ≠ Some d.
Proof. intros H N. rewrite (lowered_u8_exact _ _ _ H). congruence. Qed.
Lemma reversed_nonempty_rejected p q n : span p q n → 0 < n →
  unsigned_sub 64 (address p) (address q) = None.
Proof.
  intros H N. destruct (span_bounds _ _ _ H) as [P [O [Q [B E]]]].
  rewrite /unsigned_sub. case_bool_decide; first lia. done.
Qed.
Lemma wrong_width_rejected : unsigned_sub 32 4294967297 4294967296 = None.
Proof. vm_compute. reflexivity. Qed.
Lemma isize_overflow_rejected p q n : isize_max < n → ¬ span p q n.
Proof. intros N H. pose proof (span_bounds _ _ _ H). lia. Qed.
(* Reuse actual lrust Offset semantics and its checked simulation, not a copy. *)
Lemma formation_operational p n σ : slice_valid p n →
  head_step (BinOp OffsetOp #(pointer_loc p) #n) σ []
    #(pointer_loc (shifted p n)) σ [].
Proof.
  intros H. rewrite pointer_loc_shift.
  apply word_execution_step. constructor.
Qed.
Lemma subtraction_operational p q n σ : span p q n →
  head_step (BinOp MinusOp #(address q) #(address p)) σ [] #n σ [].
Proof.
  intros H. pose proof (span_bounds _ _ _ H) as [_ [_ [_ [_ E]]]].
  rewrite <- E. apply BinOpS. constructor.
Qed.
