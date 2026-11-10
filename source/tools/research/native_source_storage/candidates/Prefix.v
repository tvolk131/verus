From Coq Require Import List ZArith Arith.PeanoNat.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage Sites.
Import ListNotations.
Local Open Scope nat_scope.

Definition event_eq_dec (x y : event) : {x = y} + {x <> y}.
Proof. repeat decide equality. Defined.
Inductive result (H : Type) :=
| Advanced (s : state H)
| Unsupported (e : event)
| StorageFault
| ControlFault.
Arguments Advanced {H}.
Arguments Unsupported {H}.
Arguments StorageFault {H}.
Arguments ControlFault {H}.
Definition execute {H} (s : state H) e : result H :=
  if event_eq_dec (current_event s) e then
    match storage_at e with
    | Some (live,l) =>
      match (if live then storage_live s l else storage_dead s l) with
      | Some t => Advanced (advance t)
      | None => StorageFault
      end
    | None => Unsupported e
    end
  else ControlFault.
(* Existing outcome carrier has no Unsupported constructor. Fault InvalidControl
   here encodes model coverage failure, never a claim of Rust UB. Keep execute's
   richer diagnostic when interpreting this carrier. No Normal exit is supplied. *)
Definition operational_step {H} (s : state H) e (o : outcome (state H)) : Prop :=
  match execute s e with
  | Advanced t => o = Continue t
  | Unsupported _ | ControlFault => o = Fault InvalidControl
  | StorageFault => o = Fault InvalidStorage
  end.
Definition initial_locals (r : reference) (l : nat) : option local :=
  match l with
  | 0 => Some (Immediate Uninit)
  | 1 => Some (Immediate (Init (RefValue r)))
  | 2 | 3 | 4 => Some Dead
  | _ => None
  end.
Definition root_entry {H} r caller (history : H) stack fresh trace : state H :=
  State (initial_locals r) (Control 0 0 (Some 0)) r caller history stack fresh trace.
Definition prefix_one {H} (s : state H) := advance (live_sized s 2).
Definition prefix_two {H} (s : state H) := advance (live_sized (prefix_one s) 3).
Definition prefix_three {H} (s : state H) := advance (live_sized (prefix_two s) 4).

Lemma first_three_steps H r caller (history : H) stack fresh trace :
  let s := root_entry r caller history stack fresh trace in
  execute s (MirOperation 0 0 (Some 0)) = Advanced (prefix_one s) /\
  execute (prefix_one s) (MirOperation 0 0 (Some 1)) = Advanced (prefix_two s) /\
  execute (prefix_two s) (MirOperation 0 0 (Some 2)) = Advanced (prefix_three s).
Proof. repeat split; reflexivity. Qed.
Lemma prefix_exact_control H r caller (history : H) stack fresh trace :
  current_event (prefix_three (root_entry r caller history stack fresh trace)) =
    MirOperation 0 0 (Some 3).
Proof. reflexivity. Qed.
Lemma prefix_exact_locals H r caller (history : H) stack fresh trace :
  let s := prefix_three (root_entry r caller history stack fresh trace) in
  locals s 0 = Some (Immediate Uninit) /\
  Storage.operand_reference s 1 = Some r /\
  locals s 2 = Some (Immediate Uninit) /\
  locals s 3 = Some (Immediate Uninit) /\
  locals s 4 = Some (Immediate Uninit).
Proof. repeat split; reflexivity. Qed.
Lemma prefix_anchors H r caller (history : H) stack fresh trace :
  let s := root_entry r caller history stack fresh trace in
  anchors (prefix_three s) = anchors s /\
  private_stack (prefix_three s) = stack /\
  storage_trace (prefix_three s) =
    ((trace ++ [LocalWriteHook 2]) ++ [LocalWriteHook 3]) ++ [LocalWriteHook 4].
Proof. repeat split; reflexivity. Qed.
Theorem prefix_all_caller_cells H r caller (history : H) stack fresh trace :
  forall a, physical_cells (prefix_three (root_entry r caller history stack fresh trace))
    (Caller a) = caller a.
Proof. reflexivity. Qed.
Theorem prefix_original_bytes H r caller (history : H) stack fresh trace A :
  (forall j b, nth_error A j = Some b -> caller (ref_address r j) = Some (InitByte b)) ->
  forall j b, nth_error A j = Some b ->
    caller_map (prefix_three (root_entry r caller history stack fresh trace))
      (ref_address r j) = Some b.
Proof. intros B j b E. unfold caller_map. simpl. rewrite (B j b E). reflexivity. Qed.
Theorem prefix_execution H r caller (history : H) stack fresh trace :
  let s := root_entry r caller history stack fresh trace in
  operational_step s (MirOperation 0 0 (Some 0)) (Continue (prefix_one s)) /\
  operational_step (prefix_one s) (MirOperation 0 0 (Some 1)) (Continue (prefix_two s)) /\
  operational_step (prefix_two s) (MirOperation 0 0 (Some 2)) (Continue (prefix_three s)).
