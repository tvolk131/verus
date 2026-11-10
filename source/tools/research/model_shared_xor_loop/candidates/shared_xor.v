From lrust.lang Require Import proofmode notation lifting heap.
From lrust.typing Require Import shr_bor.
From lrust.lifetime Require Import lifetime_full.
From ModelBufferTransform Require Export transform.
From ModelByteXor Require Export byte_xor.
From ModelSharedByteRead Require Export session.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Proof data only: this does not define or replace an executable loop. *)
Definition shared_xor_result count start pad key :=
  transform_prefix count Z.lxor pad (drop start key).
Definition shared_xor_cost (count g : nat) : nat := count * (g+2).

Lemma shared_xor_cost_step count g :
  shared_xor_cost (S count) g = ((g+1) + (1 + shared_xor_cost count g))%nat.
Proof. rewrite /shared_xor_cost. lia. Qed.
Lemma shared_xor_result_bytes count start pad key :
  Forall byte_value pad → Forall byte_value key →
  Forall byte_value (shared_xor_result count start pad key).
Proof.
  rewrite /shared_xor_result. intros RP RK.
  have RD : Forall byte_value (drop start key) := Forall_drop _ _ _ RK.
  remember (drop start key) as ys eqn:Eq in RD |- *.
  clear Eq. revert pad ys RP RD.
  induction count; intros [|a pad] [|b key'] RP RD; simpl; try done.
  inversion RP; inversion RD; subst. constructor; [by apply byte_xor_range|by apply IHcount].
Qed.
Lemma shared_xor_result_facts count start pad key :
  (count ≤ length pad)%nat → (start+count ≤ length key)%nat →
  Forall byte_value pad → Forall byte_value key →
  length (shared_xor_result count start pad key) = length pad ∧
  drop count (shared_xor_result count start pad key) = drop count pad ∧
  Forall byte_value (shared_xor_result count start pad key).
Proof.
  intros BP BK RP RK. split.
  - apply transform_prefix_length; rewrite ?length_drop; lia.
  - split; last by apply shared_xor_result_bytes.
    apply transform_prefix_suffix; rewrite ?length_drop; lia.
Qed.

Section contract.
  Context `{!typeG Σ}.
  (* Exact typed milestone, an assertion to prove, not an axiom or assumed summary.
     R is internal and linear; persistent Cap is independent of any read. *)
  Definition shared_xor_contract n κ q (x : ~~(bytes_sort n)) d g tid
      (R : iProp Σ) p (pad : list Z) (start count : nat) (Frame : iProp Σ) : iProp Σ :=
    (key_reader n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_cost count g) -∗
    WP (App (of_val zip_transform)
      [of_val byte_xor; Lit (LitInt (Z.of_nat count));
       Lit (LitLoc p); Lit (LitLoc (q +ₗ start))]) @ ⊤ {{ v,
      ⌜v = LitV LitPoison⌝ ∗
      p ↦∗ zvalues (shared_xor_result count start pad (byte_list n x)) ∗
      R ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜length (shared_xor_result count start pad (byte_list n x)) = length pad ∧
        drop count (shared_xor_result count start pad (byte_list n x)) = drop count pad ∧
        Forall byte_value (shared_xor_result count start pad (byte_list n x))⌝ }})%I.
End contract.

Lemma shared_xor_min_result n x pad :
  shared_xor_result (Nat.min (length pad) n) 0 pad (byte_list n x) =
  xor_result pad (byte_list n x).
Proof.
  rewrite /shared_xor_result /=.
  replace (Nat.min (length pad) n) with (Nat.min (length pad) (length (byte_list n x)))
    by (rewrite byte_list_length; done).
  apply transform_xor_result.
Qed.

Section operational.
  Context `{!typeG Σ}.
  Lemma wp_shared_transform_loop (op : val) n κ q x d g tid R p pad (start count : nat) (Frame : iProp Σ) :
    (count ≤ length pad)%nat → (start+count ≤ n)%nat →
    Forall byte_value pad →
    binary_operation_spec Z.lxor op -∗
    key_reader n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_cost count g) }}}
      (App (of_val zip_transform)
        [of_val op; Lit (LitInt (Z.of_nat count));
         Lit (LitLoc p); Lit (LitLoc (q +ₗ start))])
    {{{ RET LitV LitPoison;
        p ↦∗ zvalues (shared_xor_result count start pad (byte_list n x)) ∗
        R ∗ shared_key_cap n κ q x d g tid ∗ Frame }}}.
  Proof.
    revert p pad start. induction count as [|count IH]; intros p pad start BP BK RP.
    - iIntros "#Op #Reader #Cap" (Φ). iModIntro. iIntros "[Pad [R [Frame Credit]]] Post".
      wp_rec. wp_op. wp_if. iApply "Post". by iFrame.
    - destruct pad as [|a pad]; simpl in BP; first lia.
      apply Forall_cons in RP as [RA RP].
      have Index : (start < n)%nat by lia.
      destruct (lookup_lt_is_Some_2 (byte_list n x) start) as [b Lookup].
      { by (rewrite byte_list_length; done). }
      have Drop : drop start (byte_list n x) = b :: drop (S start) (byte_list n x).
      { apply drop_S. exact Lookup. }
      iIntros "#Op #Reader #Cap" (Φ). iModIntro.
      rewrite shared_xor_cost_step.
      iIntros "[Pad [R [Frame [ReadCredit [WriteCredit Credits]]]]] Post".
      wp_rec. wp_op. wp_if.
      rewrite /zvalues /= heap_mapsto_vec_cons.
      iDestruct "Pad" as "[PadHead Pad]".
      wp_bind (Read ScOrd (Lit (LitLoc p))).
      iApply (wp_read_sc with "PadHead"); first done.
      iNext. iIntros "PadHead". wp_let.
      wp_bind (Read ScOrd (Lit (LitLoc (q +ₗ start)))).
      iApply (wp_wand with "[R ReadCredit]").
      { iApply (key_reader_read n κ q x d g tid R start True with "Reader R [] ReadCredit"); done. }
      iIntros (v) "[%z [%Facts [R [#ReadCap _]]]]".
      destruct Facts as [ReadLookup [-> RB]].
      rewrite Lookup in ReadLookup. inversion ReadLookup; subst z.
      wp_let.
      wp_bind (App (of_val op) [Lit (LitInt a); Lit (LitInt b)]).
      iPoseProof ("Op" $! a b with "[] []") as "Run"; [done|done|].
      iApply (wp_wand with "Run"). iIntros (w) "%Eq". subst w.
      wp_let. wp_bind (Write ScOrd (Lit (LitLoc p)) _).
      iApply (wp_write_sc ⊤ p _ _ _ with "[PadHead WriteCredit]"); [done|by iFrame|].
      iNext. iIntros "PadHead". wp_seq. do 3 wp_op.
      replace (Z.of_nat (S count) - 1) with (Z.of_nat count) by lia.
      replace ((q +ₗ start) +ₗ 1) with (q +ₗ S start)
        by (rewrite shift_loc_assoc; f_equal; lia).
      iApply (IH (p +ₗ 1) pad (S start) with "Op Reader Cap [Pad R Frame Credits]");
        [lia|lia|done|by iFrame|].
      iNext. iIntros "[Pad [R [#OutCap Frame]]]". iApply "Post".
      rewrite /shared_xor_result Drop /= /zvalues /= heap_mapsto_vec_cons. by iFrame.
  Qed.
  Lemma wp_shared_xor_loop n κ q x d g tid R p pad (start count : nat) (Frame : iProp Σ) :
    (count ≤ length pad)%nat → (start+count ≤ n)%nat →
    Forall byte_value pad →
    key_reader n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_cost count g) }}}
      (App (of_val zip_transform)
        [of_val byte_xor; Lit (LitInt (Z.of_nat count));
         Lit (LitLoc p); Lit (LitLoc (q +ₗ start))])
    {{{ RET LitV LitPoison;
        p ↦∗ zvalues (shared_xor_result count start pad (byte_list n x)) ∗
        R ∗ shared_key_cap n κ q x d g tid ∗ Frame }}}.
  Proof.
    intros BP BK RP. iIntros "#Reader #Cap" (Φ). iModIntro. iIntros "Resources Post".
    iApply (wp_shared_transform_loop byte_xor with "[] Reader Cap Resources");
      [done|done|done|iApply wp_byte_xor|].
    iNext. iIntros "Out". by iApply "Post".
  Qed.
