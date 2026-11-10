From NativeConstructorErasure Require Import Interface.
From Coq Require Import List.
Import ListNotations.
(* Site inventory, not a semantic interpretation of these sites. *)
Definition graph_events : list event := [
  MirOperation 0%nat 0%nat (Some 0%nat);
  MirOperation 0%nat 0%nat (Some 1%nat);
  MirOperation 0%nat 0%nat (Some 2%nat);
  MirOperation 0%nat 0%nat (Some 3%nat);
  MirOperation 0%nat 0%nat (Some 4%nat);
  MirOperation 0%nat 0%nat (Some 5%nat);
  MirOperation 0%nat 0%nat None;
  MirOperation 0%nat 1%nat (Some 0%nat);
  MirOperation 0%nat 1%nat (Some 1%nat);
  MirOperation 0%nat 1%nat (Some 2%nat);
  MirOperation 0%nat 1%nat None;
  MirOperation 1%nat 0%nat None;
  MirOperation 2%nat 0%nat None;
  MirOperation 2%nat 1%nat None;
  MirOperation 3%nat 0%nat (Some 0%nat);
  MirOperation 3%nat 0%nat (Some 1%nat);
  MirOperation 3%nat 0%nat (Some 2%nat);
  MirOperation 3%nat 0%nat (Some 3%nat);
  MirOperation 3%nat 0%nat (Some 4%nat);
  MirOperation 3%nat 0%nat (Some 5%nat);
  MirOperation 3%nat 0%nat (Some 6%nat);
  MirOperation 3%nat 0%nat (Some 7%nat);
  MirOperation 3%nat 0%nat (Some 8%nat);
  MirOperation 3%nat 0%nat (Some 9%nat);
  MirOperation 3%nat 0%nat (Some 10%nat);
  MirOperation 3%nat 0%nat (Some 11%nat);
  MirOperation 3%nat 0%nat (Some 12%nat);
  MirOperation 3%nat 0%nat (Some 13%nat);
  MirOperation 3%nat 0%nat (Some 14%nat);
  MirOperation 3%nat 0%nat (Some 15%nat);
  MirOperation 3%nat 0%nat None;
  MirOperation 3%nat 1%nat (Some 0%nat);
  MirOperation 3%nat 1%nat None;
  MirOperation 3%nat 2%nat (Some 0%nat);
  MirOperation 3%nat 2%nat (Some 1%nat);
  MirOperation 3%nat 2%nat (Some 2%nat);
  MirOperation 3%nat 2%nat (Some 3%nat);
  MirOperation 3%nat 2%nat None;
  MirOperation 3%nat 3%nat (Some 0%nat);
  MirOperation 3%nat 3%nat (Some 1%nat);
  MirOperation 3%nat 3%nat (Some 2%nat);
  MirOperation 3%nat 3%nat None;
  MirOperation 4%nat 0%nat None;
  MirOperation 5%nat 0%nat None;
  PhaseEvent CallUnwindPhase;
  PhaseEvent FakeReadPhase;
  PhaseEvent CleanupResumePhase].
Lemma graph_events_count : List.length graph_events = 47%nat.
Proof. reflexivity. Qed.
Lemma root_reservation_listed :
  In (MirOperation 0%nat 0%nat (Some 3%nat)) graph_events.
Proof. simpl. auto. Qed.
Lemma phase_events_listed :
  In (PhaseEvent FakeReadPhase) graph_events /\
  In (PhaseEvent CallUnwindPhase) graph_events /\
  In (PhaseEvent CleanupResumePhase) graph_events.
Proof. simpl. intuition congruence. Qed.
