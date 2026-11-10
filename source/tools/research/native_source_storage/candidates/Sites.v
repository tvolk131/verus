From Coq Require Import List.
From NativeConstructorErasure Require Import Interface Graph.
Import ListNotations.
(* Generated from the pinned six-instance/44-operation graph. Identity only. *)
Definition storage_sites : list (event * (bool * nat)) := [
  (MirOperation 0%nat 0%nat (Some 0%nat), (true, 2%nat));
  (MirOperation 0%nat 0%nat (Some 1%nat), (true, 3%nat));
  (MirOperation 0%nat 0%nat (Some 2%nat), (true, 4%nat));
  (MirOperation 0%nat 0%nat (Some 5%nat), (false, 4%nat));
  (MirOperation 0%nat 1%nat (Some 0%nat), (false, 3%nat));
  (MirOperation 0%nat 1%nat (Some 2%nat), (false, 2%nat));
  (MirOperation 3%nat 0%nat (Some 0%nat), (true, 2%nat));
  (MirOperation 3%nat 0%nat (Some 2%nat), (true, 3%nat));
  (MirOperation 3%nat 0%nat (Some 3%nat), (true, 4%nat));
  (MirOperation 3%nat 0%nat (Some 4%nat), (true, 7%nat));
  (MirOperation 3%nat 0%nat (Some 7%nat), (false, 7%nat));
  (MirOperation 3%nat 0%nat (Some 8%nat), (true, 8%nat));
  (MirOperation 3%nat 0%nat (Some 9%nat), (true, 9%nat));
  (MirOperation 3%nat 0%nat (Some 12%nat), (false, 9%nat));
  (MirOperation 3%nat 0%nat (Some 14%nat), (false, 8%nat));
  (MirOperation 3%nat 0%nat (Some 15%nat), (false, 4%nat));
  (MirOperation 3%nat 2%nat (Some 0%nat), (true, 6%nat));
  (MirOperation 3%nat 2%nat (Some 3%nat), (false, 6%nat));
  (MirOperation 3%nat 3%nat (Some 1%nat), (false, 3%nat));
  (MirOperation 3%nat 3%nat (Some 2%nat), (false, 2%nat)) ].
Definition storage_at (e : event) : option (bool * nat) :=
  match e with
  | MirOperation 0 0 (Some 0) => Some (true, 2%nat)
  | MirOperation 0 0 (Some 1) => Some (true, 3%nat)
  | MirOperation 0 0 (Some 2) => Some (true, 4%nat)
  | MirOperation 0 0 (Some 5) => Some (false, 4%nat)
  | MirOperation 0 1 (Some 0) => Some (false, 3%nat)
  | MirOperation 0 1 (Some 2) => Some (false, 2%nat)
  | MirOperation 3 0 (Some 0) => Some (true, 2%nat)
  | MirOperation 3 0 (Some 2) => Some (true, 3%nat)
  | MirOperation 3 0 (Some 3) => Some (true, 4%nat)
  | MirOperation 3 0 (Some 4) => Some (true, 7%nat)
  | MirOperation 3 0 (Some 7) => Some (false, 7%nat)
  | MirOperation 3 0 (Some 8) => Some (true, 8%nat)
  | MirOperation 3 0 (Some 9) => Some (true, 9%nat)
  | MirOperation 3 0 (Some 12) => Some (false, 9%nat)
  | MirOperation 3 0 (Some 14) => Some (false, 8%nat)
  | MirOperation 3 0 (Some 15) => Some (false, 4%nat)
  | MirOperation 3 2 (Some 0) => Some (true, 6%nat)
  | MirOperation 3 2 (Some 3) => Some (false, 6%nat)
  | MirOperation 3 3 (Some 1) => Some (false, 3%nat)
  | MirOperation 3 3 (Some 2) => Some (false, 2%nat)
  | _ => None
  end.
Lemma storage_sites_count : length storage_sites = 20%nat.
Proof. reflexivity. Qed.
Lemma storage_sites_in_graph : forall e b l,
  In (e,(b,l)) storage_sites -> In e graph_events.
Proof. intros e b l E. simpl in E. repeat destruct E as [E|E]; try contradiction;
  inversion E; subst; simpl; intuition congruence. Qed.
Lemma root_live_sites :
  storage_at (MirOperation 0 0 (Some 0)) = Some (true,2%nat) /\
  storage_at (MirOperation 0 0 (Some 1)) = Some (true,3%nat) /\
  storage_at (MirOperation 0 0 (Some 2)) = Some (true,4%nat).
Proof. repeat split; reflexivity. Qed.
Lemma root_dead_sites :
  storage_at (MirOperation 0 0 (Some 5)) = Some (false,4%nat) /\
  storage_at (MirOperation 0 1 (Some 0)) = Some (false,3%nat) /\
  storage_at (MirOperation 0 1 (Some 2)) = Some (false,2%nat).
Proof. repeat split; reflexivity. Qed.
