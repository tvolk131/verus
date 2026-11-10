From Coq Require Import List ZArith Bool Arith.PeanoNat Lia.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage Prefix.
From NativeSourceReborrow Require Import Model Laws.
Import ListNotations.
Local Open Scope nat_scope.

Lemma prefix_ready H r caller (h : history H) stack fresh trace n :
  n <> 0 -> span_ok (regions h) r n = true ->
  (fresh_node (nodes h) (next_tag h) && fresh_trace (next_tag h) (reborrow_trace h)) = true ->
  (forall a, In a (addresses r n) ->
    byte_ready (nodes h) a (provenance_tag r) (next_tag h) (byte_grants h a)) ->
  ready (prefix_three (root_entry r caller h stack fresh trace)) r n.
Proof. intros N R F B. repeat split; try assumption; try reflexivity. Qed.
Theorem four_step_model_prefix H r caller (h : history H) stack fresh trace n :
  n <> 0 -> span_ok (regions h) r n = true ->
  (fresh_node (nodes h) (next_tag h) && fresh_trace (next_tag h) (reborrow_trace h)) = true ->
  (forall a, In a (addresses r n) ->
    byte_ready (nodes h) a (provenance_tag r) (next_tag h) (byte_grants h a)) ->
  let s0 := root_entry r caller h stack fresh trace in
  let s := prefix_three s0 in
  Prefix.execute s0 (MirOperation 0 0 (Some 0)) = Prefix.Advanced (prefix_one s0) /\
  Prefix.execute (prefix_one s0) (MirOperation 0 0 (Some 1)) = Prefix.Advanced (prefix_two s0) /\
  Prefix.execute (prefix_two s0) (MirOperation 0 0 (Some 2)) = Prefix.Advanced s /\
  exists t, Model.execute s selected_site n = Model.Advanced t /\
    pc t = after_site /\
    Storage.operand_reference t 4 = Some (child_reference r (next_tag h)) /\
    Storage.operand_reference t 1 = Some r /\
    (forall a, caller_cells t a = caller a) /\
    prior_history (incoming_history t) = prior_history h.
Proof.
  intros N R F B. cbv zeta. repeat split; try reflexivity.
  pose proof (prefix_ready H r caller h stack fresh trace n N R F B) as Ready.
  destruct (model_progress H _ r n Ready) as [u [P E]].
  eexists; split; [exact E|]. repeat split; reflexivity.
Qed.
Theorem generic_span_bytes H (s : state (history H)) r n A : ready s r n ->
  length A = n ->
  (forall j b, nth_error A j = Some b -> caller_cells s (ref_address r j) = Some (InitByte b)) ->
  exists t, Model.execute s selected_site n = Model.Advanced t /\
    pc t = after_site /\
    (forall j b, nth_error A j = Some b -> caller_map t (ref_address r j) = Some b) /\
    (forall a, caller_cells t a = caller_cells s a).
Proof.
  intros R Len Bytes. destruct (model_progress H s r n R) as [u [P E]].
  eexists. split; [exact E|]. split.
  - exact (proj1 (all_advanced_control_local H _ _ _ _ E)).
  - split.
    + intros j b J. unfold caller_map. cbn. rewrite (Bytes j b J); reflexivity.
    + intro a; reflexivity.
Qed.
Theorem ten_byte_model H (s : state (history H)) r A : ready s r 10 ->
  length A = 10 ->
  (forall j b, nth_error A j = Some b -> caller_cells s (ref_address r j) = Some (InitByte b)) ->
  exists t, Model.execute s selected_site 10 = Model.Advanced t /\ pc t = after_site /\
    (forall j b, nth_error A j = Some b -> caller_map t (ref_address r j) = Some b) /\
    (forall a, caller_cells t a = caller_cells s a).
