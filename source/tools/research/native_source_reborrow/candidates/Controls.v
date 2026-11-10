From Coq Require Import List ZArith Bool Arith.PeanoNat Lia.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage Prefix.
From NativeSourceReborrow Require Import Model Laws Span.
Import ListNotations.
Local Open Scope nat_scope.

Definition interior_ref := Reference 7 17%Z 11.
Definition interior_nodes n := [Node 11 (Some 5) 7 17%Z n; Node 5 None 7 0%Z (n+64)].
Definition interior_grants := [Grant 11 Active false; Grant 5 Active true].
Lemma interior_lineage n j : j < n ->
  valid_suffix (interior_nodes n) (ref_address interior_ref j) interior_grants = true.
Proof.
  intro J. unfold interior_nodes,interior_grants,interior_ref,valid_suffix,in_node,ref_address.
  cbn -[Z.add Z.of_nat Z.leb Z.ltb Nat.add].
  repeat rewrite andb_true_iff. repeat split; try reflexivity; try apply Z.leb_le; try apply Z.ltb_lt; lia.
Qed.
Lemma interior_byte_ready n j : j < n ->
  byte_ready (interior_nodes n) (ref_address interior_ref j) 11 42 interior_grants.
Proof.
  intro J. exists [],(Grant 11 Active false),[Grant 5 Active true].
  repeat split; try reflexivity; try constructor. exact (interior_lineage n j J).
Qed.
Definition interior_history {H} (prior : H) n mem old_trace : history H :=
  History prior [Region 7 4096%Z (n+64) true] (interior_nodes n) mem 42 old_trace.
Theorem arbitrary_interior_context H (prior : H) n mem old_trace caller stack fresh trace :
  n <> 0 -> (Z.of_nat n <= 2^64 - 4160)%Z ->
  fresh_trace 42 old_trace = true ->
  (forall j, j < n -> mem (ref_address interior_ref j) = interior_grants) ->
  ready (prefix_three (root_entry interior_ref caller
    (interior_history prior n mem old_trace) stack fresh trace)) interior_ref n.
Proof.
  intros N Bound Old Bytes. apply prefix_ready;[exact N| | |].
  - unfold interior_history,span_ok,interior_ref; cbn -[Z.add Z.of_nat Z.leb Z.ltb Nat.add Z.pow].
    rewrite orb_false_r. repeat rewrite andb_true_iff. repeat split; try reflexivity; try apply Z.leb_le; try apply Z.ltb_lt; lia.
  - change (true && fresh_trace 42 old_trace = true). exact Old.
  - intros a I. apply addresses_member in I. destruct I as [j [J E]]. subst a.
    cbn. rewrite (Bytes j J). exact (interior_byte_ready n j J).
Qed.
Theorem protected_live_ancestor_supported n j : j < n ->
  grant_byte (interior_nodes n) (ref_address interior_ref j) 11 42 interior_grants =
    Some (Grant 42 Reserved false :: interior_grants).
Proof.
  intro J. unfold grant_byte. change (seek_parent (interior_nodes n)
    (ref_address interior_ref j) 11 42 interior_grants = Some (Grant 42 Reserved false :: interior_grants)).
  change ((if valid_suffix (interior_nodes n) (ref_address interior_ref j) interior_grants
    then Some (Grant 42 Reserved false::interior_grants) else None) = Some (Grant 42 Reserved false :: interior_grants)).
  rewrite (interior_lineage n j J);reflexivity.
Qed.
Lemma protected_younger_rejected ns a p c x tail :
  grant_tag x <> p -> protected x = true ->
  grant_byte ns a p c (x::tail) = None.
Proof.
  intros N P. unfold grant_byte. destruct (no_tag c (x::tail));[|reflexivity].
  cbn -[valid_suffix]. assert (E : Nat.eqb p (grant_tag x) = false) by (apply Nat.eqb_neq;congruence).
  rewrite E,P;reflexivity.
Qed.
Lemma disabled_parent_rejected ns a p c prot tail :
  grant_byte ns a p c (Grant p Disabled prot::tail) = None.
Proof.
  unfold grant_byte. destruct (no_tag c (Grant p Disabled prot::tail));[|reflexivity].
  cbn -[valid_suffix]. rewrite Nat.eqb_refl.
  cbn -[in_node]. destruct (find_node p ns);[|reflexivity].
  cbn -[in_node]. rewrite andb_false_r. reflexivity.
Qed.
Lemma missing_parent_rejected ns a p c : grant_byte ns a p c [] = None.
Proof. reflexivity. Qed.
Lemma reused_child_rejected ns a p c perm prot tail :
  grant_byte ns a p c (Grant c perm prot::tail) = None.
Proof. unfold grant_byte,no_tag;cbn. rewrite Nat.eqb_refl;reflexivity. Qed.
Theorem protected_disabled_ancestor_rejected n j :
  grant_byte (interior_nodes n) (ref_address interior_ref j) 11 42
    [Grant 11 Active false; Grant 5 Disabled true] = None.
Proof. unfold grant_byte,seek_parent,valid_suffix; cbn -[in_node].
  repeat rewrite andb_false_r; reflexivity. Qed.
Theorem null_address_rejected H (prior : H) n : n <> 0 ->
  span_ok [Region 7 0%Z (n+64) true] (Reference 7 0%Z 11) n = false.
Proof. intro N. unfold span_ok; cbn. repeat rewrite andb_false_r; reflexivity. Qed.
Theorem unsize_remains_unsupported H (s : state (history H)) n :
  current_event s = MirOperation 0 0 (Some 4) ->
  Model.execute s (MirOperation 0 0 (Some 4)) n = Model.Unsupported (MirOperation 0 0 (Some 4)).
Proof. intro C. apply unsupported_other_site;[exact C|discriminate]. Qed.
