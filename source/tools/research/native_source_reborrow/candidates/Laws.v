From Coq Require Import List ZArith Bool Arith.PeanoNat Lia.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage Prefix.
From NativeSourceReborrow Require Import Model.
Import ListNotations.
Local Open Scope nat_scope.

Lemma seek_progress ns a parent child younger g ancestors :
  grant_tag g = parent ->
  Forall (fun x => protected x = false /\ grant_tag x <> parent) younger ->
  valid_suffix ns a (g::ancestors) = true ->
  seek_parent ns a parent child (younger ++ g::ancestors) =
    Some (Grant child Reserved false :: g::ancestors).
Proof.
  intros G Y V. induction Y as [|x xs [P N] F IH].
  - cbn -[valid_suffix]. rewrite G, Nat.eqb_refl, V. reflexivity.
  - cbn -[valid_suffix]. rewrite P. assert (E : Nat.eqb parent (grant_tag x) = false).
    { apply Nat.eqb_neq. congruence. } rewrite E. exact IH.
Qed.
Lemma grant_progress ns a parent child gs : byte_ready ns a parent child gs ->
  exists suffix, grant_byte ns a parent child gs = Some (Grant child Reserved false::suffix).
Proof.
  intros [young [g [anc [E [G [Y [V N]]]]]]]. subst gs.
  unfold grant_byte. rewrite N, (seek_progress ns a parent child young g anc G Y V).
  eexists; reflexivity.
Qed.
Lemma seek_retains_suffix ns a p c gs out : seek_parent ns a p c gs = Some out ->
  exists younger g ancestors,
    gs = younger ++ g::ancestors /\ grant_tag g = p /\
    Forall (fun x => protected x = false /\ grant_tag x <> p) younger /\
    valid_suffix ns a (g::ancestors) = true /\
    out = Grant c Reserved false :: g::ancestors.
Proof.
  induction gs as [|x xs IH]; cbn -[valid_suffix]; [discriminate|].
  destruct (Nat.eqb p (grant_tag x)) eqn:E.
  - destruct (valid_suffix ns a (x::xs)) eqn:V; [|discriminate].
    intro O; inversion O; subst. exists [],x,xs. cbn -[valid_suffix].
    apply Nat.eqb_eq in E. repeat split; try congruence; apply Forall_nil.
  - destruct (protected x) eqn:P; [discriminate|].
    intro O. destruct (IH O) as [young [g [anc [G [T [Y [V R]]]]]]].
    exists (x::young),g,anc. cbn -[valid_suffix]. repeat split; try congruence.
    constructor; [split; [exact P|apply Nat.eqb_neq in E; congruence]|exact Y].
Qed.
Lemma grant_retains_suffix ns a p c gs out : grant_byte ns a p c gs = Some out ->
  no_tag c gs = true /\
  exists younger g ancestors,
    gs = younger ++ g::ancestors /\ grant_tag g = p /\
    Forall (fun x => protected x = false /\ grant_tag x <> p) younger /\
    valid_suffix ns a (g::ancestors) = true /\
    out = Grant c Reserved false :: g::ancestors.
Proof.
  unfold grant_byte. destruct (no_tag c gs) eqn:N; [|discriminate].
  intro E. split; [reflexivity|]. exact (seek_retains_suffix ns a p c gs out E).
Qed.
Lemma protected_grant_preserved ns a p c gs out g :
  grant_byte ns a p c gs = Some out -> In g gs -> protected g = true -> In g out.
Proof.
  intros E I P. destruct (grant_retains_suffix ns a p c gs out E)
    as [_ [young [h [anc [G [_ [Y [_ O]]]]]]]].
  subst gs out. apply in_app_or in I. destruct I as [I|I]; [|right;exact I].
  rewrite Forall_forall in Y. specialize (Y g I). destruct Y as [N _]. congruence.
Qed.
Lemma prepare_progress ns p c mem as_ :
  (forall a, In a as_ -> byte_ready ns a p c (mem a)) ->
  exists updates, prepare ns p c mem as_ = Some updates.
Proof.
  induction as_ as [|a tail IH]; intro R; simpl; [exists [];reflexivity|].
  destruct (grant_progress ns a p c (mem a) (R a (or_introl eq_refl))) as [gs G].
  destruct (IH (fun a I => R a (or_intror I))) as [rest E].
  rewrite G,E. eexists; reflexivity.
Qed.
Lemma prepare_keys ns p c mem as_ updates : prepare ns p c mem as_ = Some updates ->
  map fst updates = as_.
Proof.
  revert updates. induction as_ as [|a tail IH]; simpl.
  - intros u E; inversion E; reflexivity.
  - destruct (grant_byte ns a p c (mem a)); [|discriminate].
    destruct (prepare ns p c mem tail) eqn:R; [|discriminate].
    intros u E; inversion E; subst. simpl. f_equal. exact (IH _ eq_refl).
