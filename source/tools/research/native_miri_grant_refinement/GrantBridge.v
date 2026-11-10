(* Mathematical comparison only. No Rust proof term is imported here. *)
From stbor.lang Require Import bor_semantics.
From Coq Require Import Lia.
Local Open Scope nat_scope.

Definition succeeding_boundary (above : stack) (k : nat) : Prop :=
  k <= length above /\
  (forall j it, j < k -> above !! j = Some it -> it.(perm) = SharedReadWrite) /\
  (k = length above \/ exists it, above !! k = Some it /\ it.(perm) ≠ SharedReadWrite).

Lemma historical_srw_characterization above k :
  succeeding_boundary above k ->
  find_first_write_incompatible (reverse above) SharedReadWrite =
  Some (length above - k).
Proof.
  intros (Hbound & Hprefix & Hstop).
  unfold find_first_write_incompatible.
  rewrite reverse_involutive reverse_length.
  destruct Hstop as [Heq | [it [Hit Hbad]]].
  - subst k.
    assert (Hnone : list_find (fun it : item => perm it ≠ SharedReadWrite) above = None).
    { apply list_find_None. apply Forall_lookup. intros j it Hj Hbad.
      pose proof (lookup_lt_Some _ _ _ Hj) as Hlt.
      apply Hbad. eapply Hprefix; eauto. }
    rewrite Hnone. f_equal. lia.
  - assert (Hfind : list_find (fun it : item => perm it ≠ SharedReadWrite) above = Some (k,it)).
    { apply list_find_Some. split; [exact Hit|]. split; [exact Hbad|].
      intros j Hj. destruct (lookup_lt_is_Some_2 above j) as [x Hx]; [lia|].
      exists x. split; [exact Hx|]. intros Hne. apply Hne. eapply Hprefix; eauto. }
    rewrite Hfind. reflexivity.
Qed.

(* The same first-incompatible property as the Verus postcondition, expressed
   independently with total option lookup, over historical item values. *)
Definition vector_boundary (v : stack) (g : nat) (pm : permission) (r : nat) : Prop :=
  S g <= r /\ r <= length v /\
  match pm with
  | Unique => r = S g
  | SharedReadWrite =>
      (forall j it, g < j -> j < r -> v !! j = Some it -> it.(perm) = SharedReadWrite) /\
      (r = length v \/ exists it, v !! r = Some it /\ it.(perm) ≠ SharedReadWrite)
  | _ => False
  end.

Lemma vector_suffix_characterization v g r :
  vector_boundary v g SharedReadWrite r ->
  succeeding_boundary (drop (S g) v) (r - S g).
Proof.
  intros (Hlo & Hhi & Hprefix & Hstop). unfold succeeding_boundary.
  rewrite drop_length. split; [lia|]. split.
  - intros j it Hj Hit. rewrite lookup_drop in Hit.
    apply (Hprefix (S g + j) it); [lia|lia|exact Hit].
  - destruct Hstop as [Heq|[it [Hit Hbad]]].
    + left. lia.
    + right. exists it. split; [|exact Hbad].
      rewrite lookup_drop. replace (S g + (r - S g)) with r by lia. exact Hit.
Qed.

Lemma above_prefix_reversal (v : stack) n :
  take (length v - n) (reverse v) = reverse (drop n v).
Proof.
  replace (reverse v) with (reverse (drop n v) ++ reverse (take n v)).
  - apply take_app_alt. rewrite reverse_length drop_length. reflexivity.
  - rewrite <- reverse_app. rewrite take_drop. reflexivity.
Qed.

Theorem historical_vector_boundary_equality v g pm r :
  vector_boundary v g pm r ->
  find_first_write_incompatible
    (take (length v - 1 - g) (reverse v)) pm = Some (length v - r).
Proof.
  intros Hb.
  replace (length v - 1 - g) with (length v - S g) by lia.
  rewrite above_prefix_reversal.
  destruct pm.
  - destruct Hb as (Hlo & Hhi & Heq). subst r.
    unfold find_first_write_incompatible. rewrite reverse_length drop_length. reflexivity.
  - pose proof (vector_suffix_characterization v g r Hb) as Hsuffix.
    rewrite (historical_srw_characterization _ _ Hsuffix) drop_length.
    destruct Hb as (Hlo & Hhi & _). f_equal. lia.
  - destruct Hb as (_ & _ & Hfalse). contradiction.
  - destruct Hb as (_ & _ & Hfalse). contradiction.
Qed.

(* The granting-index reversal and prefix length do not depend on tags or
   protectors. Membership at the reversed granting index is stated separately. *)
Lemma granting_index_reversal (v : stack) g it :
  v !! g = Some it -> reverse v !! (length v - 1 - g) = Some it.
Proof.
  revert g. induction v as [|x xs IH]; intros [|g] Hit; simpl in Hit; try discriminate.
  - rewrite reverse_cons.
    change ((reverse xs ++ [x]) !! (S (length xs) - 1 - 0) = Some it).
    replace (S (length xs) - 1 - 0) with (length xs) by lia.
    assert (Hle : length (reverse xs) <= length xs) by (rewrite reverse_length; lia).
    rewrite (lookup_app_r _ _ _ Hle).
    rewrite reverse_length Nat.sub_diag. exact Hit.
  - pose proof (lookup_lt_Some _ _ _ Hit) as Hlt.
    rewrite reverse_cons.
    change ((reverse xs ++ [x]) !! (S (length xs) - 1 - S g) = Some it).
    assert (Hless : S (length xs) - 1 - S g < length (reverse xs)) by (rewrite reverse_length; lia).
    rewrite (lookup_app_l _ _ _ Hless).
    replace (S (length xs) - 1 - S g) with (length xs - 1 - g) by lia.
    apply IH. exact Hit.
Qed.

Print Assumptions succeeding_boundary.
Print Assumptions historical_srw_characterization.
Print Assumptions vector_boundary.
Print Assumptions vector_suffix_characterization.
Print Assumptions above_prefix_reversal.
Print Assumptions historical_vector_boundary_equality.
Print Assumptions granting_index_reversal.
