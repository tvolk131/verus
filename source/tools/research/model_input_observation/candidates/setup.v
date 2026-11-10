From ModelInputContext Require Export context.
From ModelStateObservation Require Export observation.
From lrust.typing Require Import soundness inv_context.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".

Section resource_setup.
  Context {Σ : gFunctors} `{!typePreG Σ} (Outer : lrustGS Σ).
  Existing Instance Outer.

  Theorem raw_model_context_wp e h p (zs : list Z) (Φ : val → iProp Σ) k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) zs →
    (∀ (i : nat) (z : Z), zs !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (TG : typeG Σ) tid κ, @type_lrustGS Σ TG = Outer → κ ≠ llft_empty →
      @llft_ctx Σ (@type_lftGS Σ TG) -∗
      @proph_ctx Σ (@type_prophG Σ TG) -∗
      @uniq_ctx Σ TG -∗ time_ctx -∗
      @elctx_interp Σ TG [] -∗ @llctx_interp Σ TG [κ ⊑ₗ []] -∗
      @invctx_interp Σ TG tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      @model_reference Σ TG zs p tid κ -∗
      initial_fragments (h ∖ interval_map p ((λ z : Z, LitV (LitInt z)) <$> zs)) -∗
      £k -∗ WP e @ ⊤ {{ Φ }}) →
    initial_fragments h -∗ £(1+k) -∗ time_ctx -∗
    WP (Skip;; e) @ ⊤ {{ Φ }}.
  Proof using All.
    intros ClosedE Range Cells Callback.
    iIntros "Fragments [LifeCredit Credits] #TIME".
    iMod (llft_alloc with "LifeCredit") as (?) "#LFT".
    iMod proph_init as (?) "#PROPH"; first done.
    iMod uniq_init as (?) "#UNIQ"; first done.
    set (TG := TypeG Σ Outer _ _ _ _ _ _).
    iMod (invctx_alloc ⊤) as (tid) "Invctx".
    iMod (llftl_begin' with "LFT") as (κ) "[Alive %Nonempty]"; first done.
    iDestruct (byte_interior_payload _ _ _ Range Cells with "Fragments")
      as "[_ [Payload Frame]]".
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt2 with "TIME"); first solve_ndisj.
    iIntros "Tickets". iDestruct (cumulative_time_receipt_sep 1 1 with "Tickets") as "[Ticket Receipt]".
    iMod persistent_time_receipt_0 as "T0".
    iMod (cumulative_persistent_time_receipt _ 1 0 with "TIME Receipt T0") as "#T1";
      first solve_ndisj.
    wp_pure Skip. wp_pure (Let _ _ _).
    iMod (@bytes_reference_construct Σ TG zs p tid κ ⊤ with "LFT PROPH UNIQ T1 Ticket Payload")
      as "Reference"; [done|solve_ndisj|].
    iApply (Callback TG tid κ eq_refl Nonempty with "LFT PROPH UNIQ TIME [] [Alive] Invctx Reference Frame Credits").
    - by rewrite /elctx_interp.
    - rewrite /llctx_interp /llctx_elt_interp /= right_id.
      iExists κ. iFrame "Alive". by rewrite left_id.
  Qed.
End resource_setup.
