From lrust.lang Require Export notation.
From stdpp Require Import gmap.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition isize_max : Z := 9223372036854775807.
Definition usize_max : Z := 18446744073709551615.
(* Native numeric addresses and model locations are related, not identified.
   The model block is the containing allocation; base may have any model offset. *)
Inductive window :=
| Window (base : loc) (address extent : Z).
Definition window_base w := match w with Window b _ _ => b end.
Definition window_address w := match w with Window _ a _ => a end.
Definition window_extent w := match w with Window _ _ n => n end.
Definition window_valid w : Prop :=
  1 ≤ window_address w ∧ 0 ≤ window_extent w ≤ isize_max ∧
  window_address w + window_extent w ≤ usize_max.
Inductive pointer :=
| Pointer (allocation : window) (cursor : Z).
Definition pointer_window p := match p with Pointer w _ => w end.
Definition pointer_cursor p := match p with Pointer _ k => k end.
Definition pointer_loc p : loc := window_base (pointer_window p) +ₗ pointer_cursor p.
Definition pointer_address p := window_address (pointer_window p) + pointer_cursor p.
Definition pointer_valid p : Prop := window_valid (pointer_window p) ∧
  0 ≤ pointer_cursor p ≤ window_extent (pointer_window p).
Definition slice_valid p n : Prop := pointer_valid p ∧
  0 ≤ n ∧ pointer_cursor p + n ≤ window_extent (pointer_window p).
Definition shifted p n := Pointer (pointer_window p) (pointer_cursor p + n).
Definition offset_valid p n : Prop := pointer_valid p ∧
  0 ≤ n ≤ isize_max ∧ pointer_cursor p + n ≤ window_extent (pointer_window p).

(* Descriptors contain no ty_own, points-to, prophecy or borrow authority. *)
Inductive descriptor :=
| SliceDesc (p : pointer) (n : Z)
| RawWide (p : pointer) (n : Z)
| NonNullWide (p : pointer) (n : Z)
| RawThin (p : pointer)
| NonNullThin (p : pointer)
| Usize (n : Z)
| Boolean (b : bool).
Inductive unsupported :=
| RawReferenceCreation
| Retag
| Reservation
| Activation
| FakeRead
| UnsizeReference
| StorageLifetime
| AggregateOwnership
| CallReturn
| Unwind
| Drop
| SourcePhase
| IntegerToPointer.
Inductive operation :=
| Metadata
| WrapWide
| UnwrapWide
| Thin
| WrapThin
| UnwrapThin
| ByteOffset
| U8ZstBranch
| Unsupported (why : unsupported).

Inductive source_step : operation → list descriptor → state → list descriptor → state → Prop :=
| StepMetadata p n σ : slice_valid p n →
    source_step Metadata [SliceDesc p n] σ [Usize n] σ
| StepWrapWide p n σ : slice_valid p n →
    source_step WrapWide [RawWide p n] σ [NonNullWide p n] σ
| StepUnwrapWide p n σ : slice_valid p n →
    source_step UnwrapWide [NonNullWide p n] σ [RawWide p n] σ
| StepThin p n σ : slice_valid p n →
    source_step Thin [RawWide p n] σ [RawThin p] σ
| StepWrapThin p σ : pointer_valid p →
    source_step WrapThin [RawThin p] σ [NonNullThin p] σ
| StepUnwrapThin p σ : pointer_valid p →
    source_step UnwrapThin [NonNullThin p] σ [RawThin p] σ
| StepByteOffset p n σ : offset_valid p n →
    source_step ByteOffset [RawThin p; Usize n] σ [RawThin (shifted p n)] σ
| StepU8ZstBranch σ :
    source_step U8ZstBranch [Boolean false] σ [Usize 2] σ.

Definition descriptor_words d : list val :=
  match d with
  | SliceDesc p n | RawWide p n | NonNullWide p n => (LitV (LitLoc (pointer_loc p)) :: LitV (LitInt n) :: nil)
  | RawThin p | NonNullThin p => (LitV (LitLoc (pointer_loc p)) :: nil)
  | Usize n => (LitV (LitInt n) :: nil)
  | Boolean b => (LitV (LitInt (Z_of_bool b)) :: nil)
  end.
Definition input_words ds := concat (descriptor_words <$> ds).

(* Projection really consumes the represented input words as beta operands. *)
Definition select_data (a b : val) : expr :=
  App (Rec BAnon [BNamed "data"; BAnon] (Var "data")) [of_val a; of_val b].
