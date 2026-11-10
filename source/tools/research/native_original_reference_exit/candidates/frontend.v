From Coq Require Import List ZArith.
Import ListNotations.

(* Algebraic interpretation of the EXISTING frontend equations, not a Rust
   reference representation or a new observation/source-release axiom. *)
Definition cf_current {T : Type} (r : T * T) := fst r.
Definition cf_future {T : Type} (r : T * T) := snd r.
Definition cf_update_current {T : Type} (r : T * T) (x : T) := (x, cf_future r).
Definition cf_resolved {T : Type} (r : T * T) := cf_current r = cf_future r.

Lemma merge_preserves_original_future {T} (original pending : T * T) :
  cf_future (cf_update_current original (cf_future pending)) = cf_future original.
Proof. reflexivity. Qed.

Lemma merge_resolved_future {T} (original pending : T * T) :
  cf_resolved (cf_update_current original (cf_future pending)) ->
  cf_future original = cf_future pending.
Proof. unfold cf_resolved, cf_update_current, cf_current; simpl. auto. Qed.

(* The source worker must prove BOTH whole-current physical observation and
   release binding at actual normal exit. They are explicit unproved source
   dependencies of native use, not facts obtained from iterator resolution. *)
Theorem conditional_original_exit {T} (original pending pending_exit : T * T)
    (actual_final_bytes : T) :
  cf_resolved (cf_update_current original (cf_future pending)) ->
  cf_future pending_exit = cf_future pending ->
  cf_current pending_exit = actual_final_bytes ->
  cf_resolved pending_exit ->
  cf_future original = actual_final_bytes.
Proof.
  intros Merge SameFuture Physical Release.
  rewrite (merge_resolved_future original pending Merge).
  rewrite <- SameFuture, <- Release. exact Physical.
Qed.

Corollary conditional_drop_exit {T} (original pending pending_exit : T * T)
    (initial actual_final_bytes : T) :
  cf_current original = initial -> actual_final_bytes = initial ->
  cf_resolved (cf_update_current original (cf_future pending)) ->
  cf_future pending_exit = cf_future pending ->
  cf_current pending_exit = actual_final_bytes ->
  cf_resolved pending_exit ->
  cf_future original = cf_current original.
Proof.
  intros Entry NoWrite Merge Same Physical Release.
  rewrite (conditional_original_exit original pending pending_exit actual_final_bytes
    Merge Same Physical Release), NoWrite. symmetry. exact Entry.
Qed.

(* Ordinary model's complete prefix AND suffix vector can be substituted here;
   there is no k<=128 premise and no loss of unissued bytes. *)
Corollary conditional_full_vector_exit (original pending pending_exit : list Z * list Z)
    actual_final_bytes result :
  cf_resolved (cf_update_current original (cf_future pending)) ->
  cf_future pending_exit = cf_future pending ->
  cf_current pending_exit = actual_final_bytes ->
  cf_resolved pending_exit -> actual_final_bytes = result ->
  cf_future original = result.
Proof.
  intros Merge Same Physical Release Writer.
  rewrite (conditional_original_exit original pending pending_exit actual_final_bytes
    Merge Same Physical Release). exact Writer.
Qed.

(* Logical witnesses, not executions or Rust semantic negatives. *)
Lemma inherited_minimal_is_insufficient :
  exists original pending : list Z * list Z,
    cf_current pending = cf_current original /\
    cf_resolved (cf_update_current original (cf_future pending)) /\
    cf_future original <> cf_current original.
Proof.
  exists ([0%Z], [1%Z]), ([0%Z], [1%Z]).
  repeat split; discriminate || reflexivity.
Qed.

Lemma released_child_does_not_resolve_live_parent :
  exists parent child : list Z * list Z,
    cf_resolved child /\
    cf_current (cf_update_current parent (cf_future child)) = cf_current child /\
    cf_future (cf_update_current parent (cf_future child)) = cf_future parent /\
    ~ cf_resolved (cf_update_current parent (cf_future child)).
Proof.
  exists ([0%Z], [2%Z]), ([1%Z], [1%Z]).
  repeat split; discriminate || reflexivity.
Qed.
