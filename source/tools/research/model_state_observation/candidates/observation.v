From NativeHeapInitialization Require Export partition.
From lrust.lang Require Import proofmode notation lifting.
From lrust.util Require Import non_atomic_cell_map adequacy_util atomic_lock_counter.
Require Import guarding.internal.na_invariants_fork.
Set Default Proof Using "Type".

(* Actual physical heap, with only its lock bookkeeping erased. *)
Definition heap_values (σ : state) : gmap loc val := snd <$> σ.1.
Definition interval_observed (σ : state) (p : loc) (vs : list val) : Prop :=
  ∀ (i : nat) v, vs !! i = Some v → heap_values σ !! (p +ₗ i) = Some v.
Definition frame_observed (σ : state) (f : gmap loc val) : Prop :=
  ∀ l v, f !! l = Some v → heap_values σ !! l = Some v.

Section agreement.
  Context `{!lrustGS Σ}.
  Lemma fragments_observe σ f :
    heap_ctx σ -∗ initial_fragments f -∗ ⌜frame_observed σ f⌝.
  Proof.
    iDestruct 1 as (hF) "(Auth & _)". iIntros "F".
    iIntros (l v Hl). rewrite /initial_fragments.
    iDestruct (big_sepM_lookup with "F") as "P"; first done.
    rewrite /heap_mapsto.
    iDestruct (points_to_heap_agree with "P Auth") as (st) "%Hs".
    iPureIntro. by rewrite /heap_values lookup_fmap Hs.
  Qed.

  Lemma interval_observe σ p vs :
    heap_ctx σ -∗ p ↦∗ vs -∗ ⌜interval_observed σ p vs⌝.
  Proof.
    rewrite -interval_fragments. iIntros "H P".
    iDestruct (fragments_observe with "H P") as "%Obs".
    iPureIntro. intros i v Hv. apply Obs. by apply interval_lookup.
  Qed.
End agreement.

Section strong_observation.
  Context {Σ : gFunctors} `{!lrustGpreS Σ}.
  (* Φ is genuinely resource-valued. Its observer uses the actual state authority
     of the SAME instance; callers need not predict a final state. *)
  Theorem initialized_observation_adequacy e h (φ : val → state → Prop) k :
    (∀ `{!lrustGS Σ}, initial_fragments h -∗ £k -∗ time_ctx -∗
      ∃ Φ : val → iProp Σ, WP e {{ Φ }} ∗
        (∀ σ v, heap_ctx σ -∗ Φ v ={⊤,∅}=∗ ⌜φ v σ⌝)) →
    adequate NotStuck e (initialized_state h) φ.
  Proof using All.
    intros Hwp. apply adequate_alt. intros t2 σ2 [n [κs ?]]%erased_steps_nsteps.
    eapply (adequacy_util.wp_strong_adequacy Σ _); [|done]=>?.
    iMod (finite_map_initialize h) as (vγ) "[Hvγ Fragments]".
    iMod (own_alloc (● (∅ : heap_freeableUR))) as (fγ) "Hfγ";
      first by apply auth_auth_valid.
    iIntros "H£". iDestruct "H£" as "[H£1 H£2]".
    iMod (time_init with "H£1") as (Htime) "[TIME Htime]"; [done|].
    iMod na_alloc as (atomic_poolname) "pool".
    iMod atomic_lock_ctr_alloc as (atomic_lock_ctr_name) "ctr".
    set (Hheap := HeapGS _ _ _ _ vγ fγ atomic_poolname atomic_lock_ctr_name).
    iDestruct (Hwp (LRustGS _ _ _ _ Hheap Htime) with "Fragments H£2 TIME")
      as (Φ) "[WP Observe]".
    iModIntro. iExists _, [Φ], _, _. simpl. iFrame "WP".
    iSplitL "Htime Hvγ Hfγ pool ctr".
    { iFrame "Htime". rewrite /heap_ctx. iExists ∅. iFrame.
      iPureIntro. apply initialized_freeability_empty. }
    iIntros ([|e' [|]] rest -> Len Safe) "//".
    iIntros "[Heap _] [Post _] _".
    destruct (to_val e') as [v|] eqn:Ev; simpl.
    - iMod ("Observe" with "Heap Post") as "%Result".
      iModIntro. iPureIntro. split; last done.
      intros w tail [= Eq _]. subst e'. rewrite to_of_val in Ev. naive_solver.
    - iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done.
      iPureIntro. split; last done.
      intros w tail [= Eq _]. subst e'. by rewrite to_of_val in Ev.
  Qed.

  Theorem initialized_interval_adequacy e h p n (P : val → list val → Prop) k :
    (∀ `{!lrustGS Σ}, initial_fragments h -∗ £k -∗ time_ctx -∗
      WP e {{ v, ∃ vs : list val, ⌜length vs = n ∧ P v vs⌝ ∗ p ↦∗ vs }}) →
    adequate NotStuck e (initialized_state h)
      (λ v σ, ∃ vs, length vs = n ∧ P v vs ∧ interval_observed σ p vs).
  Proof using All.
    intros Callback. apply (initialized_observation_adequacy _ _ _ k).
    intros ?. iIntros "F Credits Time".
    iExists (λ v, ∃ vs : list val, ⌜length vs = n ∧ P v vs⌝ ∗ p ↦∗ vs)%I.
    iSplitL "F Credits Time"; first by iApply (Callback with "F Credits Time").
    iIntros (σ v) "Heap Post". iDestruct "Post" as (vs) "[%Sem Points]".
    iDestruct (interval_observe with "Heap Points") as "%Obs".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done. iPureIntro. by exists vs.
  Qed.

  Theorem initialized_interval_frame_adequacy e h p n f (P : val → list val → Prop) k :
    (∀ `{!lrustGS Σ}, initial_fragments h -∗ £k -∗ time_ctx -∗
      WP e {{ v, ∃ vs : list val, ⌜length vs = n ∧ P v vs⌝ ∗
        p ↦∗ vs ∗ initial_fragments f }}) →
    adequate NotStuck e (initialized_state h)
      (λ v σ, ∃ vs, length vs = n ∧ P v vs ∧
        interval_observed σ p vs ∧ frame_observed σ f).
  Proof using All.
    intros Callback. apply (initialized_observation_adequacy _ _ _ k).
    intros ?. iIntros "F Credits Time".
    iExists (λ v, ∃ vs : list val, ⌜length vs = n ∧ P v vs⌝ ∗
      p ↦∗ vs ∗ initial_fragments f)%I.
    iSplitL "F Credits Time"; first by iApply (Callback with "F Credits Time").
    iIntros (σ v) "Heap Post". iDestruct "Post" as (vs) "[%Sem [Points Frame]]".
    iDestruct (interval_observe with "Heap Points") as "%Obs".
    iDestruct (fragments_observe with "Heap Frame") as "%Outside".
    iApply fupd_mask_weaken; [|iIntros "_ !>"]; first done. iPureIntro. by exists vs.
  Qed.

  Theorem initialized_interior_observation e h p input n (P : val → list val → Prop) k :
    (∀ (i : nat) v, input !! i = Some v → h !! (p +ₗ i) = Some v) →
    (∀ `{!lrustGS Σ}, p ↦∗ input -∗ initial_fragments (h ∖ interval_map p input) -∗
      £k -∗ time_ctx -∗
      WP e {{ v, ∃ vs : list val, ⌜length vs = n ∧ P v vs⌝ ∗
        p ↦∗ vs ∗ initial_fragments (h ∖ interval_map p input) }}) →
    adequate NotStuck e (initialized_state h)
      (λ v σ, ∃ vs, length vs = n ∧ P v vs ∧ interval_observed σ p vs ∧
        frame_observed σ (h ∖ interval_map p input)).
  Proof using All.
    intros Cells Callback. apply (initialized_interval_frame_adequacy _ _ _ _ _ _ k).
    intros ?. iIntros "F Credits Time".
    iDestruct (interior_extract _ _ _ Cells with "F") as "[Points Frame]".
    iApply (Callback with "Points Frame Credits Time").
  Qed.
End strong_observation.