Definition select_len (a b : val) : expr :=
  App (Rec BAnon [BAnon; BNamed "len"] (Var "len")) [of_val a; of_val b].
Definition select_thin (a : val) : expr :=
  App (Rec BAnon [BNamed "data"] (Var "data")) [of_val a].
Definition false_branch : expr := Case #0 (Lit (LitInt 2) :: Lit (LitInt 1) :: nil).
Definition translate op (ws : list val) : option (list expr) :=
  match op, ws with
  | Metadata, [a;b] => Some [select_len a b]
  | WrapWide, [a;b] | UnwrapWide, [a;b] => Some [select_data a b; select_len a b]
  | Thin, [a;b] => Some [select_data a b]
  | WrapThin, [a] | UnwrapThin, [a] => Some [select_thin a]
  | ByteOffset, [a;b] => Some [BinOp OffsetOp (of_val a) (of_val b)]
  | U8ZstBranch, [LitV (LitInt 0)] => Some [false_branch]
  | _, _ => None
  end.

(* This auxiliary relation is proved against head_step in simulation.v. *)
Inductive word_execution : expr → val → Prop :=
| ExecuteData a b : word_execution (select_data a b) a
| ExecuteLen a b : word_execution (select_len a b) b
| ExecuteThin a : word_execution (select_thin a) a
| ExecuteOffset (l : loc) (n : Z) : word_execution (BinOp OffsetOp #l #n) #(l +ₗ n)
| ExecuteFalse : word_execution false_branch #2.

Lemma pointer_loc_shift p n : pointer_loc (shifted p n) = pointer_loc p +ₗ n.
Proof. destruct p as [[b a e] k]. rewrite /pointer_loc /shifted /= /shift_loc /=. f_equal; lia. Qed.

Theorem supported_operation_simulation op ds σ outs σ' :
  source_step op ds σ outs σ' →
  σ' = σ ∧ ∃ es, translate op (input_words ds) = Some es ∧
    Forall2 word_execution es (input_words outs).
Proof.
  intros Step. inversion Step; subst; split; first done;
    eexists; split; first reflexivity;
    rewrite /input_words /descriptor_words /=; try rewrite pointer_loc_shift;
    repeat constructor.
Qed.

Lemma unsupported_rejected why ds σ outs σ' :
  ¬ source_step (Unsupported why) ds σ outs σ'.
Proof. intros H. inversion H. Qed.
Lemma unsupported_translation_rejected why ws :
  translate (Unsupported why) ws = None.
Proof. destruct ws as [|a [|b [|c ws]]]; done. Qed.
Lemma offset_result_bounds p n : offset_valid p n →
  pointer_valid (shifted p n) ∧
  pointer_address (shifted p n) ≤ usize_max ∧
  1 ≤ pointer_address (shifted p n) ∧
  (pointer_loc (shifted p n)).1 = (pointer_loc p).1.
Proof.
  destruct p as [[b a e] k].
  rewrite /offset_valid /pointer_valid /window_valid /shifted
    /pointer_window /pointer_cursor /window_address /window_extent
    /pointer_address /pointer_loc /window_base /shift_loc /=.
  intros H. intuition lia.
Qed.
Lemma interior128_offset w start : window_valid w →
  0 < start → start + 128 < window_extent w →
  offset_valid (Pointer w start) 128 ∧
  pointer_loc (shifted (Pointer w start) 128) =
    window_base w +ₗ (start + 128).
Proof.
  intros W S E. split; last done.
  rewrite /offset_valid /pointer_valid /pointer_window /pointer_cursor /=.
  split; first (split; [done|lia]). split; [rewrite /isize_max; lia|lia].
Qed.
Definition array_dereferenceable (start i : Z) : Prop := start ≤ i < start + 128.
Lemma one_past_not_array_dereferenceable start :
  ¬ array_dereferenceable start (start + 128).
Proof. rewrite /array_dereferenceable. lia. Qed.
Lemma wrong_kind_no_metadata p σ outs σ' :
  ¬ source_step Metadata [RawThin p] σ outs σ'.
Proof. intros H. inversion H. Qed.
Lemma offset_outside_rejected w k n σ outs σ' :
  window_extent w < k+n →
  ¬ source_step ByteOffset [RawThin (Pointer w k); Usize n] σ outs σ'.
Proof. intros H Step. inversion Step; subst. unfold offset_valid in *. simpl in *. lia. Qed.
