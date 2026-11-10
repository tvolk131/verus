From lrust.typing Require Import shr_bor.
From lrust.util Require Import non_atomic_cell_map.
From lrust.lifetime Require Import lifetime_full.
From ModelSharedByteRead Require Export session.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.
Typeclasses Opaque shared_key_view.

Section session.
  Context `{!typeG Σ}.

  Definition key_reader_na n κ q (x : ~~(bytes_sort n)) d g tid (R : iProp Σ) : iProp Σ :=
    □ (∀ (i : nat) (Frame : iProp Σ), ⌜(i < n)%nat⌝ -∗ R -∗ Frame -∗ £(3*(g+1)) -∗
      WP Read Na1Ord #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
        ⌜byte_list n x !! i = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
        R ∗ shared_key_cap n κ q x d g tid ∗ Frame }}).

  Global Instance key_reader_na_persistent n κ q x d g tid R :
    Persistent (key_reader_na n κ q x d g tid R).
  Proof. apply _. Qed.

  Typeclasses Opaque key_reader_na.

  Lemma key_reader_na_read n κ q x d g tid R (i : nat) (Frame : iProp Σ) :
    (i < n)%nat → key_reader_na n κ q x d g tid R -∗ R -∗ Frame -∗ £(3*(g+1)) -∗
    WP Read Na1Ord #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
      ⌜byte_list n x !! i = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
      R ∗ shared_key_cap n κ q x d g tid ∗ Frame }}.
  Proof.
    rewrite /key_reader_na. iIntros (Index) "#Read R Frame Credit".
    iApply ("Read" $! i Frame with "[] R Frame Credit"). done.
  Qed.

  Lemma shared_key_read_na n κ q x d g tid E L (i : nat) (Frame : iProp Σ) :
    (i < n)%nat → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap n κ q x d g tid -∗ Frame -∗ £(3*(g+1)) -∗
    WP Read Na1Ord #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
      ⌜byte_list n x !! i = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
      llctx_interp L ∗ shared_key_cap n κ q x d g tid ∗ Frame }}.
  Proof.
    iIntros (Index Alive) "#LFT #E L #Cap Frame Credit".
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
    iApply (wp_read_na_guarded_cells _ _ [] (llctx_interp L) (#z) (g+1)
      with "[L Credit]"); first solve_ndisj.
    { iFrame "IndexGuard' L Credit". }
    iIntros "!> L". iExists z. iFrame "L Cap Frame". done.
  Qed.


  Lemma shared_key_reader_na_session_prepared n κ q x d g tid E L :
    lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap n κ q x d g tid ={⊤}=∗
      ∃ (R G : iProp Σ), ⌜Timeless R ∧ Timeless G⌝ ∗
      R ∗ G ∗ shared_key_cap n κ q x d g tid ∗
      (G &&{↑NllftG}&&> llctx_interp L) ∗
      key_reader_na n κ q x d g tid R ∗
      (R -∗ G ={⊤}=∗ llctx_interp L).
  Proof.
    iIntros (Alive) "#LFT #E L #Cap".
    iMod (llctx_interp_make_guarded with "L") as (γ) "[Reader [Pad [#GuardL #Close]]]";
      first solve_ndisj.
    iMod (lctx_lft_alive_get_guards _ _ _ _ _ Alive with "GuardL E Reader")
      as "[Reader #GuardK]"; first solve_ndisj.
    iDestruct (guards_transitive with "GuardL GuardK") as "#AliveGuard".
    iAssert (key_reader_na n κ q x d g tid (fractional.half γ)) as "#Read".
    { rewrite /key_reader_na. iModIntro. iIntros (i Frame Index) "Reader Frame Credit".
      iPoseProof "Cap" as "[#V %Range]".
      destruct (lookup_lt_is_Some_2 (byte_list n x) i) as [z Lookup].
      { by rewrite byte_list_length. }
      have Ranged : 0 ≤ z < 256 := Forall_lookup_1 _ _ _ _ Range Lookup.
      iPoseProof "V" as "Raw".
      iEval (unfold shared_key_view, ty_own) in "Raw".
      destruct d as [|d]; first by iDestruct "Raw" as "[[] _]".
      iDestruct "Raw" as "[[#Physical _] _]".
      iDestruct (guards_transitive_right with "AliveGuard Physical") as "GP".
      iDestruct (guards_transitive_left with "GP []") as "IndexGuard".
      { iApply bytes_index_guard. exact Lookup. }
      iDestruct (guards_weaken_mask _ (↑naN) with "IndexGuard") as "IndexGuard'";
        first solve_ndisj.
      iApply (wp_read_na_guarded_cells _ _ [] (fractional.half γ) (#z) (g+1)
        with "[Reader Credit]"); first solve_ndisj.
      { iFrame "IndexGuard' Reader Credit". }
      iIntros "!> Reader". iExists z. iFrame "Reader Cap Frame". done. }
    iModIntro. iExists (fractional.half γ), (fractional.half γ).
    iFrame "Reader Pad Cap GuardL Read". iSplit; first by iPureIntro; split; apply _.
    iIntros "Reader Pad". iApply (fupd_mask_mono with "[Reader Pad]"); first solve_ndisj.
    iApply ("Close" with "Reader Pad").
  Qed.

  Lemma shared_key_reader_na_session n κ q x d g tid E L :
    lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_view n κ q x d g tid -∗ £(g+1) ={⊤}=∗
      ∃ (R G : iProp Σ), ⌜Timeless R ∧ Timeless G⌝ ∗
      R ∗ G ∗ shared_key_cap n κ q x d g tid ∗
      (G &&{↑NllftG}&&> llctx_interp L) ∗
      key_reader_na n κ q x d g tid R ∗
      (R -∗ G ={⊤}=∗ llctx_interp L).
  Proof.
    iIntros (Alive) "#LFT #E L #View Credit".
    iMod (shared_key_prepare with "LFT E L View Credit") as "[L #Cap]"; first done.
    iApply (shared_key_reader_na_session_prepared with "LFT E L Cap"). done.
  Qed.

  Lemma key_reader_na_aliases n κ q x d g tid R :
    key_reader_na n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    key_reader_na n κ q x d g tid R ∗ key_reader_na n κ q x d g tid R ∗
    shared_key_cap n κ q x d g tid ∗ shared_key_cap n κ q x d g tid.
  Proof. iIntros "#Reader #Cap". by iFrame "Reader Cap". Qed.

  Lemma key_reader_na_read_close n κ q x d g tid (R G : iProp Σ) L i (Frame : iProp Σ) :
    (i < n)%nat →
    key_reader_na n κ q x d g tid R -∗
    (R -∗ G ={⊤}=∗ llctx_interp L) -∗
    R -∗ G -∗ Frame -∗ £(3*(g+1)) -∗
    WP Read Na1Ord #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
      ⌜byte_list n x !! i = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
      llctx_interp L ∗ shared_key_cap n κ q x d g tid ∗ Frame }}.
  Proof.
    iIntros (Index) "#Reader Close R G Frame Credit".
    iApply wp_fupd. iApply (wp_wand with "[R Frame Credit]").
    { iApply (key_reader_na_read with "Reader R Frame Credit"). done. }
    iIntros (v) "[%z [Facts [R [Cap Frame]]]]".
    iMod ("Close" with "R G") as "L". iModIntro.
    iExists z. iFrame.
  Qed.
End session.