Qed.
Lemma prepare_member ns p c mem as_ updates a gs :
  prepare ns p c mem as_ = Some updates -> In (a,gs) updates ->
  grant_byte ns a p c (mem a) = Some gs.
Proof.
  revert updates. induction as_ as [|b tail IH]; simpl.
  - intros u E; inversion E; contradiction.
  - destruct (grant_byte ns b p c (mem b)) eqn:G; [|discriminate].
    destruct (prepare ns p c mem tail) eqn:R; [|discriminate].
    intros u E I; inversion E; subst. destruct I as [I|I].
    + inversion I; subst. exact G.
    + exact (IH _ eq_refl I).
Qed.
Lemma install_frame updates old a : ~ In a (map fst updates) -> install updates old a = old a.
Proof.
  induction updates as [|[b gs] tail IH]; simpl; [reflexivity|].
  intro N. destruct (address_eq_dec a b); [subst;exfalso;apply N;left;reflexivity|].
  apply IH. intro I;apply N;right;exact I.
Qed.
Lemma install_member updates old a : In a (map fst updates) ->
  exists gs, In (a,gs) updates /\ install updates old a = gs.
Proof.
  induction updates as [|[b v] tail IH]; simpl; [contradiction|].
  intro I. destruct (address_eq_dec a b) as [E|N].
  - subst. exists v. split; [left;reflexivity|reflexivity].
  - destruct I as [I|I]; [congruence|]. destruct (IH I) as [gs [M E]].
    exists gs. split; [right;exact M|exact E].
Qed.
Lemma addresses_member r n a : In a (addresses r n) <->
  exists j, j < n /\ a = ref_address r j.
Proof.
  unfold addresses. rewrite in_map_iff. split.
  - intros [j [E I]]. apply in_seq in I. exists j. split;[lia|congruence].
  - intros [j [N E]]. exists j. split;[congruence|apply in_seq;lia].
Qed.
Lemma fresh_node_absent ns c : fresh_node ns c = true -> find_node c ns = None.
Proof.
  induction ns as [|x xs IH]; simpl; [reflexivity|].
  rewrite andb_true_iff. intros [N F]. apply negb_true_iff in N.
  rewrite N. apply IH; exact F.
Qed.
Lemma commit_storage_frame H (s : state (history H)) r n updates :
  let t := commit_reborrow s r n updates in
  caller_cells t = caller_cells s /\ caller_map t = caller_map s /\
  original_operand t = original_operand s /\ private_stack t = private_stack s /\
  fresh_stack t = fresh_stack s /\
  prior_history (incoming_history t) = prior_history (incoming_history s) /\
  regions (incoming_history t) = regions (incoming_history s).
Proof. repeat split; reflexivity. Qed.
Lemma commit_local H (s : state (history H)) r n updates :
  Storage.operand_reference (commit_reborrow s r n updates) 4 =
    Some (child_reference r (next_tag (incoming_history s))).
Proof. reflexivity. Qed.
Lemma commit_other_local H (s : state (history H)) r n updates l : l <> 4 ->
  locals (commit_reborrow s r n updates) l = locals s l.
Proof. intro N. simpl. apply update_other; exact N. Qed.
Lemma commit_nodes H (s : state (history H)) r n updates tag :
  tag <> next_tag (incoming_history s) ->
  find_node tag (nodes (incoming_history (commit_reborrow s r n updates))) =
    find_node tag (nodes (incoming_history s)).
Proof. intro N. simpl. apply Nat.eqb_neq in N. rewrite N; reflexivity. Qed.
Lemma commit_new_node H (s : state (history H)) r n updates :
  find_node (next_tag (incoming_history s))
    (nodes (incoming_history (commit_reborrow s r n updates))) =
  Some (Node (next_tag (incoming_history s)) (Some (provenance_tag r))
    (allocation r) (byte_offset r) n).
Proof. simpl. rewrite Nat.eqb_refl; reflexivity. Qed.
Lemma commit_grant_frame H (s : state (history H)) r n updates a :
  prepare (nodes (incoming_history s)) (provenance_tag r) (next_tag (incoming_history s))
    (byte_grants (incoming_history s)) (addresses r n) = Some updates ->
  ~ In a (addresses r n) ->
  byte_grants (incoming_history (commit_reborrow s r n updates)) a =
    byte_grants (incoming_history s) a.
Proof.
  intros E N. simpl. apply install_frame. rewrite (prepare_keys _ _ _ _ _ _ E). exact N.
