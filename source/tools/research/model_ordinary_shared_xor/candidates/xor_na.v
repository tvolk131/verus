From lrust.lang Require Import proofmode notation lifting heap.
From lrust.typing Require Import shr_bor.
From lrust.lifetime Require Import lifetime_full.
From ModelBufferTransform Require Export transform.
From ModelByteXor Require Export byte_xor.
From ModelSharedXorLoop Require Export shared_xor.
From ModelOrdinarySharedXor Require Export reader_na.
Set Default Proof Using "Type".
Open Scope Z_scope.

(* Same expression tree as frozen zip_transform, changing only three orders. *)
Definition zip_transform_na : val :=
  rec: "loop" ["op"; "n"; "pad"; "key"] :=
    if: "n" ≤ #0 then #☠ else
      let: "a" := !"pad" in
      let: "b" := !"key" in
      let: "c" := "op" ["a"; "b"] in
      "pad" <- "c";;
      "loop" ["op"; "n" - #1; "pad" +ₗ #1; "key" +ₗ #1].
Definition shared_xor_na_cost (count g : nat) : nat := count * (3*(g+1)).
Lemma shared_xor_na_cost_step count g :
  shared_xor_na_cost (S count) g = (3*(g+1) + shared_xor_na_cost count g)%nat.
Proof. rewrite /shared_xor_na_cost. lia. Qed.

Section contract.
  Context `{!typeG Σ}.
  (* Exact typed milestone, an assertion to prove, not an axiom or assumed summary.
     R is internal and linear; persistent Cap is independent of any read. *)
  Definition shared_xor_na_contract n κ q (x : ~~(bytes_sort n)) d g tid
      (R : iProp Σ) p (pad : list Z) (start count : nat) (Frame : iProp Σ) : iProp Σ :=
    (key_reader_na n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_na_cost count g) -∗
    WP (App (of_val zip_transform_na)
      [of_val byte_xor; Lit (LitInt (Z.of_nat count));
       Lit (LitLoc p); Lit (LitLoc (q +ₗ start))]) @ ⊤ {{ v,
      ⌜v = LitV LitPoison⌝ ∗
      p ↦∗ zvalues (shared_xor_result count start pad (byte_list n x)) ∗
      R ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      ⌜length (shared_xor_result count start pad (byte_list n x)) = length pad ∧
        drop count (shared_xor_result count start pad (byte_list n x)) = drop count pad ∧
        Forall byte_value (shared_xor_result count start pad (byte_list n x))⌝ }})%I.
End contract.

Section operational.
  Context `{!typeG Σ}.
  Lemma wp_shared_transform_loop_na (op : val) n κ q x d g tid R p pad (start count : nat) (Frame : iProp Σ) :
    (count ≤ length pad)%nat → (start+count ≤ n)%nat →
    Forall byte_value pad →
    binary_operation_spec Z.lxor op -∗
    key_reader_na n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_na_cost count g) }}}
      (App (of_val zip_transform_na)
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
      rewrite shared_xor_na_cost_step.
      iIntros "[Pad [R [Frame [ReadCredit Credits]]]] Post".
      wp_rec. wp_op. wp_if.
      rewrite /zvalues /= heap_mapsto_vec_cons.
      iDestruct "Pad" as "[PadHead Pad]".
      wp_bind (Read Na1Ord (Lit (LitLoc p))).
      iApply (wp_read_na with "PadHead"); first done.
      iNext. iIntros "PadHead". wp_let.
      wp_bind (Read Na1Ord (Lit (LitLoc (q +ₗ start)))).
      iApply (wp_wand with "[R ReadCredit]").
      { iApply (key_reader_na_read n κ q x d g tid R start True with "Reader R [] ReadCredit"); done. }
      iIntros (v) "[%z [%Facts [R [#ReadCap _]]]]".
      destruct Facts as [ReadLookup [-> RB]].
      rewrite Lookup in ReadLookup. inversion ReadLookup; subst z.
      wp_let.
      wp_bind (App (of_val op) [Lit (LitInt a); Lit (LitInt b)]).
      iPoseProof ("Op" $! a b with "[] []") as "Run"; [done|done|].
      iApply (wp_wand with "Run"). iIntros (w) "%Eq". subst w.
      wp_let. wp_bind (Write Na1Ord (Lit (LitLoc p)) _).
      iApply (wp_write_na ⊤ p _ _ _ with "PadHead"); [done|].
      iNext. iIntros "PadHead". wp_seq. do 3 wp_op.
      replace (Z.of_nat (S count) - 1) with (Z.of_nat count) by lia.
      replace ((q +ₗ start) +ₗ 1) with (q +ₗ S start)
        by (rewrite shift_loc_assoc; f_equal; lia).
      iApply (IH (p +ₗ 1) pad (S start) with "Op Reader Cap [Pad R Frame Credits]");
        [lia|lia|done|by iFrame|].
      iNext. iIntros "[Pad [R [#OutCap Frame]]]". iApply "Post".
      rewrite /shared_xor_result Drop /= /zvalues /= heap_mapsto_vec_cons. by iFrame.
  Qed.
  Lemma wp_shared_xor_loop_na n κ q x d g tid R p pad (start count : nat) (Frame : iProp Σ) :
    (count ≤ length pad)%nat → (start+count ≤ n)%nat →
    Forall byte_value pad →
    key_reader_na n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_na_cost count g) }}}
      (App (of_val zip_transform_na)
        [of_val byte_xor; Lit (LitInt (Z.of_nat count));
         Lit (LitLoc p); Lit (LitLoc (q +ₗ start))])
    {{{ RET LitV LitPoison;
        p ↦∗ zvalues (shared_xor_result count start pad (byte_list n x)) ∗
        R ∗ shared_key_cap n κ q x d g tid ∗ Frame }}}.
  Proof.
    intros BP BK RP. iIntros "#Reader #Cap" (Φ). iModIntro. iIntros "Resources Post".
    iApply (wp_shared_transform_loop_na byte_xor with "[] Reader Cap Resources");
      [done|done|done|iApply wp_byte_xor|].
    iNext. iIntros "Out". by iApply "Post".
  Qed.
