From Coq Require Import List ZArith Bool Arith.PeanoNat Lia.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage Prefix.
Import ListNotations.
Local Open Scope nat_scope.

(* Conservative, tagged, byte-stack MODEL; not SB2019 or Tree Borrows. *)
Inductive permission := Reserved | Active | Frozen | Disabled.
Record grant := Grant { grant_tag : nat; grant_permission : permission; protected : bool }.
Record node := Node { node_tag : nat; parent_tag : option nat;
  node_allocation : nat; node_start : Z; node_length : nat }.
Record region := Region { region_id : nat; physical_base : Z; region_length : nat; region_live : bool }.
Inductive history_event := ChildAllocated (child parent : nat) (r : reference) (n : nat)
  | ByteRetagged (a : address) (child parent : nat)
  | LocalAssigned (l : nat) (r : reference).
Record history (H : Type) := History {
  prior_history : H;
  regions : list region;
  nodes : list node;
  byte_grants : address -> list grant;
  next_tag : nat;
  reborrow_trace : list history_event
}.
Arguments History {H}.
Arguments prior_history {H}.
Arguments regions {H}.
Arguments nodes {H}.
Arguments byte_grants {H}.
Arguments next_tag {H}.
Arguments reborrow_trace {H}.
Definition child_reference (r : reference) (tag : nat) :=
  Reference (allocation r) (byte_offset r) tag.
Definition selected_site := MirOperation 0 0 (Some 3).
Definition after_site := Control 0 0 (Some 4).
Lemma child_same_subobject r tag : same_subobject r (child_reference r tag).
Proof. split; reflexivity. Qed.

Definition address_eq_dec (a b : address) : {a=b}+{a<>b}.
Proof. decide equality; [apply Z.eq_dec|apply Nat.eq_dec]. Defined.
Definition reference_eq_dec (a b : reference) : {a=b}+{a<>b}.
Proof. decide equality; try apply Nat.eq_dec; apply Z.eq_dec. Defined.
Fixpoint find_node (tag : nat) (ns : list node) : option node :=
  match ns with [] => None | x::xs =>
    if Nat.eqb tag (node_tag x) then Some x else find_node tag xs end.
Definition in_node (x : node) (a : address) :=
  Nat.eqb (fst a) (node_allocation x) &&
  Z.leb (node_start x) (snd a) &&
  Z.ltb (snd a) (node_start x + Z.of_nat (node_length x))%Z.
Definition mutable_permission (p : permission) :=
  match p with Reserved | Active => true | _ => false end.
(* The suffix is an actual root-to-parent lineage in reverse order, not
   metadata interpreted as permission. Permission is checked separately. *)
Fixpoint valid_suffix (ns : list node) (a : address) (gs : list grant) : bool :=
  match gs with [] => false | g::tail =>
    match find_node (grant_tag g) ns with
    | None => false
    | Some x => in_node x a && mutable_permission (grant_permission g) &&
      match parent_tag x,tail with
      | None,[] => true
      | Some p,h::_ => Nat.eqb p (grant_tag h) && valid_suffix ns a tail
      | _,_ => false
      end
    end
  end.
Definition no_tag (tag : nat) (gs : list grant) :=
  forallb (fun g => negb (Nat.eqb tag (grant_tag g))) gs.
Fixpoint seek_parent (ns : list node) (a : address) (parent child : nat)
    (gs : list grant) : option (list grant) :=
  match gs with [] => None | g::tail =>
    if Nat.eqb parent (grant_tag g) then
      if valid_suffix ns a gs then Some (Grant child Reserved false :: gs) else None
    else if protected g then None else seek_parent ns a parent child tail
  end.
Definition grant_byte ns a parent child gs :=
  if no_tag child gs then seek_parent ns a parent child gs else None.
(* Pure, present-state structural premises. No outcome/future predicate. *)
Definition byte_ready ns a parent child gs : Prop :=
  exists younger g ancestors,
    gs = younger ++ g::ancestors /\ grant_tag g = parent /\
    Forall (fun x => protected x = false /\ grant_tag x <> parent) younger /\
    valid_suffix ns a (g::ancestors) = true /\ no_tag child gs = true.
