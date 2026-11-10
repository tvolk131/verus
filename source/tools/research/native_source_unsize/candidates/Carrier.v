From Coq Require Import List ZArith Bool Lia.
From NativeConstructorErasure Require Import Interface.
From NativeSourceStorage Require Import Storage.
Import ListNotations.

(* Carrier construction only: validation, retagging and write hooks are pending. *)
Definition metadata_ok (n : nat) : bool := Z.ltb (Z.of_nat n) (2 ^ 64).
Definition array_to_slice (v : value) (n : nat) : option value :=
  if metadata_ok n then
    match v with RefValue r => Some (SliceValue r n) | _ => None end
  else None.
Definition carrier_store {H} (s : state H) (src dst n : nat) : option (state H) :=
  if Nat.eqb src dst then None else
  match locals s src, locals s dst with
  | Some (Immediate (Init v)), Some (Immediate Uninit) =>
    match array_to_slice v n with
    | Some payload => Some (set_local s dst (Immediate (Init payload)))
    | None => None
    end
  | _, _ => None
  end.

Lemma metadata_ok_spec n : metadata_ok n = true <->
  (0 <= Z.of_nat n < 2 ^ 64)%Z.
Proof. unfold metadata_ok. rewrite Z.ltb_lt. pose proof (Nat2Z.is_nonneg n). lia. Qed.
Lemma metadata_ten : metadata_ok 10 = true.
Proof. reflexivity. Qed.
Lemma metadata_128 : metadata_ok 128 = true.
Proof. reflexivity. Qed.
Lemma carrier_pointer n r : metadata_ok n = true ->
  array_to_slice (RefValue r) n = Some (SliceValue r n).
Proof. intro E. unfold array_to_slice. rewrite E. reflexivity. Qed.
Lemma raw_is_not_array_reference n r : array_to_slice (RawValue r) n = None.
Proof. unfold array_to_slice. destruct (metadata_ok n); reflexivity. Qed.
Lemma slice_is_not_array_reference n r k : array_to_slice (SliceValue r k) n = None.
Proof. unfold array_to_slice. destruct (metadata_ok n); reflexivity. Qed.
Lemma unrepresentable_rejected n v : metadata_ok n = false -> array_to_slice v n = None.
Proof. intro E. unfold array_to_slice. rewrite E. reflexivity. Qed.
Lemma carrier_output n v out : array_to_slice v n = Some out ->
  exists r, v = RefValue r /\ out = SliceValue r n /\ metadata_ok n = true.
Proof.
  unfold array_to_slice. destruct (metadata_ok n) eqn:N; [|discriminate].
  destruct v; try discriminate. intro E. inversion E; subst.
  eexists. repeat split; reflexivity || assumption.
Qed.
Lemma carrier_progress H (s : state H) src dst n r :
  src <> dst -> locals s src = Some (Immediate (Init (RefValue r))) ->
  locals s dst = Some (Immediate Uninit) -> metadata_ok n = true ->
  carrier_store s src dst n =
    Some (set_local s dst (Immediate (Init (SliceValue r n)))).
Proof.
  intros N S D M. unfold carrier_store. apply Nat.eqb_neq in N.
  rewrite N, S, D, (carrier_pointer n r M). reflexivity.
Qed.
Lemma carrier_characterization H (s t : state H) src dst n :
  carrier_store s src dst n = Some t -> exists r,
    src <> dst /\ locals s src = Some (Immediate (Init (RefValue r))) /\
    locals s dst = Some (Immediate Uninit) /\ metadata_ok n = true /\
    t = set_local s dst (Immediate (Init (SliceValue r n))).
Proof.
  unfold carrier_store. destruct (Nat.eqb src dst) eqn:N; [discriminate|].
  destruct (locals s src) as [[|[|v]|k]|] eqn:S; try discriminate.
  destruct (locals s dst) as [[|[|w]|j]|] eqn:D; try discriminate.
  destruct (array_to_slice v n) as [payload|] eqn:P; try discriminate.
  intro E. inversion E; subst.
  destruct (carrier_output n v payload P) as [r [V [O M]]]. subst.
  exists r. apply Nat.eqb_neq in N. repeat split; assumption || reflexivity.
Qed.
Lemma carrier_destination H (s t : state H) src dst n :
  carrier_store s src dst n = Some t -> exists r,
    locals s src = Some (Immediate (Init (RefValue r))) /\
    locals t dst = Some (Immediate (Init (SliceValue r n))).