Qed.
Lemma commit_per_byte H (s : state (history H)) r n updates a :
  prepare (nodes (incoming_history s)) (provenance_tag r) (next_tag (incoming_history s))
    (byte_grants (incoming_history s)) (addresses r n) = Some updates ->
  In a (addresses r n) ->
  grant_byte (nodes (incoming_history s)) a (provenance_tag r) (next_tag (incoming_history s))
    (byte_grants (incoming_history s) a) =
  Some (byte_grants (incoming_history (commit_reborrow s r n updates)) a).
Proof.
  intros E I. rewrite <- (prepare_keys _ _ _ _ _ _ E) in I.
  destruct (install_member updates (byte_grants (incoming_history s)) a I) as [gs [M G]].
  simpl. rewrite G. exact (prepare_member _ _ _ _ _ _ a gs E M).
Qed.

(* Current-state conditions are structural; no desired output, certificate,
   ty_own, or defined-future premise occurs here. *)
Definition ready {H} (s : state (history H)) r n : Prop :=
  current_event s = selected_site /\ Storage.operand_reference s 1 = Some r /\
  original_operand s = r /\ locals s 4 = Some (Immediate Uninit) /\
  n <> 0 /\ span_ok (regions (incoming_history s)) r n = true /\
  (fresh_node (nodes (incoming_history s)) (next_tag (incoming_history s)) &&
    fresh_trace (next_tag (incoming_history s)) (reborrow_trace (incoming_history s))) = true /\
  forall a, In a (addresses r n) -> byte_ready (nodes (incoming_history s)) a
    (provenance_tag r) (next_tag (incoming_history s)) (byte_grants (incoming_history s) a).
Theorem model_progress H (s : state (history H)) r n : ready s r n ->
  exists updates, prepare (nodes (incoming_history s)) (provenance_tag r)
    (next_tag (incoming_history s)) (byte_grants (incoming_history s)) (addresses r n) = Some updates /\
  execute s selected_site n = Advanced (commit_reborrow s r n updates).
Proof.
  intros [C [L [O [D [N [R [F B]]]]]]].
  destruct (prepare_progress _ _ _ _ _ B) as [updates P]. exists updates;split;[exact P|].
  unfold execute. rewrite C. destruct (event_eq_dec selected_site selected_site);[|contradiction].
  rewrite L,D. destruct (reference_eq_dec r (original_operand s));[|congruence].
  assert (NZ : Nat.eqb n 0 = false) by (apply Nat.eqb_neq;exact N).
  rewrite NZ,R,F,P. reflexivity.
Qed.
Theorem all_advanced_shape H (s t : state (history H)) e n : execute s e n = Advanced t ->
  exists r updates,
    current_event s = selected_site /\ e = selected_site /\
    Storage.operand_reference s 1 = Some r /\ original_operand s = r /\
    locals s 4 = Some (Immediate Uninit) /\ n <> 0 /\
    span_ok (regions (incoming_history s)) r n = true /\
    (fresh_node (nodes (incoming_history s)) (next_tag (incoming_history s)) &&
    fresh_trace (next_tag (incoming_history s)) (reborrow_trace (incoming_history s))) = true /\
    prepare (nodes (incoming_history s)) (provenance_tag r) (next_tag (incoming_history s))
      (byte_grants (incoming_history s)) (addresses r n) = Some updates /\
    t = commit_reborrow s r n updates.
Proof.
  unfold execute. destruct (event_eq_dec (current_event s) e) as [C|C];[|discriminate].
  destruct (event_eq_dec e selected_site) as [S|S];[|discriminate].
  destruct (Storage.operand_reference s 1) as [r|] eqn:L;[|discriminate].
  destruct (locals s 4) as [[|[|v]|k]|] eqn:D;try discriminate.
  destruct (reference_eq_dec r (original_operand s)) as [O|O];[|discriminate].
  destruct (negb (Nat.eqb n 0) && span_ok (regions (incoming_history s)) r n) eqn:R;[|discriminate].
  destruct (fresh_node (nodes (incoming_history s)) (next_tag (incoming_history s)) &&
    fresh_trace (next_tag (incoming_history s)) (reborrow_trace (incoming_history s))) eqn:F;[|discriminate].
  destruct (prepare (nodes (incoming_history s)) (provenance_tag r) (next_tag (incoming_history s))
    (byte_grants (incoming_history s)) (addresses r n)) as [updates|] eqn:P;[|discriminate].
  intro E; inversion E; subst t. apply andb_true_iff in R. destruct R as [N R].
  apply negb_true_iff in N. apply Nat.eqb_neq in N.
  exists r,updates. repeat split;congruence.
Qed.
Theorem all_advanced_memory H (s t : state (history H)) e n : execute s e n = Advanced t ->
  (forall a, physical_cells t (Caller a) = physical_cells s (Caller a)) /\
  caller_map t = caller_map s /\ original_operand t = original_operand s /\
  private_stack t = private_stack s /\
  prior_history (incoming_history t) = prior_history (incoming_history s).
