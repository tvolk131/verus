From Coq Require Import List ZArith.
Import ListNotations.
Open Scope Z_scope.

Definition address := (nat * Z)%type.
Definition physical_map := address -> option Z.
Record reference := Reference {
  allocation : nat;
  byte_offset : Z;
  provenance_tag : nat
}.
Definition ref_address (r : reference) (j : nat) : address :=
  (allocation r, byte_offset r + Z.of_nat j).

Inductive phase := FakeReadPhase | CallUnwindPhase | CleanupResumePhase.
Inductive event := MirOperation (instance block : nat) (statement : option nat)
                 | PhaseEvent (p : phase).
Inductive fault := InvalidReference | InvalidRetag | InvalidRawFormation
                 | InvalidStorage | InvalidControl | UndefinedBehavior.
Inductive outcome (C : Type) :=
| Continue (s : C)
| Normal (s : C)
| Unwind (s : C)
| Fault (why : fault).
Arguments Continue {C} s.
Arguments Normal {C} s.
Arguments Unwind {C} s.
Arguments Fault {C} why.

(* A data signature, NOT an asserted Rust semantics or safety law. No instance
   is supplied here. History remains part of the actual configuration. *)
Record source_semantics := Semantics {
  configuration : Type;
  history_type : Type;
  history_of : configuration -> history_type;
  source_heap : configuration -> physical_map;
  operand_reference : configuration -> nat -> option reference;
  lawful_entry : configuration -> Prop;
  lawful_state : configuration -> Prop;
  reference_valid : configuration -> reference -> Prop;
  entry_control : configuration -> Prop;
  at_event : configuration -> event -> Prop;
  running : configuration -> Prop;
  caller_observable : configuration -> address -> Prop;
  step : configuration -> event -> outcome configuration -> Prop
}.

Definition initialized (M : source_semantics) (s : configuration M)
    (r : reference) (A : list Z) : Prop :=
  Forall (fun b => 0 <= b < 256) A /\
  forall j b, nth_error A j = Some b ->
    source_heap M s (ref_address r j) = Some b.
Definition V_entry (M : source_semantics) (s : configuration M)
    (r : reference) (A : list Z) : Prop :=
  lawful_entry M s /\ entry_control M s /\
  operand_reference M s 1%nat = Some r /\
  length A = 128%nat /\ initialized M s r A.

Definition physical_frame (M : source_semantics)
    (s0 s1 : configuration M) : Prop :=
  forall a, caller_observable M s0 a -> source_heap M s1 a = source_heap M s0 a.

Inductive execution (M : source_semantics) :
    configuration M -> outcome (configuration M) -> Prop :=
| ExecutionZero s : execution M s (Continue s)
| ExecutionStep s e o : step M s e o -> execution M s o
| ExecutionMore s e t o : step M s e (Continue t) ->
    execution M t o -> execution M s o.

Definition same_subobject (r r' : reference) : Prop :=
  allocation r' = allocation r /\ byte_offset r' = byte_offset r.

(* Open source obligation, including existence and ALL fault outcomes. *)
Definition root_reborrow_need (M : source_semantics) : Prop :=
  forall s0 r A s, V_entry M s0 r A ->
  execution M s0 (Continue s) ->
  at_event M s (MirOperation 0%nat 0%nat (Some 3%nat)) ->
  (exists t r',
    step M s (MirOperation 0%nat 0%nat (Some 3%nat)) (Continue t) /\
    operand_reference M t 4%nat = Some r' /\ same_subobject r r' /\
    reference_valid M t r') /\
  (forall e why, step M s e (Fault why) -> False) /\
  (forall e t, step M s e (Continue t) ->
    lawful_state M t /\ forall a, source_heap M t a = source_heap M s a).