Proof.
  intro E. destruct (carrier_characterization H s t src dst n E)
    as [r [N [S [D [M T]]]]]. subst. exists r. split; [assumption|apply update_same].
Qed.
Lemma carrier_other_local H (s t : state H) src dst n k :
  carrier_store s src dst n = Some t -> k <> dst -> locals t k = locals s k.
Proof.
  intros E N. destruct (carrier_characterization H s t src dst n E)
    as [r [_ [_ [_ [_ T]]]]]. subst. apply update_other. assumption.
Qed.
Lemma carrier_source_preserved H (s t : state H) src dst n :
  carrier_store s src dst n = Some t -> locals t src = locals s src.
Proof.
  intro E. destruct (carrier_characterization H s t src dst n E) as [r [N _]].
  exact (carrier_other_local H s t src dst n src E N).
Qed.
Lemma carrier_anchors H (s t : state H) src dst n :
  carrier_store s src dst n = Some t -> anchors t = anchors s.
Proof.
  intro E. destruct (carrier_characterization H s t src dst n E)
    as [r [_ [_ [_ [_ T]]]]]. subst. reflexivity.
Qed.
Lemma carrier_all_cells H (s t : state H) src dst n :
  carrier_store s src dst n = Some t -> forall a, physical_cells t a = physical_cells s a.
Proof.
  intro E. destruct (carrier_characterization H s t src dst n E)
    as [r [_ [_ [_ [_ T]]]]]. subst. intros [a|k off]; reflexivity.
Qed.
Lemma carrier_pending_control H (s t : state H) src dst n :
  carrier_store s src dst n = Some t -> pc t = pc s /\
    private_stack t = private_stack s /\ fresh_stack t = fresh_stack s /\
    storage_trace t = storage_trace s /\ incoming_history t = incoming_history s.
Proof.
  intro E. destruct (carrier_characterization H s t src dst n E)
    as [r [_ [_ [_ [_ T]]]]]. subst. repeat split; reflexivity.
Qed.
Lemma same_local_rejected H (s : state H) src n : carrier_store s src src n = None.
Proof. unfold carrier_store. rewrite Nat.eqb_refl. reflexivity. Qed.
Lemma uninitialized_source_rejected H (s : state H) src dst n :
  locals s src = Some (Immediate Uninit) -> carrier_store s src dst n = None.
Proof. intro E. unfold carrier_store. rewrite E. destruct (Nat.eqb src dst); reflexivity. Qed.
Lemma dead_destination_rejected H (s : state H) src dst n :
  locals s dst = Some Dead -> carrier_store s src dst n = None.
Proof.
  intro E. unfold carrier_store. rewrite E. destruct (Nat.eqb src dst); [reflexivity|].
  destruct (locals s src) as [[|[|v]|k]|]; reflexivity.
Qed.
Lemma ten_byte_carrier H (s : state H) src dst r :
  src <> dst -> locals s src = Some (Immediate (Init (RefValue r))) ->
  locals s dst = Some (Immediate Uninit) ->
  carrier_store s src dst 10 = Some (set_local s dst (Immediate (Init (SliceValue r 10)))).
Proof. intros. apply carrier_progress; assumption || exact metadata_ten. Qed.
Lemma byte128_carrier H (s : state H) src dst r :
  src <> dst -> locals s src = Some (Immediate (Init (RefValue r))) ->
  locals s dst = Some (Immediate Uninit) ->
  carrier_store s src dst 128 = Some (set_local s dst (Immediate (Init (SliceValue r 128)))).
Proof. intros. apply carrier_progress; assumption || exact metadata_128. Qed.

(* A retag can preserve physical addresses without preserving the provenance tag.
   This is an algebraic control, not an implementation of reference validation. *)
Definition changed_tag (r : reference) :=
  Reference (allocation r) (byte_offset r) (S (provenance_tag r)).
Lemma changed_tag_same_subobject r : same_subobject r (changed_tag r).
Proof. split; reflexivity. Qed.
Lemma changed_tag_same_address r j : ref_address (changed_tag r) j = ref_address r j.
Proof. reflexivity. Qed.
Lemma changed_tag_distinct r : changed_tag r <> r.
Proof.
  intro E. apply (f_equal provenance_tag) in E. simpl in E. lia.
Qed.
