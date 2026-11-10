From Coq Require Import List ZArith Bool Arith.PeanoNat.
From NativeConstructorErasure Require Import Interface.
Import ListNotations.

(* Authored partial MODEL. H is arbitrary incoming history, never inspected. *)
Inductive value :=
| RefValue (r : reference)
| RawValue (r : reference)
| SliceValue (r : reference) (len : nat)
| ScalarValue (z : Z)
| PairValue (a b : value).
Inductive immediate := Uninit | Init (v : value).
Inductive local := Dead | Immediate (v : immediate) | Indirect (block : nat).
Inductive cell := UninitByte | InitByte (z : Z) | PointerByte (r : reference) (part : nat).
Record stack_allocation := StackAllocation {
  stack_live : bool;
  stack_align : nat;
  stack_bytes : list cell
}.
Inductive location := Caller (a : address) | Private (block : nat) (offset : Z).
Inductive storage_effect :=
| LocalWriteHook (l : nat)
| StackAllocated (block : nat)
| StackDeallocated (block : nat).
Record control := Control { instance_id : nat; block_id : nat; statement_id : option nat }.
Record state (H : Type) := State {
  locals : nat -> option local;
  pc : control;
  original_operand : reference;
  caller_cells : address -> option cell;
  incoming_history : H;
  private_stack : nat -> option stack_allocation;
  fresh_stack : nat;
  storage_trace : list storage_effect
}.
Arguments State {H}.
Arguments locals {H}.
Arguments pc {H}.
Arguments original_operand {H}.
Arguments caller_cells {H}.
Arguments incoming_history {H}.
Arguments private_stack {H}.
Arguments fresh_stack {H}.
Arguments storage_trace {H}.

Definition update {A : Type} (f : nat -> A) (n : nat) (v : A) :=
  fun k => if Nat.eqb k n then v else f k.
Definition set_local {H} (s : state H) l v :=
  State (update (locals s) l (Some v)) (pc s) (original_operand s)
    (caller_cells s) (incoming_history s) (private_stack s) (fresh_stack s) (storage_trace s).
Definition set_control {H} (s : state H) c :=
  State (locals s) c (original_operand s) (caller_cells s) (incoming_history s)
    (private_stack s) (fresh_stack s) (storage_trace s).
Definition emit {H} (s : state H) effect :=
  State (locals s) (pc s) (original_operand s) (caller_cells s) (incoming_history s)
    (private_stack s) (fresh_stack s) (storage_trace s ++ [effect]).
Definition set_stack {H} (s : state H) mem next :=
  State (locals s) (pc s) (original_operand s) (caller_cells s) (incoming_history s)
    mem next (storage_trace s).
Definition caller_map {H} (s : state H) : physical_map := fun a =>
  match caller_cells s a with Some (InitByte z) => Some z | _ => None end.
Definition physical_cells {H} (s : state H) (a : location) : option cell :=
  match a with
  | Caller p => caller_cells s p
  | Private k z =>
    match private_stack s k with
    | Some mem => if stack_live mem && Z.leb 0 z then
        nth_error (stack_bytes mem) (Z.to_nat z) else None
    | None => None
    end
  end.
Definition operand_reference {H} (s : state H) l : option reference :=
  match locals s l with Some (Immediate (Init (RefValue r))) => Some r | _ => None end.
Definition current_event {H} (s : state H) :=
  MirOperation (instance_id (pc s)) (block_id (pc s)) (statement_id (pc s)).
Definition advance {H} (s : state H) :=
  set_control s (Control (instance_id (pc s)) (block_id (pc s))
    (option_map S (statement_id (pc s)))).
Definition model_history {H} (s : state H) := (incoming_history s, storage_trace s).
Definition anchors {H} (s : state H) :=
  (caller_cells s, original_operand s, incoming_history s).

(* This is backing-storage release, not referent Drop or loan resolution.
   None denotes malformed backing storage, not repeated live/dead. *)
Definition retire {H} (old : local) (s : state H) : option (state H) :=
  match old with
  | Indirect k =>
    match private_stack s k with
    | Some mem => if stack_live mem then
        Some (emit (set_stack s (update (private_stack s) k
          (Some (StackAllocation false (stack_align mem) (stack_bytes mem))))
          (fresh_stack s)) (StackDeallocated k)) else None
    | None => None
    end
  | _ => Some s
  end.
