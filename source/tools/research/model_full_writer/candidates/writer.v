From ModelFullWriter Require Export vectors.
From ModelMutableWriter Require Import transform.
From ModelPairReader Require Import setup whole_key.
From ModelSharedXorLoop Require Import shared_xor.
From lrust.typing Require Import soundness inv_context programs_util.
From lrust.lifetime Require Import lifetime_full.
Require Import guarding.internal.na_invariants_fork.
From lrust.util Require Import atomic_lock_counter.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition writer_loop p q (pad key : list Z) : expr :=
  App (of_val zip_transform)
    [of_val byte_xor; Lit (LitInt (Z.of_nat (Nat.min (length pad) (length key))));
     Lit (LitLoc p); Lit (LitLoc q)].
Definition writer_body p q pad key : expr :=
  ((Skip;; (Newlft;; ReBor;; writer_loop p q pad key;; Endlft;; Skip);; Skip);; Endlft)%E.
Definition writer_budget (pad key : list Z) : nat :=
  1 + (shared_xor_cost (Nat.min (length pad) (length key)) 0 + length key).
Definition writer_final h p pad q key (σ : state) : Prop :=
  length (xor_result pad key) = length pad ∧
  take (Nat.min (length pad) (length key)) (xor_result pad key) = xor_prefix pad key ∧
  drop (Nat.min (length pad) (length key)) (xor_result pad key) =
    drop (Nat.min (length pad) (length key)) pad ∧
  Forall byte_value (xor_result pad key) ∧
  interval_observed σ p (zvalues (xor_result pad key)) ∧
  (∀ (i : nat) (z : Z), key !! i = Some z → heap_values σ !! (q +ₗ i) = Some #z) ∧
  frame_observed σ (pair_outside h p pad q key).

Section final_resource.
  Context `{Outer : lrustGS Σ}.
  Lemma heap_ctx_for_observation (NA : na_invG Σ) (AC : alc_logicG Σ) σ :
    @heap_ctx Σ (@lrustGS_gen_heapGS Σ Outer) (@lrustGS_invGS Σ Outer) NA AC σ -∗
    @heap_ctx Σ (@lrustGS_gen_heapGS Σ Outer) (@lrustGS_invGS Σ Outer)
      (@lrustGS_na_invGS Σ Outer) (@lrustGS_atomic_lock_ctr_invGS Σ Outer) (σ.1, true).
  Proof.
    iDestruct 1 as (hF) "(Map & Free & %Rel & _)".
    iExists hF. iFrame. done.
  Qed.

  Definition writer_observer h p pad q key : iProp Σ :=
    ∀ (NA : na_invG Σ) (AC : alc_logicG Σ) σ,
      @heap_ctx Σ (@lrustGS_gen_heapGS Σ Outer) (@lrustGS_invGS Σ Outer) NA AC σ
      ={⊤,∅}=∗ ⌜writer_final h p pad q key σ⌝.
End final_resource.

Section writer_execution.
  Context `{!typeG Σ}.

  Lemma wp_writer_body h p pad q key tid κpad κkey (R G : iProp Σ) :
    Forall byte_value pad → Forall byte_value key →
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp [] -∗
    llctx_interp [κpad ⊑ₗ []] -∗ invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference pad p tid κpad -∗ R -∗ G -∗
    shared_key_cap (length key) κkey q (bytes_of_list key) 1 0 tid -∗
    key_reader (length key) κkey q (bytes_of_list key) 1 0 tid R -∗
    (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) -∗
    initial_fragments (pair_outside h p pad q key) -∗ £(writer_budget pad key) -∗
    WP writer_body p q pad key {{ _, writer_observer h p pad q key }}.
  Proof.
    iIntros (PadRange KeyRange) "#LFT #TIME #PROPH #UNIQ #E L I Ref R G #Cap #Reader Close Frame Credits".
    iEval (rewrite /writer_budget) in "Credits".
    iDestruct "Credits" as "[Open [LoopCredits ObserveCredits]]".
    rewrite /writer_body.
    iApply (wp_wand with "[L I Ref R G Close Frame Open LoopCredits ObserveCredits]").
    { unshelve iApply (initialized_nested_transform pad p tid κpad
        (bytes_from_length (xor_result pad key) (xor_result_length pad key))
        (writer_loop p q pad key)
        (key_reader (length key) κkey q (bytes_of_list key) 1 0 tid R ∗
         shared_key_cap (length key) κkey q (bytes_of_list key) 1 0 tid ∗
         R ∗ £(shared_xor_cost (Nat.min (length pad) (length key)) 0))%I R
        (initial_fragments (pair_outside h p pad q key) ∗ G ∗
         (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) ∗ £(length key))%I
        _ _ _ with "LFT TIME PROPH UNIQ E L I Ref Open [R LoopCredits] [Frame G Close ObserveCredits]").
      - rewrite /writer_loop. solve_closed.
      - rewrite byte_list_from_length. by apply xor_result_bytes.
      - intros RangeInput. iIntros "Pad [#Reader [#Cap [R Credit]]]".
        rewrite byte_list_from_length /writer_loop.
        iApply (wp_shared_xor_min (length key) κkey q (bytes_of_list key) 1 0 tid R
          p pad True RangeInput with "Reader Cap [Pad R Credit]"); first by iFrame.
        iNext.
        iIntros "[Pad [R [_ [_ %Facts]]]]".
        iFrame "R". iEval (rewrite byte_list_of_list) in "Pad". iFrame "Pad".
      - iSplit.
        + rewrite /key_reader. iModIntro. iExact "Reader".
        + iFrame "Cap R LoopCredits".
      - iFrame "Frame G Close ObserveCredits". }
    iIntros (w) "[[Frame [G [Close ObserveCredits]]] [R Pad]]".
    iEval (rewrite byte_list_from_length) in "Pad".
    iIntros (NA AC σ) "Heap".
    iPoseProof (heap_ctx_for_observation NA AC σ with "Heap") as "Heap".
    iMod ("Close" with "R G") as "Lkey".
    iEval (replace (length key) with ((0+1)*length key)%nat by lia) in "ObserveCredits".
    iMod (whole_key_heap_observe key κkey q 1 0 tid (σ.1, true)
      (p ↦∗ zvalues (xor_result pad key) ∗ initial_fragments (pair_outside h p pad q key))%I
      with "LFT E Lkey Cap [Pad Frame] Heap ObserveCredits")
      as "[Heap [Lkey [_ [[Pad Frame] %KeyCells]]]]"; first by iFrame.
    iDestruct (interval_observe with "Heap Pad") as "%PadCells".
    iDestruct (fragments_observe with "Heap Frame") as "%Outside".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done.
    iPureIntro. rewrite /writer_final.
    repeat split; auto using xor_result_length, xor_result_prefix, xor_result_suffix, xor_result_bytes.
    intros i z Lookup. destruct (KeyCells i z Lookup) as [r [Cell Range]].
    by rewrite /heap_values lookup_fmap Cell.
  Qed.
End writer_execution.

Section initialized_writer.
  Context {Σ : gFunctors} `{!typePreG Σ}.

  Theorem initialized_shared_xor_writer h p pad q key :
    Forall byte_value pad → Forall byte_value key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    adequate NotStuck (pair_setup (writer_body p q pad key)) (initialized_state h)
      (λ _ σ, writer_final h p pad q key σ).
  Proof using All.
    intros PadRange KeyRange PadCells KeyCells Disjoint.
    apply (initialized_observation_adequacy (Σ:=Σ) _ _ _ (2 + writer_budget pad key)).
    intros Outer. iIntros "Fragments Credits Time".
    iExists (λ _, @writer_observer Σ Outer h p pad q key).
    iSplitL "Fragments Credits Time".
    - iApply (raw_pair_reader_context_wp Outer (writer_body p q pad key) h p pad q key
        (λ _, @writer_observer Σ Outer h p pad q key) (writer_budget pad key)
        with "Fragments Credits Time"); try done;
        first (unfold writer_body, writer_loop; solve_closed).
      intros TG tid κpad κkey R G SameOuter PadNonempty KeyNonempty. subst Outer.
      iIntros "#LFT #PROPH #UNIQ #TIME #E L I Ref _ R G #Cap _ _ #Reader Close Frame Credits".
      iApply (wp_writer_body h p pad q key tid κpad κkey R G PadRange KeyRange
        with "LFT TIME PROPH UNIQ E L I Ref R G Cap Reader Close Frame Credits").
    - iIntros (σ v) "Heap Observe".
      iEval (rewrite /writer_observer) in "Observe".
      by iApply ("Observe" $! _ _ σ with "Heap").
  Qed.
End initialized_writer.
