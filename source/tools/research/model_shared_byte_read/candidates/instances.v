From lrust.lifetime Require Import lifetime_full.
From lrust.typing Require Import shr_bor.
From ModelSharedByteRead Require Export observation.
From ModelInputContext Require Import vector.
Set Default Proof Using "Type".
Open Scope Z_scope.

Lemma empty_key_has_no_index (i : nat) : ¬ (i < 0)%nat.
Proof. lia. Qed.

Section instances.
  Context `{!typeG Σ}.

  Lemma shared_view_depth_zero n κ q x g tid :
    shared_key_view n κ q x 0 g tid ⊢ False.
  Proof. rewrite /shared_key_view /ty_own /=. iIntros "[[] _]". Qed.

  Lemma shared_cap_duplicate n κ q x d g tid :
    shared_key_cap n κ q x d g tid ⊢
      shared_key_cap n κ q x d g tid ∗ shared_key_cap n κ q x d g tid.
  Proof. iIntros "#Cap". iFrame "Cap". Qed.

  Lemma shared_read_same_alias n κ q x d g tid E L (i : nat) (Frame : iProp Σ) :
    (i < n)%nat → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap n κ q x d g tid -∗
    (shared_key_cap n κ q x d g tid ∗ Frame) -∗ £(g+1) -∗
    WP Read ScOrd #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
      ⌜byte_list n x !! i = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
      llctx_interp L ∗ shared_key_cap n κ q x d g tid ∗
      (shared_key_cap n κ q x d g tid ∗ Frame) }}.
  Proof. apply shared_key_read. Qed.

  Lemma shared_read129 κ q (x : ~~(bytes_sort 129)) tid E L (Frame : iProp Σ) :
    lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap 129 κ q x 1 0 tid -∗ Frame -∗ £1 -∗
    WP Read ScOrd #(q +ₗ 128) @ ⊤ {{ v, ∃ z : Z,
      ⌜byte_list 129 x !! 128%nat = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
      llctx_interp L ∗ shared_key_cap 129 κ q x 1 0 tid ∗ Frame }}.
  Proof. intros Alive. apply (shared_key_read 129 κ q x 1 0 tid E L 128 Frame); [lia|done]. Qed.

  Lemma list_shared_read zs κ q tid E L (i : nat) (z : Z) d g (Frame : iProp Σ) :
    zs !! i = Some z → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_view (length zs) κ q (bytes_of_list zs) d g tid -∗ Frame -∗ £(2*(g+1)) -∗
    WP Read ScOrd #(q +ₗ i) @ ⊤ {{ v,
      ⌜v = #z ∧ 0 ≤ z < 256⌝ ∗ llctx_interp L ∗
      shared_key_view (length zs) κ q (bytes_of_list zs) d g tid ∗ Frame }}.
  Proof.
    iIntros (Lookup Alive) "#LFT #E L #View Frame Credit".
    iApply (wp_wand with "[-]").
    { iApply (shared_view_read with "LFT E L View Frame Credit"); last done.
      by apply lookup_lt_Some with z. }
    iIntros (v) "[%z' [%Facts [L [View' Frame']]]]".
    destruct Facts as [Lookup' [Value Range]].
    rewrite byte_list_of_list Lookup in Lookup'. inversion Lookup'; subst.
    iFrame "L View Frame'". done.
  Qed.
End instances.