Definition live_sized {H} (s : state H) l :=
  set_local (emit s (LocalWriteHook l)) l (Immediate Uninit).
Definition storage_live {H} (s : state H) l : option (state H) :=
  match locals s l with
  | Some old => retire old (live_sized s l)
  | None => None
  end.
Definition storage_dead {H} (s : state H) l : option (state H) :=
  match l with
  | O => None (* rustc asserts: return place cannot be StorageDead *)
  | S _ => match locals s l with
      | Some old => retire old (set_local s l Dead)
      | None => None
      end
  end.
(* Private allocation helper for an address-taken UNINITIALIZED sized local.
   Initialized spills/encoding and actual address-taking MIR remain unsupported. *)
Definition materialize_uninit {H} (s : state H) l size align : option (state H) :=
  match locals s l, private_stack s (fresh_stack s) with
  | Some (Immediate Uninit), None =>
    Some (emit (set_local (set_stack s
      (update (private_stack s) (fresh_stack s)
        (Some (StackAllocation true align (repeat UninitByte size))))
      (S (fresh_stack s))) l (Indirect (fresh_stack s))) (StackAllocated (fresh_stack s)))
  | _, _ => None
  end.

Lemma retire_anchors H old (s t : state H) : retire old s = Some t -> anchors t = anchors s.
Proof.
  destruct old; simpl; try (intro E; inversion E; reflexivity).
  destruct (private_stack s block) as [mem|]; try discriminate.
  destruct (stack_live mem); intro E; inversion E; reflexivity.
Qed.
Lemma live_anchors H (s t : state H) l : storage_live s l = Some t -> anchors t = anchors s.
Proof.
  unfold storage_live. destruct (locals s l); try discriminate.
  intro E. exact (retire_anchors H l0 (live_sized s l) t E).
Qed.
Lemma dead_anchors H (s t : state H) l : storage_dead s l = Some t -> anchors t = anchors s.
Proof.
  destruct l; simpl; try discriminate. destruct (locals s (S l)); try discriminate.
  intro E. exact (retire_anchors H l0 (set_local s (S l) Dead) t E).
Qed.
Lemma materialize_anchors H (s t : state H) l size align :
  materialize_uninit s l size align = Some t -> anchors t = anchors s.
Proof.
  unfold materialize_uninit. destruct (locals s l) as [[|[|v]|k]|]; try discriminate;
  destruct (private_stack s (fresh_stack s)); intro E; inversion E; reflexivity.
Qed.
Lemma retire_locals H old (s t : state H) : retire old s = Some t -> locals t = locals s.
Proof.
  destruct old; simpl; try (intro E; inversion E; reflexivity).
  destruct (private_stack s block) as [mem|]; try discriminate.
  destruct (stack_live mem); intro E; inversion E; reflexivity.
Qed.
Lemma update_same A (f : nat -> A) n v : update f n v n = v.
Proof. unfold update. rewrite Nat.eqb_refl. reflexivity. Qed.
Lemma update_other A (f : nat -> A) n v k : k <> n -> update f n v k = f k.
Proof. intro E. unfold update. apply Nat.eqb_neq in E. rewrite E. reflexivity. Qed.
Lemma live_local H (s t : state H) l : storage_live s l = Some t ->
  locals t l = Some (Immediate Uninit).
Proof.
  unfold storage_live. destruct (locals s l); try discriminate. intro E.
  rewrite (retire_locals H l0 (live_sized s l) t E). apply update_same.
Qed.
Lemma dead_local H (s t : state H) l : storage_dead s l = Some t -> locals t l = Some Dead.
Proof.
  destruct l; simpl; try discriminate. destruct (locals s (S l)); try discriminate. intro E.
  rewrite (retire_locals H l0 (set_local s (S l) Dead) t E). apply update_same.
Qed.
Lemma live_other H (s t : state H) l k : storage_live s l = Some t -> k <> l ->
  locals t k = locals s k.
Proof.
  unfold storage_live. destruct (locals s l); try discriminate. intros E N.
  rewrite (retire_locals H l0 (live_sized s l) t E). apply update_other. exact N.
