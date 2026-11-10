From NativePointerDistance Require Export distance.
Set Default Proof Using "Type".
Open Scope Z_scope.

Inductive pc :=
| Entry
| OrderCheck
| PositiveSize
| UpperSize
| Intrinsic
| MakeTuple
| Returned
| ZstRejected
| UnsafeFailure
| AssertFailure.
(* Scalar/control fragment: field reads/casts/source phases are outside this relation. *)
Inductive chain_step (ub zst : bool) (size a b : Z) : pc → pc → Prop :=
| CEntry : zst = false → chain_step ub zst size a b Entry
    (if ub then OrderCheck else PositiveSize)
| CZst : zst = true → chain_step ub zst size a b Entry ZstRejected
| COrder : b ≤ a → chain_step ub zst size a b OrderCheck PositiveSize
| COrderFail : a < b → chain_step ub zst size a b OrderCheck UnsafeFailure
| CPositive : 0 < size → chain_step ub zst size a b PositiveSize UpperSize
| CPositiveFail : size ≤ 0 → chain_step ub zst size a b PositiveSize AssertFailure
| CUpper : size ≤ isize_max → chain_step ub zst size a b UpperSize Intrinsic
| CUpperFail : isize_max < size → chain_step ub zst size a b UpperSize AssertFailure
| CIntrinsic d : lowered_distance 64 size a b = Some d →
    chain_step ub zst size a b Intrinsic MakeTuple
| CTuple : chain_step ub zst size a b MakeTuple Returned.
Definition path (ub zst : bool) (size a b : Z) (pcs : list pc) : Prop :=
  Forall (λ xy, chain_step ub zst size a b xy.1 xy.2)
    (zip pcs (tail pcs)).
Definition u8_path (ub : bool) : list pc :=
  [Entry] ++ (if ub then [OrderCheck] else []) ++
  [PositiveSize; UpperSize; Intrinsic; MakeTuple; Returned].
Definition hint n : Z * option Z := (n,Some n).
Definition computed_hint start finish : option (Z * option Z) :=
  match lowered_distance 64 1 (address finish) (address start) with
  | Some d => Some (hint d) | None => None end.
Definition native_min a b := if bool_decide (b < a) then b else a.
Definition zip_record (a b : endpoint) (n : Z) := (a,b,0,n).

Theorem u8_branch_path ub p q n : span p q n →
  path ub false 1 (address q) (address p) (u8_path ub).
Proof.
  intros H. destruct (span_bounds _ _ _ H) as [P [O [Q [N E]]]].
  unfold path, u8_path. destruct ub; simpl;
  repeat constructor; try done; try (rewrite /isize_max; lia).
  all: eapply CIntrinsic; apply lowered_u8_exact; exact H.
Qed.
Theorem failure_exclusion ub p q n from bad : span p q n →
  bad = ZstRejected ∨ bad = UnsafeFailure ∨ bad = AssertFailure →
  ¬ chain_step ub false 1 (address q) (address p) from bad.
Proof.
  intros H B S. pose proof (span_bounds _ _ _ H) as Bounds.
  destruct ub; destruct B as [B|[B|B]]; subst bad; inversion S; subst;
    try discriminate; unfold isize_max in *; lia.
Qed.
Lemma u8_hint_exact p q n : span p q n →
  (match lowered_distance 64 1 (address q) (address p) with
   | Some d => Some (hint d) | None => None end) = Some (n,Some n).
Proof. intros H. rewrite (lowered_u8_exact _ _ _ H). reflexivity. Qed.
Lemma trusted_size_projection n : (hint n).1 = n.
Proof. reflexivity. Qed.
Lemma native_min_exact a b : native_min a b = Z.min a b.
Proof.
  unfold native_min. case_bool_decide.
  - rewrite Z.min_r; lia.
  - rewrite Z.min_l; lia.
Qed.
Theorem specialized_zip_length pad pend key kend n :
  span pad pend 128 → span key kend n →
  ∃ ph kh, computed_hint pad pend = Some ph ∧ computed_hint key kend = Some kh ∧
  ph = hint 128 ∧ kh = hint n ∧
  native_min ph.1 kh.1 = Z.min 128 n ∧
  0 ≤ native_min ph.1 kh.1 ≤ 128 ∧ native_min ph.1 kh.1 ≤ n ∧
  zip_record pad key (native_min ph.1 kh.1) = (pad,key,0,Z.min 128 n).
Proof.
  intros P K. pose proof (span_bounds _ _ _ K) as [_ [_ [_ [N _]]]].
  exists (hint 128), (hint n).
  rewrite /computed_hint (lowered_u8_exact _ _ _ P) (lowered_u8_exact _ _ _ K).
  simpl. rewrite native_min_exact.
  repeat split; try done; lia.
Qed.
Lemma zip_empty_key pad key : zip_record pad key (native_min 128 0) = (pad,key,0,0).
Proof. rewrite native_min_exact. reflexivity. Qed.
Lemma zip_long_key pad key n : 128 < n →
  zip_record pad key (native_min 128 n) = (pad,key,0,128).
Proof. intros H. rewrite native_min_exact Z.min_l; [done|lia]. Qed.
Lemma wrong_u8_ctfe_rejected ub size a b :
  ¬ chain_step ub true size a b Entry (if ub then OrderCheck else PositiveSize).
Proof. intros H. inversion H; subst; destruct ub; discriminate. Qed.
Lemma unsafe_failure_requires_wrong_order ub zst size a b from :
  chain_step ub zst size a b from UnsafeFailure → a < b.
Proof. destruct ub; intros H; inversion H; subst; done. Qed.
