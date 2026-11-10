From lrust.lifetime Require Import lifetime_full.
From lrust.typing Require Import shr_bor.
From ModelSharedByteRead Require Export shared_read.
From lrust.util Require Import non_atomic_cell_map.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section observation.
  Context `{!typeG Σ}.

  Lemma shared_key_heap_observe n κ q x d g tid E L (i : nat) σ (Frame : iProp Σ) :
    (i < n)%nat → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap n κ q x d g tid -∗ Frame -∗ heap_ctx σ -∗ £(g+1)
    ={⊤}=∗ heap_ctx σ ∗ llctx_interp L ∗
      shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜∃ (z : Z) r, byte_list n x !! i = Some z ∧
        σ.1 !! (q +ₗ i) = Some (RSt r, #z) ∧ 0 ≤ z < 256⌝.
  Proof.
    iIntros (Index Alive) "#LFT #E L #Cap Frame Heap Credit".
    iPoseProof "Cap" as "[#View %Range]".
    destruct (lookup_lt_is_Some_2 (byte_list n x) i) as [z Lookup].
    { by rewrite byte_list_length. }
    have Ranged : 0 ≤ z < 256 := Forall_lookup_1 _ _ _ _ Range Lookup.
    iDestruct (Alive with "L E") as "#Alive".
    iPoseProof "View" as "V".
    iEval (unfold shared_key_view, ty_own) in "V".
    destruct d as [|d]; first by iDestruct "V" as "[[] _]".
    iDestruct "V" as "[[#Physical _] _]".
    iDestruct (guards_transitive_right with "Alive Physical") as "GP".
    iDestruct (guards_transitive_left with "GP []") as "IndexGuard".
    { iApply bytes_index_guard. exact Lookup. }
    iDestruct (guards_weaken_mask _ (↑naN) with "IndexGuard") as "IndexGuard'".
    { solve_ndisj. }
    iDestruct "Heap" as (hF) "(Map & Free & %Rel & Atomic)".
    iMod (atomic_read with "Credit [IndexGuard'] L Map") as (r) "[%Cell [L Map]]".
    { solve_ndisj. }
    { rewrite /heap_mapsto_cells_fancy /fv2sum. iFrame "IndexGuard'". }
    iModIntro. iFrame "L Cap Frame". iSplit.
    - iExists hF. iFrame. done.
    - iPureIntro. exists z, r. done.
  Qed.
End observation.