End operational.

Section verified_contract.
  Context `{!typeG Σ}.
  Lemma shared_xor_na_verified n κ q x d g tid R p pad start count Frame :
    (count ≤ length pad)%nat → (start+count ≤ n)%nat →
    Forall byte_value pad →
    ⊢ shared_xor_na_contract n κ q x d g tid R p pad start count Frame.
  Proof.
    intros BP BK RP. rewrite /shared_xor_na_contract.
    iIntros "#Reader #Cap [Pad [R [Frame Credits]]]".
    iPoseProof "Cap" as "[_ %RK]".
    iApply (wp_shared_xor_loop_na n κ q x d g tid R p pad start count Frame
      with "Reader Cap [Pad R Frame Credits]");
      [done|done|done|by iFrame|].
    iNext. iIntros "[Pad [R [#OutCap Frame]]]". iFrame "Pad R Cap Frame".
    iSplit; first done. iPureIntro.
    apply shared_xor_result_facts; rewrite ?byte_list_length; done.
  Qed.

  Lemma wp_shared_xor_na_min n κ q x d g tid R p pad (Frame : iProp Σ) :
    Forall byte_value pad →
    key_reader_na n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_na_cost (Nat.min (length pad) n) g) }}}
      (App (of_val zip_transform_na)
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
    iPoseProof (wp_shared_xor_loop_na n κ q x d g tid R p pad 0 (Nat.min (length pad) n) Frame
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

  Lemma wp_shared_xor_na_128 n κ q x d g tid R p pad (Frame : iProp Σ) :
    length pad = 128%nat → Forall byte_value pad →
    key_reader_na n κ q x d g tid R -∗ shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame ∗ £(shared_xor_na_cost (Nat.min 128 n) g) }}}
      (App (of_val zip_transform_na)
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
    iApply (wp_shared_xor_na_min with "Reader Cap Resources"); [done|].
    iNext. iIntros "[Pad [R [#OutCap [Frame %Facts]]]]".
    iApply "Post". iFrame "Pad R Cap Frame". done.
  Qed.
End verified_contract.

Section zero_count.
  Context `{!typeG Σ}.
  (* No Reader, index lookup, pad-head split, or memory access in this branch.
     Cap remains a separate input; p and q need not be distinct. *)
  Lemma wp_shared_xor_na_zero n κ q x d g tid (R Frame : iProp Σ) p pad :
    shared_key_cap n κ q x d g tid -∗
    {{{ p ↦∗ zvalues pad ∗ R ∗ Frame }}}
      (App (of_val zip_transform_na)
        [of_val byte_xor; Lit (LitInt 0); Lit (LitLoc p); Lit (LitLoc q)])
    {{{ RET LitV LitPoison;
      p ↦∗ zvalues pad ∗ R ∗ shared_key_cap n κ q x d g tid ∗ Frame }}}.
  Proof.
    iIntros "#Cap" (Φ). iModIntro. iIntros "[Pad [R Frame]] Post".
    remember byte_xor as op eqn:OpEq.
    wp_rec. wp_op. wp_if. iApply "Post". by iFrame.
  Qed.
End zero_count.
