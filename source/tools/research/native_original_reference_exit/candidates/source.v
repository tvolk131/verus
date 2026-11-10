From NativeOriginalReferenceExit Require Import frontend.
From NativeConstructorErasure Require Import Interface Composition.
From Coq Require Import List ZArith Lia.
Import ListNotations.

(* Reuse the delivered source signature. M is STILL UNINSTANTIATED; neither
   this lemma nor its import supplies a Rust interpretation or certificate. *)
Lemma initialized_bytes_unique M s r xs ys :
  initialized M s r xs -> initialized M s r ys ->
  length xs = length ys -> xs = ys.
Proof.
  intros [_ X] [_ Y] Len. apply nth_error_ext. intros j.
  destruct (nth_error xs j) as [x|] eqn:HX;
    destruct (nth_error ys j) as [y|] eqn:HY; try reflexivity.
  - specialize (X j x HX). specialize (Y j y HY). congruence.
  - assert (J : (j < length xs)%nat).
    { apply nth_error_Some. rewrite HX. discriminate. }
    apply nth_error_None in HY. lia.
  - assert (J : (j < length ys)%nat).
    { apply nth_error_Some. rewrite HY. discriminate. }
    apply nth_error_None in HX. lia.
Qed.

(* Whole-vector observation, including every unissued suffix byte. The two
   physical CURRENT interpretations and the pending release are explicit
   UNPROVED source bindings in native use, not an abstract F=heap axiom. *)
Theorem conditional_source_final_vector M final original_ref
    (original pending pending_exit : list Z * list Z) result :
  initialized M final original_ref result ->
  initialized M final original_ref (cf_current pending_exit) ->
  length (cf_current pending_exit) = length result ->
  cf_resolved (cf_update_current original (cf_future pending)) ->
  cf_future pending_exit = cf_future pending -> cf_resolved pending_exit ->
  cf_future original = result /\
  initialized M final original_ref (cf_future original).
Proof.
  intros Result Current Len Merge Same Release.
  pose proof (initialized_bytes_unique M final original_ref
    (cf_current pending_exit) result Current Result Len) as Physical.
  pose proof (conditional_original_exit original pending pending_exit result
    Merge Same Physical Release) as Future.
  split; [exact Future|]. rewrite Future. exact Result.
Qed.

(* Constructor/drop specialization on the SAME incoming `_1` reference from
   V_entry. Fault coverage comes from the full certificate, not from filtering
   executions to normal/defined runs. No actual M/certificate is supplied. *)
Theorem conditional_source_drop_exit M events initial final original_ref bytes
    (original pending pending_exit : list Z * list Z) :
  source_entry_certificate M events ->
  V_entry M initial original_ref bytes -> execution M initial (Normal final) ->
  cf_current original = bytes ->
  initialized M final original_ref (cf_current pending_exit) ->
  length (cf_current pending_exit) = 128%nat ->
  cf_resolved (cf_update_current original (cf_future pending)) ->
  cf_future pending_exit = cf_future pending -> cf_resolved pending_exit ->
  (forall why, ~ execution M initial (Fault why)) /\
  cf_future original = cf_current original /\
  initialized M final original_ref (cf_future original).
Proof.
  intros Certificate Entry Run EntryCF Current Len Merge Same Release.
  pose proof (entry_normal_bytes M events Certificate initial original_ref bytes
    final Entry Run) as FinalBytes.
  assert (LenBytes : length bytes = 128%nat).
  { destruct Entry as [_ [_ [_ [H _]]]]. exact H. }
  assert (EqualLengths : length (cf_current pending_exit) = length bytes) by lia.
  destruct (conditional_source_final_vector M final original_ref original pending
    pending_exit bytes FinalBytes Current EqualLengths Merge Same Release)
    as [Future Observed].
  split.
  - exact (proj1 (entry_erasure M events Certificate initial original_ref bytes Entry)).
  - split; [|exact Observed]. rewrite EntryCF. exact Future.
Qed.