Proof.
  intro E. destruct (all_advanced_shape H s t e n E) as [r [u [_ [_ [_ [_ [_ [_ [_ [_ [_ T]]]]]]]]]]].
  subst t. repeat split; reflexivity.
Qed.
Theorem all_advanced_control_local H (s t : state (history H)) e n : execute s e n = Advanced t ->
  pc t = after_site /\
  Storage.operand_reference t 4 = Some (child_reference (original_operand s) (next_tag (incoming_history s))) /\
  Storage.operand_reference t 1 = Some (original_operand s) /\
  (forall l, l <> 4 -> locals t l = locals s l).
Proof.
  intro E. destruct (all_advanced_shape H s t e n E) as [r [u [C [_ [L [O [_ [_ [_ [_ [_ T]]]]]]]]]]].
  subst t. split.
  - change (Control (instance_id (pc s)) (block_id (pc s))
      (option_map S (statement_id (pc s))) = after_site).
    unfold current_event,selected_site in C. destruct (pc s) as [i b st].
    simpl in C. inversion C. reflexivity.
  - rewrite O. split;[apply commit_local|]. split.
    + change (Storage.operand_reference s 1 = Some r). exact L.
    + intros l N. apply commit_other_local;exact N.
Qed.
Theorem all_outcomes_at_ready H (s : state (history H)) r n o :
  ready s r n -> step s selected_site n o ->
  exists u, o = Advanced (commit_reborrow s r n u).
Proof.
  intros R E. destruct (model_progress H s r n R) as [u [_ P]].
  exists u. unfold step in E. congruence.
Qed.
Theorem unsupported_other_site H (s : state (history H)) e n :
  current_event s = e -> e <> selected_site -> execute s e n = Unsupported e.
Proof.
  intros C N. unfold execute. destruct (event_eq_dec (current_event s) e);[|contradiction].
  destruct (event_eq_dec e selected_site);[contradiction|reflexivity].
Qed.

Definition registered {H} (h : history H) : Prop :=
  forall a g, In g (byte_grants h a) -> exists x, find_node (grant_tag g) (nodes h) = Some x.
Lemma fresh_global H (h : history H) : registered h ->
  fresh_node (nodes h) (next_tag h) = true ->
  forall a g, In g (byte_grants h a) -> grant_tag g <> next_tag h.
Proof.
  intros R F a g I E. destruct (R a g I) as [x N]. rewrite E in N.
  rewrite (fresh_node_absent _ _ F) in N;discriminate.
Qed.
Lemma commit_trace H (s : state (history H)) r n updates :
  let h := incoming_history s in
  reborrow_trace (incoming_history (commit_reborrow s r n updates)) =
    reborrow_trace h ++ [ChildAllocated (next_tag h) (provenance_tag r) r n] ++
      map (fun a => ByteRetagged a (next_tag h) (provenance_tag r)) (addresses r n) ++
      [LocalAssigned 4 (child_reference r (next_tag h))] /\
  next_tag (incoming_history (commit_reborrow s r n updates)) = S (next_tag h) /\
  storage_trace (commit_reborrow s r n updates) = storage_trace s ++ [LocalWriteHook 4].
Proof. repeat split;reflexivity. Qed.
Theorem advanced_ancestor_and_protector H (s t : state (history H)) e n j :
  Model.execute s e n = Model.Advanced t -> j < n ->
  let a := ref_address (original_operand s) j in
  (exists young g ancestors,
    byte_grants (incoming_history s) a = young ++ g::ancestors /\
    grant_tag g = provenance_tag (original_operand s) /\
    valid_suffix (nodes (incoming_history s)) a (g::ancestors) = true /\
    byte_grants (incoming_history t) a =
      Grant (next_tag (incoming_history s)) Reserved false :: g::ancestors) /\
  (forall g, In g (byte_grants (incoming_history s) a) -> protected g = true ->
    In g (byte_grants (incoming_history t) a)).
Proof.
  intros E J. destruct (all_advanced_shape H s t e n E)
    as [r [u [_ [_ [_ [O [_ [_ [_ [_ [P T]]]]]]]]]]]. subst t. rewrite O.
  pose proof (commit_per_byte H s r n u (ref_address r j) P) as G.
  assert (I : In (ref_address r j) (addresses r n)).
  { apply addresses_member. exists j; split;[exact J|reflexivity]. }
  specialize (G I). split.
  - destruct (grant_retains_suffix _ _ _ _ _ _ G)
      as [_ [young [g [anc [R [Tag [_ [V Out]]]]]]]].
    exists young,g,anc. repeat split; try assumption;congruence.
  - intros g M Prot. eapply protected_grant_preserved; eassumption.
Qed.
