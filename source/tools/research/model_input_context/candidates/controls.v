From ModelInputContext Require Export context.
From NativeHeapInitialization Require Import controls.
From lrust.typing Require Import uniq_util.
Set Default Proof Using "Type".

(* A mathematical assignment, used only to refute universal future equations.
   This allocates no prophecy token and asserts no observation. *)
Definition assignment_at (ξ : proph_var) (a : ξ.(pv_ty)) : proph_asn :=
  λ ζ, match decide (ξ = ζ) with
    | left E => eq_rect ξ (λ η, η.(pv_ty)) a ζ E
    | right _ => inhabitant ζ
    end.

Lemma assignment_at_lookup ξ a : assignment_at ξ a ξ = a.
Proof. rewrite /assignment_at. destruct (decide (ξ = ξ)); [by simpl_eq|done]. Qed.

Lemma nonempty_future_not_constant (head : Z) tail p ξi idx :
  let x := bytes_of_list (head :: tail) in
  let m : ~~(uniq_borₛ (bytes_sort (length (head :: tail)))) := ((p, repeat [] (length (head :: tail))),x,ξi,0%nat,0%nat,idx) in
  ∃ π0 π1, @uniq_bor_future (bytes_sort (length (head :: tail))) m π0 ≠
            @uniq_bor_future (bytes_sort (length (head :: tail))) m π1.
Proof.
  intros x m. set ξ := uniq_prvar x ξi.
  set y := @vπ (bytes_sort (length tail)) (bytes_of_list tail) inhabitant.
  exists (assignment_at ξ ((0%Z,y),(1%positive,1%positive))),
         (assignment_at ξ ((1%Z,y),(1%positive,1%positive))).
  rewrite /uniq_bor_future /m /=.
  change ((assignment_at ξ ((0%Z,y),(1%positive,1%positive)) ξ).1 ≠
         (assignment_at ξ ((1%Z,y),(1%positive,1%positive)) ξ).1).
  rewrite !assignment_at_lookup. intros H. inversion H.
Qed.

Lemma nonempty_future_not_assumed_current (head : Z) tail p ξi idx :
  let x := bytes_of_list (head :: tail) in
  let m : ~~(uniq_borₛ (bytes_sort (length (head :: tail)))) :=
      ((p, repeat [] (length (head :: tail))),x,ξi,0%nat,0%nat,idx) in
  ¬ (∀ π, @uniq_bor_future (bytes_sort (length (head :: tail))) m π =
      @vπ (bytes_sort (length (head :: tail)))
        (@uniq_bor_current (bytes_sort (length (head :: tail))) m) π).
Proof.
  intros x m Always.
  destruct (nonempty_future_not_constant head tail p ξi idx) as [π0 [π1 Ne]].
  apply Ne. rewrite (Always π0) (Always π1). apply bytes_observation_independent.
Qed.

Section resources.
  Context `{!typeG Σ}.

  Lemma wrong_reference_pointer zs p q tid κ ξi idx : p ≠ q →
    ty_own (&uniq{κ} (bytes_ty (length zs)))
      ((p, repeat [] (length zs)), bytes_of_list zs, ξi,0%nat,0%nat,idx) 1 0 tid [FVal #q] -∗ False.
  Proof. iIntros (Ne) "[_ %Phys]". simpl in Phys. inversion Phys. congruence. Qed.

  Lemma wrong_current_bytes zs (x : ~~(bytes_sort (length zs))) :
    byte_list (length zs) x ≠ zs → x ≠ bytes_of_list zs.
  Proof. intros Ne ->. by rewrite byte_list_of_list in Ne. Qed.

  Lemma cannot_duplicate_nonempty_payload p (z : Z) zs :
    p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> (z :: zs)) -∗
    p ↦∗ ((λ z : Z, LitV (LitInt z)) <$> (z :: zs)) -∗ False.
  Proof.
    rewrite /= !heap_mapsto_vec_cons. iIntros "[P _] [Q _]".
    rewrite /heap_mapsto. iApply (duplicate_same_namespace with "P Q").
  Qed.

  Lemma cannot_duplicate_outside_frame f p v : f !! p = Some v →
    initial_fragments f -∗ initial_fragments f -∗ False.
  Proof. intros H. by apply (overlapping_frame_refused f f p v v H H). Qed.
End resources.