Definition addresses (r : reference) (n : nat) := map (ref_address r) (seq 0 n).
Definition span_ok (rs : list region) (r : reference) (n : nat) :=
  existsb (fun x => region_live x && Z.leb 0 (physical_base x) &&
    Z.leb (physical_base x + Z.of_nat (region_length x))%Z (2^64)%Z && Nat.eqb (region_id x) (allocation r) &&
    Z.leb 0 (byte_offset r) &&
    Z.leb (byte_offset r + Z.of_nat n)%Z (Z.of_nat (region_length x)) &&
    Z.ltb 0 (physical_base x + byte_offset r)%Z &&
    Z.leb (physical_base x + byte_offset r + Z.of_nat n)%Z (2^64)%Z) rs.
Definition fresh_event child (e : history_event) :=
  match e with
  | ChildAllocated c p r _ => negb (Nat.eqb child c) && negb (Nat.eqb child p) &&
      negb (Nat.eqb child (provenance_tag r))
  | ByteRetagged _ c p => negb (Nat.eqb child c) && negb (Nat.eqb child p)
  | LocalAssigned _ r => negb (Nat.eqb child (provenance_tag r))
  end.
Definition fresh_trace child events := forallb (fresh_event child) events.
Definition fresh_node ns child :=
  forallb (fun x => negb (Nat.eqb child (node_tag x))) ns.
Fixpoint prepare (ns : list node) (parent child : nat)
    (mem : address -> list grant) (as_ : list address)
    : option (list (address * list grant)) :=
  match as_ with [] => Some [] | a::tail =>
    match grant_byte ns a parent child (mem a),prepare ns parent child mem tail with
    | Some gs,Some rest => Some ((a,gs)::rest) | _,_ => None end
  end.
Fixpoint install (updates : list (address * list grant))
    (old : address -> list grant) (a : address) : list grant :=
  match updates with [] => old a | (b,gs)::tail =>
    if address_eq_dec a b then gs else install tail old a end.
Definition set_history {H} (s : state (history H)) (h : history H) :=
  State (locals s) (pc s) (original_operand s) (caller_cells s) h
    (private_stack s) (fresh_stack s) (storage_trace s).
Definition commit_reborrow {H} (s : state (history H)) r n updates :=
  let h := incoming_history s in
  let child := next_tag h in
  let r' := child_reference r child in
  let h' := History (prior_history h) (regions h)
    (Node child (Some (provenance_tag r)) (allocation r) (byte_offset r) n :: nodes h)
    (install updates (byte_grants h)) (S child)
    (reborrow_trace h ++ [ChildAllocated child (provenance_tag r) r n] ++
      map (fun a => ByteRetagged a child (provenance_tag r)) (addresses r n) ++
      [LocalAssigned 4 r']) in
  advance (set_local (emit (set_history s h') (LocalWriteHook 4))
    4 (Immediate (Init (RefValue r')))).
Inductive result (H : Type) :=
| Advanced (s : state (history H))
| Unsupported (e : event)
| InvalidControl | InvalidLocal | InvalidRange | InvalidFreshness | InvalidGrant.
Arguments Advanced {H}.
Arguments Unsupported {H}.
Arguments InvalidControl {H}.
Arguments InvalidLocal {H}.
Arguments InvalidRange {H}.
Arguments InvalidFreshness {H}.
Arguments InvalidGrant {H}.
Definition execute {H} (s : state (history H)) e n : result H :=
  if event_eq_dec (current_event s) e then
  if event_eq_dec e selected_site then
  match Storage.operand_reference s 1,locals s 4 with
  | Some r,Some (Immediate Uninit) =>
    if reference_eq_dec r (original_operand s) then
    let h := incoming_history s in
    if negb (Nat.eqb n 0) && span_ok (regions h) r n then
    if fresh_node (nodes h) (next_tag h) && fresh_trace (next_tag h) (reborrow_trace h) then
    match prepare (nodes h) (provenance_tag r) (next_tag h) (byte_grants h) (addresses r n) with
    | Some updates => Advanced (commit_reborrow s r n updates)
    | None => InvalidGrant
    end else InvalidFreshness else InvalidRange
    else InvalidLocal
  | _,_ => InvalidLocal
  end else Unsupported e else InvalidControl.
Definition step {H} (s : state (history H)) e n o : Prop := execute s e n = o.