Qed.
Lemma dead_other H (s t : state H) l k : storage_dead s l = Some t -> k <> l ->
  locals t k = locals s k.
Proof.
  destruct l; simpl; try discriminate. destruct (locals s (S l)); try discriminate. intros E N.
  rewrite (retire_locals H l0 (set_local s (S l) Dead) t E). apply update_other. exact N.
Qed.
Lemma repeated_live H (s : state H) l v : locals s l = Some (Immediate v) ->
  storage_live s l = Some (live_sized s l).
Proof. intro E. unfold storage_live. rewrite E. reflexivity. Qed.
Lemma repeated_dead H (s : state H) l : l <> 0%nat -> locals s l = Some Dead ->
  storage_dead s l = Some (set_local s l Dead).
Proof. destruct l; [contradiction|]. intros _ E. simpl. rewrite E. reflexivity. Qed.
Lemma dead_twice_locals H (s t u : state H) l : storage_dead s l = Some t ->
  storage_dead t l = Some u -> forall k, locals u k = locals t k.
Proof.
  intros E F k. pose proof (dead_local H s t l E) as D.
  destruct l; [discriminate|]. simpl in F. rewrite D in F. inversion F; subst.
  simpl. unfold update. destruct (Nat.eqb k (S l)) eqn:B; [apply Nat.eqb_eq in B; subst; symmetry; exact D|reflexivity].
Qed.
Lemma backing_release H (s : state H) l k mem : l <> 0%nat ->
  locals s l = Some (Indirect k) -> private_stack s k = Some mem -> stack_live mem = true ->
  exists t, storage_dead s l = Some t /\
    private_stack t k = Some (StackAllocation false (stack_align mem) (stack_bytes mem)) /\
    physical_cells t (Private k 0%Z) = None /\ anchors t = anchors s.
Proof.
  destruct l; [contradiction|]. intros _ L M A.
  unfold storage_dead. rewrite L. unfold retire. simpl. rewrite M, A.
  eexists. split; [reflexivity|]. simpl. rewrite update_same. split; [reflexivity|].
  split; reflexivity.
Qed.

Lemma materialize_progress H (s : state H) l size align :
  locals s l = Some (Immediate Uninit) -> private_stack s (fresh_stack s) = None ->
  exists t, materialize_uninit s l size align = Some t /\
    locals t l = Some (Indirect (fresh_stack s)) /\
    private_stack t (fresh_stack s) = Some (StackAllocation true align (repeat UninitByte size)) /\
    fresh_stack t = S (fresh_stack s) /\ anchors t = anchors s.
Proof.
  intros L M. unfold materialize_uninit. rewrite L, M. eexists.
  split; [reflexivity|]. simpl. rewrite !update_same. repeat split; reflexivity.
Qed.
Lemma live_releases_backing H (s : state H) l k mem :
  locals s l = Some (Indirect k) -> private_stack s k = Some mem -> stack_live mem = true ->
  exists t, storage_live s l = Some t /\ locals t l = Some (Immediate Uninit) /\
    private_stack t k = Some (StackAllocation false (stack_align mem) (stack_bytes mem)) /\
    storage_trace t = storage_trace s ++ [LocalWriteHook l] ++ [StackDeallocated k] /\
    anchors t = anchors s.
Proof.
  intros L M A. unfold storage_live. rewrite L. unfold retire, live_sized. simpl. rewrite M, A.
  eexists. split; [reflexivity|]. simpl. rewrite !update_same. repeat split; try reflexivity.
  symmetry. exact (app_assoc (storage_trace s) [LocalWriteHook l] [StackDeallocated k]).
Qed.
Lemma repeated_dead_observations H (s t : state H) l :
  l <> 0%nat -> locals s l = Some Dead -> storage_dead s l = Some t ->
  (forall k, locals t k = locals s k) /\ pc t = pc s /\
    private_stack t = private_stack s /\ storage_trace t = storage_trace s /\ anchors t = anchors s.
Proof.
  intros N D E. rewrite (repeated_dead H s l N D) in E. inversion E; subst.
  split.
  - intro k. simpl. unfold update. destruct (Nat.eqb k l) eqn:B; [apply Nat.eqb_eq in B; subst; symmetry; exact D|reflexivity].
  - repeat split; reflexivity.
Qed.
