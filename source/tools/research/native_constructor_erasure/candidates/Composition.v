From NativeConstructorErasure Require Import Interface.
From Coq Require Import List.
Import ListNotations.

Definition valid_outcome (M : source_semantics)
    (Inv : configuration M -> Prop) (o : outcome (configuration M)) : Prop :=
  match o with
  | Continue s => Inv s /\ running M s
  | Normal s | Unwind s => Inv s
  | Fault _ => False
  end.

(* Every enabled source outcome is checked, including fault and cleanup.
   Membership is identity coverage only, never the proof of a transition. *)
Definition local_certificate (M : source_semantics) (events : list event)
    (Inv : configuration M -> Prop) : Prop :=
  (forall s, Inv s -> running M s -> exists e o, step M s e o) /\
  (forall s e o, Inv s -> step M s e o ->
    In e events /\ valid_outcome M Inv o).

Definition bounded_certificate (M : source_semantics) (events : list event)
    (s0 : configuration M) : Prop :=
  exists Inv : configuration M -> Prop,
    (Inv s0 /\ running M s0) /\ local_certificate M events Inv /\
    (forall s, Inv s -> physical_frame M s0 s).

Definition source_entry_certificate (M : source_semantics)
    (events : list event) : Prop :=
  forall s0 r A, V_entry M s0 r A ->
    (forall j, (j < 128)%nat -> caller_observable M s0 (ref_address r j)) /\
    bounded_certificate M events s0.

Lemma execution_invariant M events Inv s o :
  local_certificate M events Inv -> (Inv s /\ running M s) -> execution M s o ->
  valid_outcome M Inv o.
Proof.
  intros [Hprogress Hstep] HI Hrun.
  induction Hrun.
  - exact HI.
  - exact (proj2 (Hstep _ _ _ (proj1 HI) H)).
  - apply IHHrun. exact (proj2 (Hstep _ _ _ (proj1 HI) H)).
Qed.

Theorem bounded_erasure M events s0 : bounded_certificate M events s0 ->
  (forall why, ~ execution M s0 (Fault why)) /\
  (forall s, execution M s0 (Normal s) -> physical_frame M s0 s) /\
  (forall s, execution M s0 (Unwind s) -> physical_frame M s0 s) /\
  (forall s, execution M s0 (Continue s) ->
    exists e o, step M s e o).
Proof.
  intros [Inv [Hstart [Hlocal Hframe]]].
  assert (Hinv : forall o, execution M s0 o -> valid_outcome M Inv o).
  { intros o H. exact (execution_invariant M events Inv s0 o Hlocal Hstart H). }
  split.
  - intros why H. exact (Hinv _ H).
  - split.
    + intros s H. apply Hframe. exact (Hinv _ H).
    + split.
      * intros s H. apply Hframe. exact (Hinv _ H).
      * intros s H. destruct (Hinv _ H) as [HI HR]. exact (proj1 Hlocal s HI HR).
Qed.

Theorem entry_erasure M events : source_entry_certificate M events ->
  forall s0 r A, V_entry M s0 r A ->
  (forall why, ~ execution M s0 (Fault why)) /\
  (forall s, execution M s0 (Normal s) -> physical_frame M s0 s) /\
  (forall s, execution M s0 (Unwind s) -> physical_frame M s0 s) /\
  (forall s, execution M s0 (Continue s) -> exists e o, step M s e o).
Proof. intros H s0 r A HV. apply bounded_erasure with events. exact (proj2 (H s0 r A HV)). Qed.

(* A single bad enabled branch refutes a purported certificate. No defined-run
   or successful-normal-outcome restriction can make it disappear. *)
Theorem enabled_fault_refutes_certificate M events s0 s e why :
  execution M s0 (Continue s) -> step M s e (Fault why) ->
  ~ bounded_certificate M events s0.
Proof.
  intros Hrun Hfault [Inv [Hstart [Hlocal Hframe]]].
  pose proof (execution_invariant M events Inv s0 (Continue s)
    Hlocal Hstart Hrun) as HI.
  exact (proj2 (proj2 Hlocal s e (Fault why) (proj1 HI) Hfault)).
Qed.

Lemma operand_projection M s r A : V_entry M s r A ->
  operand_reference M s 1%nat = Some r.
Proof. intros [_ [_ [H _]]]. exact H. Qed.

Lemma original_bytes M s r A : V_entry M s r A ->
  forall j b, List.nth_error A j = Some b ->
    source_heap M s (ref_address r j) = Some b.
Proof. intros [_ [_ [_ [_ [_ H]]]]]. exact H. Qed.

Lemma frame_preserves_initialized M s0 s r A :
  initialized M s0 r A -> physical_frame M s0 s ->
  (forall j, (j < 128)%nat -> caller_observable M s0 (ref_address r j)) ->
  List.length A = 128%nat -> initialized M s r A.
Proof.
  intros [Hbytes Hlookup] Hframe Hobs Hlength. split; [exact Hbytes |].
  intros j b Hnth. rewrite Hframe; [exact (Hlookup j b Hnth) |].
  apply Hobs. rewrite <- Hlength. apply nth_error_Some. rewrite Hnth. discriminate.
Qed.

Theorem entry_normal_bytes M events : source_entry_certificate M events ->
  forall s0 r A s, V_entry M s0 r A -> execution M s0 (Normal s) ->
    initialized M s r A.
Proof.
  intros Hcert s0 r A s HV Hrun.
  destruct (Hcert s0 r A HV) as [Hobs Hbounded].
  destruct HV as [Hlaw [Hentry [Hop [Hlength Hinit]]]].
  apply (frame_preserves_initialized M s0 s r A Hinit).
  - exact (proj1 (proj2 (bounded_erasure M events s0 Hbounded)) s Hrun).
  - exact Hobs.
  - exact Hlength.
Qed.
