From simuliris.tree_borrows Require Import steps_progress loc_controlled_laws.

(** A pre-entry structural condition, evaluated on the ORIGINAL tree.
    Child here means the access through [parent] is a child access at [it],
    i.e. [it] is [parent] or one of its ancestors. *)
Definition entry_node_eligible (tr : tree item) (C : call_id_set)
    (parent : tag) (r : range') (it : item) : Prop :=
  forall l, range'_contains r l ->
  match rel_dec tr parent it.(itag) with
  | Child _ => perm (item_lookup it l) <> Disabled
  | Foreign _ =>
      protector_is_active it.(iprot) C ->
      initialized (item_lookup it l) = PermInit ->
      perm (item_lookup it l) <> Active /\
      perm (item_lookup it l) <> Disabled
  end.

Definition entry_eligible tr C parent r : Prop :=
  every_node (entry_node_eligible tr C parent r) tr.

(** Typed target interface. Its fields are CONCLUSIONS, never entry premises.
    The subsequent theorem must construct these witnesses from state_wf,
    represented parent, active call, positive size, and entry_eligible. *)
Record entry_result (s : state) (blk : block) (parent : tag)
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
  entry_outside : forall other, other <> blk -> final !! other = s.(strs) !! other
}.
