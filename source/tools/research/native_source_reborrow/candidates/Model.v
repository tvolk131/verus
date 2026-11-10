From Coq Require Import List ZArith Bool Arith.PeanoNat Lia.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage Prefix.
Import ListNotations.

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
