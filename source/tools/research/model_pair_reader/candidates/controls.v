From ModelPairReader Require Export setup whole_key.
From ModelPairInput Require Import controls.
From lrust.typing Require Import type_context programs inv_context soundness.
From lrust.lifetime Require Import lifetime_full.
From lrust.lang Require Import proofmode notation lifting.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section reader_controls.
  Context `{!typeG Σ}.

  Lemma list_key_reader_read_exact key κ q d g tid R (i : nat) (z : Z) (Frame : iProp Σ) :
    key !! i = Some z →
    key_reader (length key) κ q (bytes_of_list key) d g tid R -∗
    R -∗ Frame -∗ £(g+1) -∗
    WP Read ScOrd #(q +ₗ i) @ ⊤ {{ v,
      ⌜v = #z ∧ 0 ≤ z < 256⌝ ∗ R ∗
      shared_key_cap (length key) κ q (bytes_of_list key) d g tid ∗ Frame }}.
  Proof.
    iIntros (Lookup) "#Reader R Frame Credit".
    iApply (wp_wand with "[-]").
    { iApply (key_reader_read with "Reader R Frame Credit").
      eapply lookup_lt_Some. exact Lookup. }
    iIntros (v) "[%z' [[%Eq [%Val %Range]] [R [Cap Frame]]]]".
    rewrite byte_list_of_list in Eq. assert (z = z') by congruence. subst z'.
    iFrame "R Cap Frame". done.
  Qed.

  Lemma shared_cap_aliases key κ q d g tid :
    shared_key_cap (length key) κ q (bytes_of_list key) d g tid ⊣⊢
    shared_key_cap (length key) κ q (bytes_of_list key) d g tid ∗
    shared_key_cap (length key) κ q (bytes_of_list key) d g tid.
  Proof.
    iSplit.
    - iIntros "#Cap". by iFrame "Cap".
    - iIntros "[Cap _]". done.
  Qed.

  (* This proof invokes no cell observer or read, including when q=p. *)
  Lemma whole_key_empty_no_access κ q d g tid σ (Frame : iProp Σ) :
    heap_ctx σ -∗ llctx_interp [κ ⊑ₗ []] -∗
    shared_key_cap 0 κ q (bytes_of_list []) d g tid -∗ Frame -∗
    heap_ctx σ ∗ llctx_interp [κ ⊑ₗ []] ∗
    shared_key_cap 0 κ q (bytes_of_list []) d g tid ∗ Frame ∗
    ⌜∀ (i : nat) (z : Z), ([] : list Z) !! i = Some z →
      ∃ r, σ.1 !! (q +ₗ i) = Some (RSt r, #z)⌝.
  Proof.
    iIntros "Heap L Cap Frame". iFrame.
    iPureIntro. intros i z Hi. by rewrite lookup_nil in Hi.
  Qed.
End reader_controls.

Theorem pair_reader_outside_observed h p pad q key :
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
  byte_map p pad ##ₘ byte_map q key →
  adequate NotStuck (pair_setup #0) (initialized_state h)
    (λ v σ, v = #0 ∧ frame_observed σ (pair_outside h p pad q key)).
Proof.
  intros PadRange KeyRange PadCells KeyCells Disjoint.
  apply (raw_pair_reader_observed (Σ:=typeΣ) #0 h p pad q key
    (λ Outer v, (⌜v = #0⌝ ∗ @initial_fragments typeΣ
      (@lrustGS_gen_heapGS typeΣ Outer) (pair_outside h p pad q key))%I) _ 0%nat); try done.
  - intros Outer TG tid κpad κkey R G SameOuter PadNonempty KeyNonempty. subst Outer.
    iIntros "_ _ _ _ _ _ _ _ _ R G _ _ _ _ Close Frame _".
    iMod ("Close" with "R G") as "Lkey".
    iApply wp_value. iFrame "Frame". done.
  - intros Outer σ v. iIntros "Heap [%Value Frame]".
    iDestruct (fragments_observe with "Heap Frame") as "%Outside".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done. done.
Qed.

Theorem empty_key_same_address_reader_observed h p pad :
  Forall (λ z, 0 ≤ z < 256) pad →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  adequate NotStuck (pair_setup #0) (initialized_state h)
    (λ v σ, v = #0 ∧ frame_observed σ (pair_outside h p pad p [])).
Proof.
  intros Range Cells. apply pair_reader_outside_observed; try done.
  apply byte_map_empty_alias.
Qed.

Theorem same_allocation_reader_observed (base : loc) pad key gap h :
  (0 ≤ gap)%Z →
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! ((base +ₗ 1) +ₗ i) = Some #z) →
  (∀ (i : nat) (z : Z), key !! i = Some z →
    h !! ((base +ₗ (1 + length pad + gap)) +ₗ i) = Some #z) →
  adequate NotStuck (pair_setup #0) (initialized_state h)
    (λ v σ, v = #0 ∧ frame_observed σ
      (pair_outside h (base +ₗ 1) pad (base +ₗ (1 + length pad + gap)) key)).
Proof.
  intros Gap PadRange KeyRange PadCells KeyCells.
  apply pair_reader_outside_observed; try done. by apply byte_map_same_block_gap.
Qed.

(* An actual indexed SC read supplied only by initialized raw input. The
   restoration half stays outside the call, then joins the returned R. *)
Theorem pair_reader_first_observed h p pad q z keytail :
  Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) (z :: keytail) →
  (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
  (∀ (i : nat) (w : Z), (z :: keytail) !! i = Some w → h !! (q +ₗ i) = Some #w) →
  byte_map p pad ##ₘ byte_map q (z :: keytail) →
  adequate NotStuck (pair_setup (Read ScOrd #(q +ₗ 0))) (initialized_state h)
    (λ v σ, v = #z ∧ frame_observed σ (pair_outside h p pad q (z :: keytail))).
Proof.
  intros PadRange KeyRange PadCells KeyCells Disjoint.
  apply (raw_pair_reader_observed (Σ:=typeΣ) (Read ScOrd #(q +ₗ 0)) h p pad q (z :: keytail)
    (λ Outer v, (⌜v = #z⌝ ∗ @initial_fragments typeΣ
      (@lrustGS_gen_heapGS typeΣ Outer) (pair_outside h p pad q (z :: keytail)))%I) _ 1%nat);
    try done.
  - intros Outer TG tid κpad κkey R G SameOuter PadNonempty KeyNonempty. subst Outer.
    iIntros "_ _ _ _ _ _ _ _ _ R G _ _ _ #Reader Close Frame Credit".
    iApply wp_fupd. iApply (wp_wand with "[R Frame Credit]").
    { iApply (list_key_reader_read_exact (z :: keytail) κkey q 1 0 tid R 0 z
        with "Reader R Frame Credit"). done. }
    iIntros (v) "[[%Value %Range] [R [Cap Frame]]]".
    iMod ("Close" with "R G") as "Lkey". iModIntro. iFrame "Frame". done.
  - intros Outer σ v. iIntros "Heap [%Value Frame]".
    iDestruct (fragments_observe with "Heap Frame") as "%Outside".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done. done.
Qed.
