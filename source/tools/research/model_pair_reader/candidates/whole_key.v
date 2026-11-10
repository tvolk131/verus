From ModelPairReader Require Export handoff.
From lrust.lifetime Require Import lifetime_full.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section whole_key.
  Context `{!typeG Σ}.

  Lemma shared_key_heap_observe_prefix n κ q x d g tid E L m σ (Frame : iProp Σ) :
    (m ≤ n)%nat → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap n κ q x d g tid -∗ Frame -∗ heap_ctx σ -∗ £((g+1)*m)
    ={⊤}=∗ heap_ctx σ ∗ llctx_interp L ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜∀ i, (i < m)%nat → ∃ (z : Z) r,
        byte_list n x !! i = Some z ∧ σ.1 !! (q +ₗ i) = Some (RSt r, #z) ∧ 0 ≤ z < 256⌝.
  Proof.
    intros Bound Alive. iIntros "#LFT #E L #Cap Frame Heap Credit".
    iInduction m as [|m] "IH".
    - iModIntro. iFrame "Heap L Cap Frame". iPureIntro. intros i Hi. lia.
    - replace ((g+1)*S m)%nat with (((g+1)*m)+(g+1))%nat by lia.
      iDestruct "Credit" as "[Previous Last]".
      iMod ("IH" with "[] L Frame Heap Previous") as "[Heap [L [_ [Frame %Prefix]]]]";
        first (iPureIntro; lia).
      iMod (shared_key_heap_observe n κ q x d g tid E L m σ Frame
        with "LFT E L Cap Frame Heap Last") as "[Heap [L [_ [Frame %Cell]]]]";
        [lia|done|].
      iModIntro. iFrame "Heap L Cap Frame". iPureIntro.
      intros i Hi. destruct (decide (i = m)) as [->|Ne]; first exact Cell.
      apply Prefix. lia.
  Qed.

  Lemma whole_key_heap_observe key κ q d g tid σ (Frame : iProp Σ) :
    llft_ctx -∗ elctx_interp [] -∗ llctx_interp [κ ⊑ₗ []] -∗
    shared_key_cap (length key) κ q (bytes_of_list key) d g tid -∗
    Frame -∗ heap_ctx σ -∗ £((g+1)*length key)
    ={⊤}=∗ heap_ctx σ ∗ llctx_interp [κ ⊑ₗ []] ∗
      shared_key_cap (length key) κ q (bytes_of_list key) d g tid ∗ Frame ∗
      ⌜∀ (i : nat) (z : Z), key !! i = Some z → ∃ r, σ.1 !! (q +ₗ i) = Some (RSt r, #z) ∧ 0 ≤ z < 256⌝.
  Proof.
    iIntros "#LFT #E L #Cap Frame Heap Credit".
    iMod (shared_key_heap_observe_prefix (length key) κ q (bytes_of_list key) d g tid
      [] [κ ⊑ₗ []] (length key) σ Frame with "LFT E L Cap Frame Heap Credit")
      as "[Heap [L [_ [Frame %Cells]]]]"; [lia|apply fresh_local_alive|].
    iModIntro. iFrame "Heap L Cap Frame". iPureIntro.
    intros i z Lookup. destruct (Cells i (lookup_lt_Some _ _ _ Lookup)) as (z' & r & Eq & Cell & Range).
    rewrite byte_list_of_list in Eq. assert (z = z') by congruence. subst z'.
    exists r. done.
  Qed.
End whole_key.
