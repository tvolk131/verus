Require Import guarding.internal.na_invariants_fork.
From lrust.util Require Import atomic_lock_counter adequacy_util.
From lrust.lang Require Export adequacy.
From lrust.lang Require Import proofmode notation lifting.
From lrust.util Require Import non_atomic_cell_map.
Set Default Proof Using "Type".

Definition initialized_map (h : gmap loc val) : gmap loc (lock_state * val) :=
  (λ v, (RSt 0, v)) <$> h.
Definition initialized_state (h : gmap loc val) : state := (initialized_map h, false).
Definition initial_fragments `{!heapGS Σ} (h : gmap loc val) : iProp Σ :=
  [∗ map] l ↦ v ∈ h, l ↦ v.

Section allocation.
  Context `{!invGS Σ, !na_logicG loc val Σ}.
  Lemma finite_map_initialize (h : gmap loc val) :
    ⊢ |==> ∃ ι, non_atomic_map ι (initialized_map h) ∗
      ([∗ map] l ↦ v ∈ h, inl l @↦[ι] inr v).
  Proof.
    induction h as [|l v h Hfresh IH] using map_ind.
    - rewrite /initialized_map fmap_empty.
      iMod non_atomic_map_alloc_empty as (ι) "H". iModIntro. iExists ι. by iFrame.
    - iMod IH as (ι) "[H F]".
      iMod (non_atomic_map_insert ι l v with "H") as "[P H]".
      { by rewrite /initialized_map lookup_fmap Hfresh. }
      rewrite /initialized_map fmap_insert.
      iModIntro. iExists ι. iFrame "H".
      iApply big_sepM_insert; first done.
      iFrame.
  Qed.
End allocation.

Lemma initialized_lookup h l v :
  initialized_map h !! l = Some (RSt 0, v) ↔ h !! l = Some v.
Proof. rewrite /initialized_map lookup_fmap. destruct (h !! l); naive_solver. Qed.

Lemma initialized_freeability_empty h : heap_freeable_rel (initialized_state h) ∅.
Proof. intros blk qs. rewrite lookup_empty. done. Qed.

Section operational.
  Context {Σ : gFunctors} `{!lrustGpreS Σ}.
  Theorem initialized_heap_adequacy e h φ k :
    (∀ `{!lrustGS Σ}, initial_fragments h -∗ £k -∗ time_ctx -∗
      WP e {{ v, ⌜φ v⌝ }}) →
    adequate NotStuck e (initialized_state h) (λ v _, φ v).
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
    iModIntro. iExists _, [_], _, _. simpl.
    iDestruct (Hwp (LRustGS _ _ _ _ Hheap Htime) with "Fragments H£2 TIME") as "$".
    iSplitL.
    { iFrame "Htime". rewrite /heap_ctx. iExists ∅. iFrame.
      iPureIntro. apply initialized_freeability_empty. }
    iIntros ([|e' [|]]? -> ??) "//".
    iIntros "[??] [?_] _". iApply fupd_mask_weaken; [|iIntros "_ !>"]; [done|].
    iSplit; [|done]. iIntros (v2 t2'' [= -> <-]). by rewrite to_of_val.
  Qed.
End operational.

(* An executed instance that actually reads an existing nonempty heap cell. *)
Theorem initialized_read_adequate (h : gmap loc val) (l : loc) (v : val) :
  h !! l = Some v →
  adequate NotStuck (Read ScOrd #l) (initialized_state h) (λ w _, w = v).
Proof.
  intros Hl. apply (initialized_heap_adequacy (Σ:=lrustΣ) _ _ _ 0).
  intros ?. iIntros "F _ _".
  rewrite /initial_fragments.
  iDestruct (big_sepM_lookup with "F") as "P"; first exact Hl.
  iApply (wp_read_sc with "P"); first done.
  iNext. iIntros "_". done.
Qed.
