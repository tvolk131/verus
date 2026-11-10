(* UNVALIDATED CANDIDATE. Compile only against the checked pinned stbor baseline.
   These are conditional theorems of that published MODEL, not native Rust facts. *)
From stbor.lang Require Import steps_progress steps_retag steps_inversion.

Set Default Proof Using "Type".

Definition mutable_top_block (alpha : stacks) (l : loc) (p : ptr_id) (n : nat) : Prop :=
  forall i, (i < n)%nat -> exists pm prot tail,
    (pm = Unique \/ pm = SharedReadWrite) /\
    alpha !! (l +ₗ i) = Some (mkItem pm (Tagged p) prot :: tail).

Lemma top_write_pre cids p pm prot tail :
  (pm = Unique \/ pm = SharedReadWrite) ->
  access1_pre cids (mkItem pm (Tagged p) prot :: tail) AccessWrite (Tagged p).
Proof.
  intros PM. rewrite /access1_pre.
  exists 0%nat, (mkItem pm (Tagged p) prot).
  destruct PM as [->|->]; simpl; repeat split; try done;
    intros j jt LT _; lia.
Qed.

Lemma top_block_valid alpha cids l p n :
  mutable_top_block alpha l p n ->
  valid_block alpha cids l (Tagged p) (RefPtr Mutable) (FixedSize n).
Proof.
  intros TOP. split; [done|]. intros i LT.
  destruct (TOP i LT) as (pm & prot & tail & PM & LOOK).
  split.
  - apply elem_of_dom. eexists. exact LOOK.
  - exists (mkItem pm (Tagged p) prot :: tail). split; [exact LOOK|].
    by apply top_write_pre.
Qed.

(* Covers the published Default/TwoPhase/FnEntry reference operation. The
   call-stack and WF premises remain explicit, with arbitrary ancestor tails. *)
