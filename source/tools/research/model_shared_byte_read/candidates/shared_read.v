From NativeSemanticBridge Require Export byte_array.
From lrust.typing Require Import shr_bor.
From lrust.util Require Import non_atomic_cell_map.
From guarding Require Import guard tactics.
From lrust.lifetime Require Import lifetime_full.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section shared_read.
  Context `{!typeG Σ}.

  Definition shared_key_view n κ q (x : ~~(bytes_sort n)) d g tid : iProp Σ :=
    ty_own (&shr{κ} (bytes_ty n)) ((q, repeat [] n), x) d g tid [FVal #q].

  Global Instance shared_key_view_persistent n κ q x d g tid :
    Persistent (shared_key_view n κ q x d g tid).
  Proof. destruct d; rewrite /shared_key_view /ty_own /=; apply _. Qed.

  Typeclasses Opaque shared_key_view.

  Definition shared_key_cap n κ q (x : ~~(bytes_sort n)) d g tid : iProp Σ :=
    shared_key_view n κ q x d g tid ∗
    ⌜Forall (λ z, 0 ≤ z < 256) (byte_list n x)⌝.

  Global Instance shared_key_cap_persistent n κ q x d g tid :
    Persistent (shared_key_cap n κ q x d g tid).
  Proof. apply _. Qed.

  Lemma bytes_index_guard n (x : ~~(bytes_sort n)) tid q (i : nat) (z : Z) F :
    byte_list n x !! i = Some z →
    ⊢ (q ↦[^ repeat [] n]!∗ ty_phys (bytes_ty n) x tid)
      &&{F}&&> ((q +ₗ i) ↦[^ []] #z).
  Proof.
    intros Lookup. iApply guards_point.
    - apply pt_seq_point_prop.
    - rewrite bytes_physical. revert x q i z Lookup.
      induction n as [|n IH]; intros x q i z Lookup; simpl in *.
      + destruct i; done.
      + destruct i as [|i].
        * inversion Lookup; subst. rewrite shift_loc_0. iIntros "[$ _]".
        * iIntros "[_ Tail]".
          replace (q +ₗ S i) with ((q +ₗ 1) +ₗ i) by (rewrite shift_loc_assoc; f_equal; lia).
          iApply (IH with "Tail"). done.
  Qed.

  Lemma shared_key_prepare n κ q x d g tid E L :
    lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_view n κ q x d g tid -∗ £(g+1) ={⊤}=∗
    llctx_interp L ∗ shared_key_cap n κ q x d g tid.
  Proof.
    iIntros (Alive) "#LFT #E L #View Credit".
    iDestruct (Alive with "L E") as "#Alive".
    iPoseProof "View" as "V".
    iEval (unfold shared_key_view, ty_own) in "V".
    destruct d as [|d]; first by iDestruct "V" as "[[] _]".
    iDestruct "V" as "[[#Physical [#Ghost #Pers]] _]".
    iDestruct (guards_transitive_right with "Alive Ghost") as "GG".
    leaf_open_laters "GG" with "L" as "Bytes". { solve_ndisj. }
    iMod (lc_fupd_add_laterN with "Credit Bytes") as "[Bytes Close]".
    iDestruct (bytes_initialized with "Bytes") as "%Range".
    iMod ("Close" with "[]") as "L". { by rewrite bytes_initialized. }
    iModIntro. iFrame "L View". done.
  Qed.

  Lemma shared_key_read n κ q x d g tid E L (i : nat) (Frame : iProp Σ) :
    (i < n)%nat → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_cap n κ q x d g tid -∗ Frame -∗ £(g+1) -∗
    WP Read ScOrd #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
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
    iApply (wp_read_sc_guarded_cells _ _ [] (#z) (g+1) (llctx_interp L)
      with "[L Credit]"); first solve_ndisj.
    { iFrame "IndexGuard' L Credit". }
    iIntros "!> L". iExists z. iFrame "L Cap Frame". done.
  Qed.

  Lemma shared_view_read n κ q x d g tid E L (i : nat) (Frame : iProp Σ) :
    (i < n)%nat → lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    shared_key_view n κ q x d g tid -∗ Frame -∗ £(2*(g+1)) -∗
    WP Read ScOrd #(q +ₗ i) @ ⊤ {{ v, ∃ z : Z,
      ⌜byte_list n x !! i = Some z ∧ v = #z ∧ 0 ≤ z < 256⌝ ∗
      llctx_interp L ∗ shared_key_view n κ q x d g tid ∗ Frame }}.
  Proof.
    replace (2*(g+1))%nat with ((g+1)+(g+1))%nat by lia.
    iIntros (Index Alive) "#LFT #E L #View Frame [Prepare Read]".
    iMod (shared_key_prepare with "LFT E L View Prepare") as "[L #Cap]"; first done.
    iApply (wp_wand with "[-]").
    { iApply (shared_key_read with "LFT E L Cap Frame Read"); done. }
    iIntros (v) "[%z [Facts [L [OutCap OutFrame]]]]". iExists z. iFrame "Facts L View OutFrame".
  Qed.
End shared_read.
