(* Conditional historical MODEL composition; no native/source carrier instance. *)
From handoff Require Import candidate.
From stbor.lang Require Import steps_progress steps_retag steps_inversion.
Set Default Proof Using "Type".

(* Whole heaps, calls and next-call counters are exact. Existing active
   ancestors/protectors survive; no claim that all unprotected caller grants
   survive FnEntry's access check. *)
Definition model_retag_frame (before after : state) (l : loc) (n : nat) : Prop :=
  after.(shp) = before.(shp) /\ after.(scs) = before.(scs) /\
  after.(snc) = before.(snc) /\ Wf after /\
  (forall outside, (forall i, (i < n)%nat -> outside ≠ l +ₗ i) ->
    after.(sst) !! outside = before.(sst) !! outside) /\
  (forall location stk pm t active,
    before.(sst) !! location = Some stk -> active ∈ before.(scs) ->
    mkItem pm t (Some active) ∈ stk ->
    exists stk', after.(sst) !! location = Some stk' /\
      mkItem pm t (Some active) ∈ stk').

Lemma model_retag_frame_trans a b d l n :
  model_retag_frame a b l n -> model_retag_frame b d l n ->
  model_retag_frame a d l n.
Proof.
  intros (H1 & C1 & N1 & W1 & O1 & P1) (H2 & C2 & N2 & W2 & O2 & P2).
  split; [congruence|]. split; [congruence|]. split; [congruence|].
  split; [done|]. split.
  - intros out OUT. rewrite O2; [by apply O1|done].
  - intros loc stk pm t ac LOOK LIVE IN.
    destruct (P1 loc stk pm t ac LOOK LIVE IN) as (mid & LOOK1 & IN1).
    apply (P2 loc mid pm t ac LOOK1); [by rewrite C1|done].
Qed.

Lemma successful_reference_frame fns sigma l old rk n new alpha' next c cids :
  Wf sigma -> sigma.(scs) = c :: cids ->
  retag sigma.(sst) sigma.(snp) sigma.(scs) c l old rk
    (RefPtr Mutable) (FixedSize n) = Some (new, alpha', next) ->
  model_retag_frame sigma
    (mkState sigma.(shp) alpha' sigma.(scs) next sigma.(snc)) l n.
Proof.
  intros WF CS RT. split; [done|]. split; [done|]. split; [done|].
  split.
  - eapply (head_step_wf fns); [eapply retag_head_step'; eauto|done].
  - split.
    + exact (reference_outside_stack_frame _ _ _ _ _ _ _ _ _ _ _ RT).
    + intros loc stk pm t ac LOOK LIVE IN.
      exact (reference_active_protector_frame _ _ _ _ _ _ _ _ _ _ _ RT
        loc stk t ac pm LOOK LIVE IN).
Qed.

Lemma fnentry_exact_protected_head alpha next c cids l old n new alpha' next' :
  retag alpha next cids c l old FnEntry (RefPtr Mutable) (FixedSize n) =
    Some (new, alpha', next') ->
  forall i, (i < n)%nat -> exists tail,
    alpha' !! (l +ₗ i) = Some (mkItem Unique new (Some c) :: tail).
Proof.
  intros RT i LT.
  destruct (proj2 (retag_Some _ _ _ _ _ _ _ _ _ _ _ _ RT) i LT)
    as (stk & stk' & LOOK & LOOK' & GR).
  destruct (tag_on_top_grant_uniq_SRO stk old (mkItem Unique new (Some c))
    cids stk' (or_introl eq_refl) GR) as [tail EQ].
  rewrite EQ in LOOK'. by exists tail.
Qed.

(* On a Unique head, the actual published grant prepends the TwoPhase item.
   Every entry ancestor tail, including inactive protectors, is untouched. *)
Lemma two_phase_grant_on_unique cids p q prot tail :
  grant (mkItem Unique (Tagged p) prot :: tail) (Tagged p)
    (mkItem SharedReadWrite (Tagged q) None) cids =
  Some (mkItem SharedReadWrite (Tagged q) None ::
    mkItem Unique (Tagged p) prot :: tail).
Proof.
  rewrite /grant /find_granting /=.
  rewrite decide_True; last by split.
  simpl. rewrite /item_insert_dedup /= decide_False //.
Qed.

Lemma two_phase_exact_installation alpha next c cids l p n new alpha' next' :
  retag alpha next cids c l (Tagged p) TwoPhase
    (RefPtr Mutable) (FixedSize n) = Some (new, alpha', next') ->
  forall i prot tail, (i < n)%nat ->
    alpha !! (l +ₗ i) = Some (mkItem Unique (Tagged p) prot :: tail) ->
    alpha' !! (l +ₗ i) = Some (mkItem SharedReadWrite new None ::
      mkItem Unique (Tagged p) prot :: tail).
Proof.
  intros RT i prot tail LT LOOK.
  destruct (proj2 (retag_Some _ _ _ _ _ _ _ _ _ _ _ _ RT) i LT)
    as (stk & stk' & LOOK0 & LOOK' & GR).
  rewrite LOOK in LOOK0. injection LOOK0 as EQ. subst stk.
  destruct (retag_change_ref_NZST alpha next cids c l (Tagged p) TwoPhase Mutable
    (FixedSize n) new alpha' next') as [TAG NEXT].
  - simpl. lia.
  - exact RT.
  - subst new. rewrite two_phase_grant_on_unique in GR.
    injection GR as EQ. by subst stk'.
Qed.

(* This relation installs returned MODEL tags in the next Retag expression and
   actual stack. It does NOT assert a Rust _1/_4 carrier write or PC advance. *)
Definition entry_reborrow_contract fns sigma l old n c entry final : Prop :=
  let p := sigma.(snp) in let q := S p in
  retag sigma.(sst) p sigma.(scs) c l old FnEntry
    (RefPtr Mutable) (FixedSize n) = Some (Tagged p, entry.(sst), q) /\
  entry = mkState sigma.(shp) entry.(sst) sigma.(scs) q sigma.(snc) /\
  head_step fns (Retag #[ScPtr l old] (RefPtr Mutable) (FixedSize n) FnEntry)
    sigma [RetagEvt l old (Tagged p) (RefPtr Mutable) (FixedSize n) FnEntry]
    #[ScPtr l (Tagged p)] entry [] /\
  retag entry.(sst) entry.(snp) entry.(scs) c l (Tagged p) TwoPhase
    (RefPtr Mutable) (FixedSize n) = Some (Tagged q, final.(sst), S q) /\
  final = mkState entry.(shp) final.(sst) entry.(scs) (S q) entry.(snc) /\
  head_step fns (Retag #[ScPtr l (Tagged p)] (RefPtr Mutable) (FixedSize n) TwoPhase)
    entry [RetagEvt l (Tagged p) (Tagged q) (RefPtr Mutable) (FixedSize n) TwoPhase]
    #[ScPtr l (Tagged q)] final [] /\
  model_retag_frame sigma entry l n /\
  model_retag_frame entry final l n /\
  model_retag_frame sigma final l n /\
  forall i, (i < n)%nat -> exists tail,
    entry.(sst) !! (l +ₗ i) = Some (mkItem Unique (Tagged p) (Some c) :: tail) /\
    final.(sst) !! (l +ₗ i) = Some (mkItem SharedReadWrite (Tagged q) None ::
      mkItem Unique (Tagged p) (Some c) :: tail).

Lemma published_fnentry_two_phase fns sigma l old n c cids :
  (0 < n)%nat -> Wf sigma -> sigma.(scs) = c :: cids ->
  valid_block sigma.(sst) sigma.(scs) l old (RefPtr Mutable) (FixedSize n) ->
  exists entry final, entry_reborrow_contract fns sigma l old n c entry final.
Proof.
  intros POS WF CS BLOCK.
  destruct (fnentry_normalizes_from_valid_block sigma l old n c cids POS WF CS BLOCK)
    as (p & a1 & next & RT1 & TOP).
  destruct (nonempty_reference_result _ _ _ _ _ _ _ _ _ _ _ POS RT1) as [TAG NEXT].
  injection TAG as EQ. subst p next.
  set (entry := mkState sigma.(shp) a1 sigma.(scs) (S sigma.(snp)) sigma.(snc)).
  have HS1 := retag_head_step' fns sigma l old (Tagged sigma.(snp))
    (RefPtr Mutable) (FixedSize n) FnEntry c cids a1 (S sigma.(snp)) CS RT1.
  have WF1 : Wf entry by eapply head_step_wf; eauto.
  have CS1 : entry.(scs) = c :: cids by exact CS.
  destruct (bounded_reference_head_step fns entry l sigma.(snp) n TwoPhase c cids
    CS1 WF1 TOP) as (new & a2 & next & RT2 & HS2 & HEAP & CALL & WF2).
  destruct (nonempty_reference_result _ _ _ _ _ _ _ _ _ _ _ POS RT2) as [TAG NEXT].
  subst new next.
  set (final := mkState entry.(shp) a2 entry.(scs) (S entry.(snp)) entry.(snc)).
  have F1 : model_retag_frame sigma entry l n.
  { exact (successful_reference_frame fns sigma l old FnEntry n _ a1 _ c cids WF CS RT1). }
  have F2 : model_retag_frame entry final l n.
  { exact (successful_reference_frame fns entry l _ TwoPhase n _ a2 _ c cids WF1 CS1 RT2). }
  exists entry, final. unfold entry_reborrow_contract.
  split; [exact RT1|]. split; [done|]. split; [exact HS1|].
  split; [exact RT2|]. split; [done|]. split; [exact HS2|].
  split; [exact F1|]. split; [exact F2|].
  split; [eapply model_retag_frame_trans; eauto|].
  intros i LT. destruct (fnentry_exact_protected_head _ _ _ _ _ _ _ _ _ _ RT1 i LT)
    as [tail LOOK]. exists tail. split; [exact LOOK|].
  exact (two_phase_exact_installation _ _ _ _ _ _ _ _ _ _ RT2 i (Some c) tail LT LOOK).
Qed.

Lemma published_fnentry_two_phase_10 fns sigma l old c cids :
  Wf sigma -> sigma.(scs) = c :: cids ->
  valid_block sigma.(sst) sigma.(scs) l old (RefPtr Mutable) (FixedSize 10) ->
  exists entry final, entry_reborrow_contract fns sigma l old 10 c entry final.
Proof. intros. eapply published_fnentry_two_phase; [lia|eassumption..]. Qed.

Lemma published_fnentry_two_phase_128 fns sigma l old c cids :
  Wf sigma -> sigma.(scs) = c :: cids ->
  valid_block sigma.(sst) sigma.(scs) l old (RefPtr Mutable) (FixedSize 128) ->
  exists entry final, entry_reborrow_contract fns sigma l old 128 c entry final.
Proof. intros. eapply published_fnentry_two_phase; [lia|eassumption..]. Qed.

(* n=0 requires no allocated stack location in this historical model: even an
   Untagged/dangling model location stutters. This is NOT current Rust behavior. *)
Lemma empty_model_entry_two_phase alpha next c cids l old :
  retag alpha next cids c l old FnEntry (RefPtr Mutable) (FixedSize 0) =
    Some (old, alpha, next) /\
  retag alpha next cids c l old TwoPhase (RefPtr Mutable) (FixedSize 0) =
    Some (old, alpha, next).
Proof. split; apply empty_reference_result. Qed.