End operational.

Section verified_contract.
  Context `{!typeG Σ}.
  Lemma shared_xor_verified n κ q x d g tid R p pad start count Frame :
    (count ≤ length pad)%nat → (start+count ≤ n)%nat →
    Forall byte_value pad →
    ⊢ shared_xor_contract n κ q x d g tid R p pad start count Frame.
  Proof.
    intros BP BK RP. rewrite /shared_xor_contract.
    iIntros "#Reader #Cap [Pad [R [Frame Credits]]]".
    iPoseProof "Cap" as "[_ %RK]".
    iApply (wp_shared_xor_loop n κ q x d g tid R p pad start count Frame
      with "Reader Cap [Pad R Frame Credits]");
      [done|done|done|by iFrame|].
    iNext. iIntros "[Pad [R [#OutCap Frame]]]". iFrame "Pad R Cap Frame".
    iSplit; first done. iPureIntro.
    apply shared_xor_result_facts; rewrite ?byte_list_length; done.
  Qed.

  Lemma wp_shared_xor_min n κ q x d g tid R p pad (Frame : iProp Σ) :
    Forall byte_value pad →
    key_reader n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_cost (Nat.min (length pad) n) g) }}}
      (App (of_val zip_transform)
        [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min (length pad) n)));
         Lit (LitLoc p); Lit (LitLoc q)])
    {{{ RET LitV LitPoison;
      p ↦∗ zvalues (xor_result pad (byte_list n x)) ∗
      R ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜length (xor_result pad (byte_list n x)) = length pad ∧
        take (Nat.min (length pad) n) (xor_result pad (byte_list n x)) = xor_prefix pad (byte_list n x) ∧
        drop (Nat.min (length pad) n) (xor_result pad (byte_list n x)) = drop (Nat.min (length pad) n) pad ∧
        Forall byte_value (xor_result pad (byte_list n x))⌝ }}}.
  Proof.
    intros RP. iIntros "#Reader #Cap" (Φ). iModIntro. iIntros "Resources Post".
    iPoseProof "Cap" as "[_ %RK]".
    iPoseProof (wp_shared_xor_loop n κ q x d g tid R p pad 0 (Nat.min (length pad) n) Frame
      with "Reader Cap") as "Loop"; [lia|lia|done|].
    iEval (rewrite shift_loc_0) in "Loop".
    iApply ("Loop" with "Resources").
    iNext. iIntros "[Pad [R [#OutCap Frame]]]".
    rewrite shared_xor_min_result.
    iApply "Post". iFrame "Pad R Cap Frame". iPureIntro.
    replace (Nat.min (length pad) n) with (Nat.min (length pad) (length (byte_list n x)))
      by (rewrite byte_list_length; done).
    repeat split; auto using xor_result_length, xor_result_prefix, xor_result_suffix, xor_result_bytes.
  Qed.

  Lemma wp_shared_xor_128 n κ q x d g tid R p pad (Frame : iProp Σ) :
    length pad = 128%nat → Forall byte_value pad →
    key_reader n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_cost (Nat.min 128 n) g) }}}
      (App (of_val zip_transform)
        [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min 128 n)));
         Lit (LitLoc p); Lit (LitLoc q)])
    {{{ RET LitV LitPoison;
      p ↦∗ zvalues (xor_result pad (byte_list n x)) ∗
      R ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜length (xor_result pad (byte_list n x)) = 128%nat ∧
        take (Nat.min 128 n) (xor_result pad (byte_list n x)) = xor_prefix pad (byte_list n x) ∧
        drop (Nat.min 128 n) (xor_result pad (byte_list n x)) = drop (Nat.min 128 n) pad ∧
        Forall byte_value (xor_result pad (byte_list n x))⌝ }}}.
  Proof.
    intros Len RP. rewrite -Len.
    iIntros "#Reader #Cap" (Φ). iModIntro. iIntros "Resources Post".
    iApply (wp_shared_xor_min with "Reader Cap Resources"); [done|].
    iNext. iIntros "[Pad [R [#OutCap [Frame %Facts]]]]".
    iApply "Post". iFrame "Pad R Cap Frame". done.
  Qed.
End verified_contract.