Proof. apply generic_span_bytes. Qed.
Theorem pad128_model H (s : state (history H)) r A : ready s r 128 ->
  length A = 128 ->
  (forall j b, nth_error A j = Some b -> caller_cells s (ref_address r j) = Some (InitByte b)) ->
  exists t, Model.execute s selected_site 128 = Model.Advanced t /\ pc t = after_site /\
    (forall j b, nth_error A j = Some b -> caller_map t (ref_address r j) = Some b) /\
    (forall a, caller_cells t a = caller_cells s a).
Proof. apply generic_span_bytes. Qed.
Lemma new_node_extent r n c j : j < n ->
  in_node (Node c (Some (provenance_tag r)) (allocation r) (byte_offset r) n) (ref_address r j) = true.
Proof.
  intro J. unfold in_node,ref_address; simpl. rewrite Nat.eqb_refl. simpl.
  apply andb_true_iff. split;[apply Z.leb_le|apply Z.ltb_lt]; lia.
Qed.
Lemma valid_suffix_extend ns a c x gs :
  node_tag x = c -> no_tag c gs = true ->
  valid_suffix ns a gs = true -> valid_suffix (x::ns) a gs = true.
Proof.
  revert x. induction gs as [|g tail IH]; [discriminate|].
  intros x X N V. unfold no_tag in N. simpl in N. apply andb_true_iff in N.
  destruct N as [N T]. apply negb_true_iff in N.
  cbn [valid_suffix find_node]. rewrite X,Nat.eqb_sym,N.
  cbn [valid_suffix] in V.
  destruct (find_node (grant_tag g) ns) as [nd|] eqn:D;[|discriminate].
  apply andb_true_iff in V. destruct V as [V Rest].
  apply andb_true_iff. split;[exact V|].
  destruct (parent_tag nd) as [p|],tail as [|h hs]; try discriminate; try reflexivity.
  apply andb_true_iff in Rest. destruct Rest as [P Q]. rewrite P. simpl.
  apply (IH x X T Q).
Qed.
Lemma grant_child_valid ns r n c a gs out :
  in_node (Node c (Some (provenance_tag r)) (allocation r) (byte_offset r) n) a = true ->
  grant_byte ns a (provenance_tag r) c gs = Some out ->
  valid_suffix (Node c (Some (provenance_tag r)) (allocation r) (byte_offset r) n::ns) a out = true.
Proof.
  intros I G. destruct (grant_retains_suffix _ _ _ _ _ _ G)
    as [N [young [g [anc [E [T [Y [V O]]]]]]]].
  subst gs out. unfold no_tag in N. rewrite forallb_app in N.
  apply andb_true_iff in N. destruct N as [_ N].
  cbn [valid_suffix find_node]. rewrite Nat.eqb_refl.
  change (in_node (Node c (Some (provenance_tag r)) (allocation r) (byte_offset r) n) a &&
    true && (Nat.eqb (provenance_tag r) (grant_tag g) &&
      valid_suffix (Node c (Some (provenance_tag r)) (allocation r) (byte_offset r) n::ns) a (g::anc)) = true).
  rewrite I,T,Nat.eqb_refl.
  change (valid_suffix (Node c (Some (provenance_tag r)) (allocation r) (byte_offset r) n::ns) a (g::anc) = true).
  eapply valid_suffix_extend; eauto.
Qed.
Theorem advanced_child_valid H (s t : state (history H)) e n j :
  Model.execute s e n = Model.Advanced t -> j < n ->
  valid_suffix (nodes (incoming_history t)) (ref_address (original_operand s) j)
    (byte_grants (incoming_history t) (ref_address (original_operand s) j)) = true.
Proof.
  intros E J. destruct (all_advanced_shape H s t e n E)
    as [r [u [_ [_ [_ [O [_ [_ [_ [_ [P T]]]]]]]]]]]. subst t. rewrite O.
  apply (grant_child_valid (nodes (incoming_history s)) r n (next_tag (incoming_history s))
    (ref_address r j) (byte_grants (incoming_history s) (ref_address r j))).
  - apply new_node_extent;exact J.
  - apply commit_per_byte;[exact P|]. apply addresses_member. exists j;split;[exact J|reflexivity].
Qed.