Proof. repeat split; reflexivity. Qed.
Theorem reborrow_is_explicitly_unsupported H r caller (history : H) stack fresh trace :
  let s := prefix_three (root_entry r caller history stack fresh trace) in
  execute s (MirOperation 0 0 (Some 3)) = Unsupported (MirOperation 0 0 (Some 3)).
Proof. reflexivity. Qed.
Lemma all_unimplemented_explicit H (s : state H) e :
  current_event s = e -> storage_at e = None -> execute s e = Unsupported e.
Proof. intros E N. unfold execute. destruct (event_eq_dec (current_event s) e); [rewrite N; reflexivity|contradiction]. Qed.
Lemma execute_anchors H (s t : state H) e : execute s e = Advanced t -> anchors t = anchors s.
Proof.
  unfold execute. destruct (event_eq_dec (current_event s) e); try discriminate.
  destruct (storage_at e) as [[b l]|]; try discriminate. destruct b.
  - destruct (storage_live s l) as [u|] eqn:E; try discriminate.
    intro F. inversion F; subst. exact (live_anchors H s u l E).
  - destruct (storage_dead s l) as [u|] eqn:E; try discriminate.
    intro F. inversion F; subst. exact (dead_anchors H s u l E).
Qed.
Theorem execute_caller_frame H (s t : state H) e : execute s e = Advanced t ->
  (forall a, physical_cells t (Caller a) = physical_cells s (Caller a)) /\
  original_operand t = original_operand s /\ incoming_history t = incoming_history s.
Proof.
  intro E. pose proof (execute_anchors H s t e E) as A.
  unfold anchors in A. inversion A. repeat split; try congruence. intro a. simpl. congruence.
Qed.
(* Pointwise StorageDead progress at the actual root sites, conditional only on
   the current local being immediate or already dead. Reachability past the
   unsupported reborrow/call is NOT asserted. *)
Lemma storage_dead_immediate_progress H (s : state H) e l v :
  current_event s = e -> storage_at e = Some (false,l) -> l <> 0%nat ->
  locals s l = Some (Immediate v) ->
  execute s e = Advanced (advance (set_local s l Dead)).
Proof.
  intros C S N L. unfold execute. destruct (event_eq_dec (current_event s) e); [|contradiction].
  rewrite S. destruct l; [contradiction|]. simpl. rewrite L. reflexivity.
Qed.
Theorem root_storage_dead_sites_progress H (s : state H) v :
  (current_event s = MirOperation 0 0 (Some 5) -> locals s 4 = Some (Immediate v) ->
    execute s (MirOperation 0 0 (Some 5)) = Advanced (advance (set_local s 4 Dead))) /\
  (current_event s = MirOperation 0 1 (Some 0) -> locals s 3 = Some (Immediate v) ->
    execute s (MirOperation 0 1 (Some 0)) = Advanced (advance (set_local s 3 Dead))) /\
  (current_event s = MirOperation 0 1 (Some 2) -> locals s 2 = Some (Immediate v) ->
    execute s (MirOperation 0 1 (Some 2)) = Advanced (advance (set_local s 2 Dead))).
Proof. repeat split; intros C L; eapply storage_dead_immediate_progress; try eassumption; discriminate || reflexivity. Qed.
Theorem storage_dead_reference_preserves_caller H (s : state H) e l r t :
  current_event s = e -> storage_at e = Some (false,l) -> l <> 0%nat ->
  locals s l = Some (Immediate (Init (RefValue r))) ->
  execute s e = Advanced t -> forall a, caller_cells t a = caller_cells s a.
Proof.
  intros C S N L E a. pose proof (execute_anchors H s t e E) as A.
  unfold anchors in A. inversion A. congruence.
Qed.

Theorem prefix_all_enabled_outcomes H r caller (history : H) stack fresh trace :
  let s := root_entry r caller history stack fresh trace in
  (forall o, operational_step s (MirOperation 0 0 (Some 0)) o -> o = Continue (prefix_one s)) /\
  (forall o, operational_step (prefix_one s) (MirOperation 0 0 (Some 1)) o -> o = Continue (prefix_two s)) /\
  (forall o, operational_step (prefix_two s) (MirOperation 0 0 (Some 2)) o -> o = Continue (prefix_three s)).
Proof. repeat split; intros o E; exact E. Qed.

Lemma prefix_history_effects H r caller (history : H) stack fresh trace :
  model_history (prefix_three (root_entry r caller history stack fresh trace)) =
    (history, ((trace ++ [LocalWriteHook 2]) ++ [LocalWriteHook 3]) ++ [LocalWriteHook 4]).
Proof. reflexivity. Qed.