Lemma bounded_reference_head_step fns sigma l p n rk c cids :
  sigma.(scs) = c :: cids -> Wf sigma ->
  mutable_top_block sigma.(sst) l p n ->
  exists new alpha' next,
    retag sigma.(sst) sigma.(snp) sigma.(scs) c l (Tagged p) rk
      (RefPtr Mutable) (FixedSize n) = Some (new, alpha', next) /\
    let final := mkState sigma.(shp) alpha' sigma.(scs) next sigma.(snc) in
    head_step fns (Retag #[ScPtr l (Tagged p)] (RefPtr Mutable) (FixedSize n) rk)
      sigma [RetagEvt l (Tagged p) new (RefPtr Mutable) (FixedSize n) rk]
      #[ScPtr l new] final [] /\
    final.(shp) = sigma.(shp) /\ final.(scs) = sigma.(scs) /\ Wf final.
Proof.
  intros CS WF TOP.
  have BAR : exists active, active \in sigma.(scs).
  { exists c. rewrite CS. by left. }
  have BLOCK := top_block_valid sigma.(sst) sigma.(scs) l p n TOP.
  destruct (retag_head_step fns sigma l (Tagged p) (RefPtr Mutable)
    (FixedSize n) rk BAR BLOCK WF)
    as (c0 & rest & new & alpha' & next & CS0 & RT & STEP).
  rewrite CS in CS0. injection CS0 as EQC EQREST. subst c0 rest.
  exists new, alpha', next. split; [exact RT|].
  split; [exact STEP|]. split; [reflexivity|]. split; [reflexivity|].
  eapply head_step_wf; eauto.
Qed.

Lemma nonempty_reference_result alpha next c cids l old rk n new alpha' next' :
  (0 < n)%nat ->
  retag alpha next cids c l old rk (RefPtr Mutable) (FixedSize n) =
    Some (new, alpha', next') ->
  new = Tagged next /\ next' = S next.
Proof.
  intros POS RT. eapply retag_change_ref_NZST; [exact POS|exact RT].
Qed.

Lemma empty_reference_result alpha next c cids l old rk :
  retag alpha next cids c l old rk (RefPtr Mutable) (FixedSize 0) =
    Some (old, alpha, next).
Proof. destruct rk; reflexivity. Qed.

Lemma reference_outside_stack_frame alpha next c cids l old rk n new alpha' next' :
  retag alpha next cids c l old rk (RefPtr Mutable) (FixedSize n) =
    Some (new, alpha', next') ->
  forall outside, (forall i, (i < n)%nat -> outside <> l +ₗ i) ->
    alpha' !! outside = alpha !! outside.
Proof.
  intros RT. exact (proj1 (retag_Some alpha next c cids l old rk
    (RefPtr Mutable) (FixedSize n) new alpha' next' RT)).
Qed.

Lemma reference_active_protector_frame alpha next c cids l old rk n new alpha' next' :
  retag alpha next cids c l old rk (RefPtr Mutable) (FixedSize n) =
    Some (new, alpha', next') ->
  forall at before t active pm,
    alpha !! at = Some before -> active \in cids ->
    mkItem pm t (Some active) \in before ->
    exists after, alpha' !! at = Some after /\ mkItem pm t (Some active) \in after.
Proof.
  intros RT. exact (retag_item_active_preserving alpha next c cids l old rk
    (RefPtr Mutable) (FixedSize n) new alpha' next' RT).
Qed.

(* Universal over actual transitions of the published language, not just the
   constructed successful witness. Progress plus this statement handles its
   stuck-state possibility under the earlier structural premises. *)
Lemma all_reference_steps_keep_heap fns (ptr : result) pk T rk e' sigma sigma' :
  (Retag ptr pk T rk, sigma) ~{fns}~> (e', sigma') ->
  sigma'.(shp) = sigma.(shp).
Proof.
  intros STEP.
  destruct (tstep_retag_inv fns ptr T pk rk e' sigma sigma' STEP)
    as (l & old & c & cids & new & alpha' & next & _ & _ & _ & _ & FINAL).
  subst sigma'. reflexivity.
Qed.

Lemma all_reference_steps_keep_initialized_vector fns (ptr : result) pk T rk
    e' sigma sigma' l (bytes : list Z) :
  (Retag ptr pk T rk, sigma) ~{fns}~> (e', sigma') ->
  (forall i b, bytes !! i = Some b -> sigma.(shp) !! (l +ₗ i) = Some (ScInt b)) ->
  forall i b, bytes !! i = Some b -> sigma'.(shp) !! (l +ₗ i) = Some (ScInt b).
Proof.
  intros STEP INIT i b LOOK.
  rewrite (all_reference_steps_keep_heap fns ptr pk T rk e' sigma sigma' STEP).
  exact (INIT i b LOOK).
Qed.

Lemma two_phase_128_progress (fns : fn_env) sigma l p c cids :
  sigma.(scs) = c :: cids -> Wf sigma ->
  mutable_top_block sigma.(sst) l p 128 ->
  exists new alpha' next,
    retag sigma.(sst) sigma.(snp) sigma.(scs) c l (Tagged p) TwoPhase
      (RefPtr Mutable) (FixedSize 128) = Some (new, alpha', next).
Proof.
  intros CS WF TOP.
  destruct (bounded_reference_head_step fns sigma l p 128 TwoPhase c cids CS WF TOP)
    as (new & alpha' & next & RT & REST).
  exists new, alpha', next. exact RT.
Qed.

Lemma two_phase_10_progress (fns : fn_env) sigma l p c cids :
  sigma.(scs) = c :: cids -> Wf sigma ->
  mutable_top_block sigma.(sst) l p 10 ->
  exists new alpha' next,
    retag sigma.(sst) sigma.(snp) sigma.(scs) c l (Tagged p) TwoPhase
      (RefPtr Mutable) (FixedSize 10) = Some (new, alpha', next).
Proof.
  intros CS WF TOP.
  destruct (bounded_reference_head_step fns sigma l p 10 TwoPhase c cids CS WF TOP)
    as (new & alpha' & next & RT & REST).
  exists new, alpha', next. exact RT.
Qed.
