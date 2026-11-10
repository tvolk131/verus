From ModelOrdinaryFullWriter Require Export vectors.
From lrust.lifetime Require Import lifetime_full.
From lrust.typing Require Import soundness inv_context programs_util.
Require Import guarding.internal.na_invariants_fork.
From lrust.util Require Import atomic_lock_counter.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section observer_type.
  Context {Σ : gFunctors} (Outer : lrustGS Σ).
  Existing Instance Outer.
  (* Only bookkeeping resources depend on NA/AC. Keep the actual heap map and
     freeability authority, and discard bookkeeping, without assuming a state. *)
  Lemma heap_ctx_for_observation (NA : na_invG Σ) (AC : alc_logicG Σ) σ :
    @heap_ctx Σ (@lrustGS_gen_heapGS Σ Outer) (@lrustGS_invGS Σ Outer) NA AC σ -∗
    @heap_ctx Σ (@lrustGS_gen_heapGS Σ Outer) (@lrustGS_invGS Σ Outer)
      (@lrustGS_na_invGS Σ Outer) (@lrustGS_atomic_lock_ctr_invGS Σ Outer) (σ.1, true).
  Proof.
    iDestruct 1 as (hF) "(Map & Free & %Rel & _)".
    iExists hF. iFrame. done.
  Qed.

  Definition ordinary_observer h p pad q key (_ : val) : iProp Σ :=
    (∀ (NA : na_invG Σ) (AC : alc_logicG Σ) σ,
      @heap_ctx Σ (@lrustGS_gen_heapGS Σ Outer) (@lrustGS_invGS Σ Outer) NA AC σ
      ={⊤,∅}=∗ ⌜ordinary_post h p pad q key #☠ σ⌝)%I.
End observer_type.

Section composition.
  Context `{!typeG Σ}.

  Lemma ordinary_session h p pad q key tid κpad κkey (R G : iProp Σ) :
    Forall byte_value pad → Forall byte_value key →
    llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
    llctx_interp [κpad ⊑ₗ []] -∗
    invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    model_reference pad p tid κpad -∗ R -∗ G -∗
    shared_key_cap (length key) κkey q (bytes_of_list key) 1 0 tid -∗
    key_reader_na (length key) κkey q (bytes_of_list key) 1 0 tid R -∗
    (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) -∗
    initial_fragments (pair_outside h p pad q key) -∗
    £(1 + shared_xor_na_cost (Nat.min (length pad) (length key)) 0 + length key) -∗
    WP ordinary_mutation p q pad key {{ v,
      ordinary_observer type_lrustGS h p pad q key v }}.
  Proof.
    iIntros (PadRange KeyRange)
      "#LFT #PROPH #UNIQ #TIME #E Lpad I Ref R G #Cap #Reader Close Outside Credits".
    replace (1 + shared_xor_na_cost (Nat.min (length pad) (length key)) 0 + length key)%nat
      with (1 + (shared_xor_na_cost (Nat.min (length pad) (length key)) 0 + length key))%nat by lia.
    iDestruct "Credits" as "[Open [LoopCredits ObserveCredits]]".
    rewrite /ordinary_mutation.
    iApply wp_fupd. iApply (wp_wand with "[Lpad I Ref Open R G Close Outside LoopCredits ObserveCredits]").
    { unshelve iApply (initialized_nested_transform pad p tid κpad (final_bytes pad key)
        (ordinary_loop p q pad key)
        (key_reader_na (length key) κkey q (bytes_of_list key) 1 0 tid R ∗
         shared_key_cap (length key) κkey q (bytes_of_list key) 1 0 tid ∗
         R ∗ £(shared_xor_na_cost (Nat.min (length pad) (length key)) 0))%I R
        (G ∗ (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) ∗
         initial_fragments (pair_outside h p pad q key) ∗ £(length key))%I
        (ordinary_loop_closed _ _ _ _) _ _
        with "LFT TIME PROPH UNIQ E Lpad I Ref Open [R LoopCredits] [G Close Outside ObserveCredits]").
      - apply final_bytes_range; done.
      - intros InputRange. iIntros "Pad [#ReaderIn [#CapIn [R LoopCredits]]]".
        rewrite /ordinary_loop. iApply (wp_shared_xor_na_min (length key) κkey q (bytes_of_list key) 1 0 tid R p pad emp
            with "ReaderIn CapIn [Pad R LoopCredits]"); [done|by iFrame|].
        iNext. iIntros "[Pad [R [_ [_ %Facts]]]]".
        rewrite byte_list_of_list final_bytes_exact. by iFrame.
      - iFrame "Reader Cap R LoopCredits".
      - iFrame "G Close Outside ObserveCredits". }
    iIntros (v) "[[G [Close [Outside ObserveCredits]]] [R Pad]]".
    iMod ("Close" with "R G") as "Lkey".
    rewrite /ordinary_observer. iModIntro. iIntros (NA AC σ) "Heap".
    iPoseProof (heap_ctx_for_observation type_lrustGS NA AC σ with "Heap") as "Heap".
    iAssert (£((0+1)*length key)) with "[ObserveCredits]" as "ObserveCredits".
    { replace ((0+1)*length key)%nat with (length key) by lia. done. }
    iMod (whole_key_heap_observe key κkey q 1 0 tid (σ.1, true) emp
      with "LFT E Lkey Cap [] Heap ObserveCredits")
      as "[Heap [Lkey [_ [_ %KeyCells]]]]"; first done.
    rewrite final_bytes_exact.
    iDestruct (interval_observe with "Heap Pad") as "%PadCells".
    iDestruct (fragments_observe with "Heap Outside") as "%OutsideCells".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done.
    iPureIntro. rewrite /ordinary_post.
    repeat split; auto using xor_result_length, xor_result_prefix, xor_result_suffix, xor_result_bytes.
  Qed.
End composition.

Section public.
  Context {Σ : gFunctors} `{!typePreG Σ}.

  Theorem ordinary_writer_observed h p pad q key :
    Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    ordinary_writer_claim h p pad q key.
  Proof using All.
    intros PadRange KeyRange PadCells KeyCells Disjoint.
    rewrite /ordinary_writer_claim /ordinary_writer.
    apply (initialized_observation_adequacy (Σ:=Σ) _ _ _ (ordinary_budget pad key)).
    intros Outer. iIntros "Fragments Credits Time".
    iExists (ordinary_observer Outer h p pad q key).
    iSplitL "Fragments Credits Time".
    - iEval (rewrite ordinary_budget_split) in "Credits".
      iApply (raw_pair_reader_na_context_wp Outer (ordinary_mutation p q pad key) h p pad q key
        (ordinary_observer Outer h p pad q key)
        (1 + shared_xor_na_cost (Nat.min (length pad) (length key)) 0 + length key)%nat
        with "Fragments Credits Time"); try done; first apply ordinary_mutation_closed.
      intros TG tid κpad κkey R G SameOuter PadNonempty KeyNonempty. subst Outer.
      iIntros "LFT PROPH UNIQ TIME E Lpad I Ref _ R G Cap _ _ Reader Close Outside Credits".
      iApply (ordinary_session with "LFT PROPH UNIQ TIME E Lpad I Ref R G Cap Reader Close Outside Credits"); done.
    - iIntros (σ v) "Heap Observe". iEval (rewrite /ordinary_observer) in "Observe".
      iApply ("Observe" $! _ _ σ with "Heap").
  Qed.
End public.
