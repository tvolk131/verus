From simuliris.tree_borrows Require Import steps_progress loc_controlled_laws tree_access_laws.

(** A pre-entry structural condition, evaluated on the ORIGINAL tree.
    Child here means the access through [parent] is a child access at [it],
    i.e. [it] is [parent] or one of its ancestors. *)
Definition entry_node_eligible (tr : tree item) (C : call_id_set)
    (parent : tag) (r : range') (it : item) : Prop :=
  forall l, range'_contains r l ->
  match rel_dec tr parent it.(itag) with
  | Child _ => perm (item_lookup it l) ≠ Disabled
  | Foreign _ =>
      protector_is_active it.(iprot) C ->
      initialized (item_lookup it l) = PermInit ->
      perm (item_lookup it l) ≠ Active /\
      perm (item_lookup it l) ≠ Disabled
  end.

Definition entry_eligible tr C parent r : Prop :=
  every_node (entry_node_eligible tr C parent r) tr.

(** Typed target interface. Its fields are CONCLUSIONS, never entry premises.
    The subsequent theorem must construct these witnesses from state_wf,
    represented parent, active call, positive size, and entry_eligible. *)
Record entry_result (s : state) (blk : locations.block) (parent : tag)
    (r : range') (cid : call_id) (mid final : trees) : Prop := {
  entry_created : apply_within_trees
    (create_child s.(scs) parent s.(snp) MutRef TyFrz FnEntry cid)
    blk s.(strs) = Some mid;
  entry_read : apply_within_trees
    (memory_access AccessRead s.(scs) s.(snp) r) blk mid = Some final;
  entry_step : bor_step s.(strs) s.(scs) s.(snp) s.(snc)
    (RetagEvt blk r parent s.(snp) MutRef TyFrz cid FnEntry)
    final s.(scs) (S s.(snp)) s.(snc);
  entry_wf : state_wf (mkState s.(shp) final s.(scs) (S s.(snp)) s.(snc));
  entry_outside : forall other, other ≠ blk -> final !! other = s.(strs) !! other
}.

Lemma entry_read_permission (rd : rel_pos) (prot : bool) (lp : lazy_permission) :
  (match rd with
   | Child _ => perm lp ≠ Disabled
   | Foreign _ => prot = true -> initialized lp = PermInit ->
       perm lp ≠ Active /\ perm lp ≠ Disabled
   end) ->
  exists post, apply_access_perm_inner AccessRead rd prot (perm lp) = Some post /\
    (post = Disabled -> most_init (initialized lp) (requires_init rd) = PermLazy \/ prot = false).
Proof.
  destruct rd as [[]|[]]; destruct lp as [[] p];
    destruct p as [|[]| | | |]; destruct prot;
    simpl; intros H; naive_solver.
Qed.



Lemma entry_occurrence_lookup tr it :
  wf_tree tr -> exists_node (eq it) tr -> tree_lookup tr it.(itag) it.
Proof.
  intros Hwf Hex.
  assert (tree_contains it.(itag) tr) as Hcont.
  { apply exists_node_eqv_existential. exists it. split; done. }
  destruct (unique_lookup tr it.(itag) (Hwf _ Hcont)) as (it' & Hdet).
  assert (it = it') as ->.
  { exact ((proj1 (every_node_eqv_universal _ _)) Hdet it Hex eq_refl). }
  split; done.
Qed.

(** Closed-core creation: derived by reduction, with no success hypothesis. *)
Lemma entry_create tr C parent fresh cid :
  create_child C parent fresh MutRef TyFrz FnEntry cid tr =
  Some (insert_child_at tr (mkItem fresh (Some (mkProtector ProtStrong cid))
    (Reserved ResActivable) ∅) (IsTag parent)).
Proof. reflexivity. Qed.

Lemma entry_rel_self tr tg : rel_dec tr tg tg = Child This.
Proof.
  rewrite /rel_dec decide_True; last by left.
  rewrite decide_True; last by left. done.
Qed.

(** Internal composition lemma. Creation equality is used only to connect the
    upstream insertion-preservation lemmas; the public progress theorem below
    derives that equality from entry_create. *)
Lemma entry_inserted_compatible tr tr' C parent fresh cid r :
  wf_tree tr -> wf_tree tr' -> tree_contains parent tr ->
  ~ tree_contains fresh tr ->
  create_child C parent fresh MutRef TyFrz FnEntry cid tr = Some tr' ->
  entry_eligible tr C parent r ->
  every_node (is_access_compatible tr' C fresh r AccessRead) tr'.
Proof.
  intros Hwf Hwf' Hparent Hfresh Hcreate Helig.
  apply every_node_eqv_universal. intros it Hex.
  pose proof (entry_occurrence_lookup _ _ Hwf' Hex) as Hlookup.
  pose proof (create_child_tree_lookup_general C parent fresh MutRef TyFrz FnEntry
    cid tr tr' (itag it) it Hparent Hfresh Hcreate Hlookup) as Hcases.
  destruct Hcases as [(Hne & Hold)|(Htag & Hnew)].
  - pose proof ((proj1 (every_node_eqv_universal _ _)) Helig it
      (tree_lookup_to_exists_node _ _ _ Hold)) as Hit.
    pose proof (create_child_rel_dec_new fresh (itag it) C parent fresh
      MutRef TyFrz FnEntry cid tr tr' (rel_dec tr' fresh (itag it))
      Hwf Hwf' Hparent Hfresh (lookup_implies_contains Hlookup) Hcreate eq_refl Hne) as Hrel.
    unfold is_access_compatible. intros l Hl. apply entry_read_permission.
    specialize (Hit l Hl).
    destruct Hrel as [(Hcontra & _)|[(_ & Hpar & Hrd)|[(_ & _ & Hrd & i & Hrdold)|(_ & _ & Hrd & f & Hrdold)]]].
    + contradiction.
    + rewrite Hrd. rewrite Hpar entry_rel_self in Hit. exact Hit.
    + rewrite Hrd. rewrite Hrdold in Hit. exact Hit.
    + rewrite Hrd. rewrite Hrdold in Hit.
      intros Hprot Hinit. apply Hit; last exact Hinit.
      by apply bool_decide_eq_true in Hprot.
  - simpl in Hnew. injection Hnew as Hnew. subst it.
    unfold is_access_compatible. intros l Hl. apply entry_read_permission.
    rewrite /= entry_rel_self /item_lookup lookup_empty /=. done.
Qed.

Lemma entry_fresh s blk tr :
  state_wf s -> s.(strs) !! blk = Some tr -> ~ tree_contains s.(snp) tr.
Proof.
  intros Hwf Htr Hcont.
  eapply tree_items_compat_next_not_containing; first exact Hcont.
  - exact (state_wf_tree_compat _ Hwf _ _ Htr).
  - lia.
Qed.

Lemma entry_progress s blk tr parent off sz cid :
  state_wf s -> s.(strs) !! blk = Some tr ->
  tree_contains parent tr -> cid ∈ s.(scs) -> (0 < sz)%nat ->
  entry_eligible tr s.(scs) parent (off, sz) ->
  exists mid final, entry_result s blk parent (off, sz) cid mid final.
Proof.
  intros Hwf Htr Hparent Hcid Hsz Helig.
  pose (child := mkItem s.(snp) (Some (mkProtector ProtStrong cid))
    (Reserved ResActivable) ∅).
  pose (trmid := insert_child_at tr child (IsTag parent)).
  pose (mid := <[blk := trmid]> s.(strs)).
  assert (apply_within_trees
    (create_child s.(scs) parent s.(snp) MutRef TyFrz FnEntry cid)
    blk s.(strs) = Some mid) as Hcreate.
  { rewrite /apply_within_trees Htr /=. done. }
  assert (trees_contain parent s.(strs) blk) as Hpar.
  { by rewrite /trees_contain /trees_at_block Htr. }
  assert (~ trees_contain s.(snp) s.(strs) blk) as Hfresh.
  { rewrite /trees_contain /trees_at_block Htr. by apply (entry_fresh s blk tr). }
  pose proof (retag_step_wf_inner s blk parent MutRef TyFrz FnEntry cid mid
    Hwf Hpar Hfresh Hcid Hcreate) as (Hmidwf & Hnewtag).
  assert (mid !! blk = Some trmid) as Hmidtr by apply lookup_insert_eq.
  assert (is_Some (memory_access AccessRead s.(scs) s.(snp) (off, sz) trmid)) as Hread.
  { apply all_access_compatible_implies_access_success.
    eapply entry_inserted_compatible.
    - exact (proj1 (state_wf_tree_unq _ Hwf) _ _ Htr).
    - exact (proj1 (state_wf_tree_unq _ Hmidwf) _ _ Hmidtr).
    - exact Hparent.
    - exact (entry_fresh s blk tr Hwf Htr).
    - apply entry_create.
    - exact Helig. }
  destruct Hread as (trfinal & Hread).
  pose (final := <[blk := trfinal]> mid).
  assert (apply_within_trees (memory_access AccessRead s.(scs) s.(snp) (off, sz))
    blk mid = Some final) as Hreadtrees.
  { by rewrite /apply_within_trees Hmidtr /= Hread. }
  exists mid, final. constructor.
  - exact Hcreate.
  - exact Hreadtrees.
  - eapply RetagIS; done.
  - exact (access_step_wf_inner (mkState s.(shp) mid s.(scs) (S s.(snp)) s.(snc)) false AccessRead s.(snp) blk off sz final
      Hnewtag Hreadtrees Hmidwf).
  - intros other Hne. rewrite /final /mid !lookup_insert_ne; done.
Qed.

(** Existing nodes retain their tag/protector/default permission and every
    lazy permission outside the retagged interval, even within the same block.
    This is a consequence of the already derived result, not an entry premise. *)
Lemma entry_existing_range_frame s blk tr parent off sz cid mid final tg it :
  state_wf s -> s.(strs) !! blk = Some tr ->
  tree_contains parent tr -> cid ∈ s.(scs) ->
  entry_result s blk parent (off, sz) cid mid final ->
  trees_lookup s.(strs) blk tg it ->
  exists itnew, trees_lookup final blk tg itnew /\
    initp it = initp itnew /\ iprot it = iprot itnew /\
    forall l, ~ range'_contains (off, sz) l -> item_lookup it l = item_lookup itnew l.
Proof.
  intros Hwf Htr Hparent Hcid Hres Hold.
  destruct Hres as [Hcreate Hread Hstep Hfinalwf Houtside].
  assert (trees_contain parent s.(strs) blk) as Hpar.
  { by rewrite /trees_contain /trees_at_block Htr. }
  assert (~ trees_contain s.(snp) s.(strs) blk) as Hfresh.
  { rewrite /trees_contain /trees_at_block Htr. by apply (entry_fresh s blk tr). }
  pose proof (retag_step_wf_inner s blk parent MutRef TyFrz FnEntry cid mid
    Hwf Hpar Hfresh Hcid Hcreate) as (Hmidwf & Hnewtag).
  assert (tg ≠ s.(snp)) as Hne.
  { intros ->. apply Hfresh. by eapply trees_contain_trees_lookup_2. }
  assert (trees_lookup mid blk tg it) as Hmidlookup.
  { destruct Hold as (tr0 & Htr0 & Hlu).
    rewrite /apply_within_trees in Hcreate.
    apply bind_Some in Hcreate as (tr1 & Htr1 & Hcreate).
    apply bind_Some in Hcreate as (tr2 & Hcr & Hmid).
    injection Hmid as <-. assert (tr0 = tr1) as -> by congruence.
    exists tr2. split; first apply lookup_insert_eq.
    exact ((proj1 (create_child_tree_lookup s.(scs) parent s.(snp) MutRef TyFrz FnEntry
      cid tr1 tr2 tg it Hne Hcr)) Hlu). }
  destruct (apply_trees_access_lookup_inout false s.(scs) mid AccessRead blk off sz
    s.(snp) tg final it Hread (state_wf_tree_unq _ Hmidwf) Hmidlookup)
    as (itnew & Hlu & Hinit & Hprot & Hloc).
  exists itnew. split; first exact Hlu. split; first exact Hinit.
  split; first exact Hprot. intros l Hl. apply (proj2 (Hloc l)). exact Hl.
Qed.
