From ModelPairReader Require Export fixed.
From lrust.typing Require Import borrow type_context programs inv_context soundness.
From lrust.lifetime Require Import lifetime_full.
From lrust.lang Require Import proofmode notation lifting.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section pair_setup_wp.
  Context {Σ : gFunctors} `{!typePreG Σ} (Outer : lrustGS Σ).
  Existing Instance Outer.

  Theorem raw_pair_reader_context_wp e h p pad q key (Φ : val → iProp Σ) k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    (∀ (TG : typeG Σ) tid κpad κkey (R G : iProp Σ),
      @type_lrustGS Σ TG = Outer → κpad ≠ llft_empty → κkey ≠ llft_empty →
      llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
      llctx_interp [κpad ⊑ₗ []] -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      model_reference pad p tid κpad -∗
      ⌜Timeless R ∧ Timeless G⌝ -∗ R -∗ G -∗
      shared_key_cap (length key) κkey q (bytes_of_list key) 1 0 tid -∗
      ModelPairInput.setup.shared_key_view key q tid κkey -∗
      (G &&{↑NllftG}&&> llctx_interp [κkey ⊑ₗ []]) -∗
      key_reader (length key) κkey q (bytes_of_list key) 1 0 tid R -∗
      (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) -∗
      initial_fragments (pair_outside h p pad q key) -∗ £k -∗ WP e @ ⊤ {{ Φ }}) →
    initial_fragments h -∗ £(2+k) -∗ time_ctx -∗
    WP pair_setup e @ ⊤ {{ Φ }}.
  Proof using All.
    intros ClosedE PadRange KeyRange PadCells KeyCells Disjoint Callback.
    replace (2+k)%nat with (1+(1+k))%nat by lia.
    iIntros "Fragments [LifeCredit [Prepare Credits]] #TIME".
    iMod (llft_alloc with "LifeCredit") as (?) "#LFT".
    iMod proph_init as (?) "#PROPH"; first done.
    iMod uniq_init as (?) "#UNIQ"; first done.
    set (TG := TypeG Σ Outer _ _ _ _ _ _).
    iMod (invctx_alloc ⊤) as (tid) "Invctx".
    iMod (llftl_begin' with "LFT") as (κpad) "[PadAlive %PadNonempty]"; first done.
    iMod (llftl_begin' with "LFT") as (κkey) "[KeyAlive %KeyNonempty]"; first done.
    iAssert (llctx_interp [κpad ⊑ₗ []]) with "[PadAlive]" as "Lpad".
    { rewrite /llctx_interp /llctx_elt_interp /= right_id.
      iExists κpad. iFrame "PadAlive". by rewrite left_id. }
    iAssert (llctx_interp [κkey ⊑ₗ []]) with "[KeyAlive]" as "Lkey".
    { rewrite /llctx_interp /llctx_elt_interp /= right_id.
      iExists κkey. iFrame "KeyAlive". by rewrite left_id. }
    iDestruct (@raw_pair_payload Σ TG _ _ _ _ _ PadRange KeyRange PadCells KeyCells Disjoint
      with "Fragments") as "[PadPayload [KeyPayload Frame]]".
    rewrite /pair_setup.
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt2 with "TIME"); first solve_ndisj.
    iIntros "Tickets". iDestruct (cumulative_time_receipt_sep 1 1 with "Tickets") as "[PadTicket Receipt]".
    iMod persistent_time_receipt_0 as "T0".
    iMod (cumulative_persistent_time_receipt _ 1 0 with "TIME Receipt T0") as "#T1";
      first solve_ndisj.
    wp_pure Skip. wp_pure (Let _ _ _).
    wp_bind Skip.
    iApply (wp_cumulative_time_receipt2 with "TIME"); first solve_ndisj.
    iIntros "Tickets". iDestruct (cumulative_time_receipt_sep 1 1 with "Tickets") as "[Ticket Spare]".
    wp_pure Skip. wp_pure (Let _ _ _).
    iMod (@bytes_reference_construct Σ TG key q tid κkey ⊤ with "LFT PROPH UNIQ T1 Ticket KeyPayload")
      as "KeyRef"; [done|solve_ndisj|].
    wp_bind Share. iApply (wp_wand with "[Lkey KeyRef]").
    { iApply (@model_key_share_fixed Σ TG key q tid κkey KeyRange
        with "LFT PROPH UNIQ TIME [] Lkey KeyRef").
      by rewrite /elctx_interp. }
    iIntros (v) "[Lkey #Key]". wp_pure (Let _ _ _).
    iMod (@bytes_reference_construct Σ TG pad p tid κpad ⊤ with "LFT PROPH UNIQ T1 PadTicket PadPayload")
      as "PadRef"; [done|solve_ndisj|].
    iDestruct (@fixed_view_pair_context Σ TG key q tid κkey with "T1 Key") as "#PairKey".
    iMod (@fixed_view_reader_session Σ TG key q tid κkey
      with "LFT [] Lkey Key Prepare") as (R G) "[%Timeless [R [G [#Cap [#Guard [#Reader Close]]]]]]".
    { by rewrite /elctx_interp. }
    iApply (Callback TG tid κpad κkey R G eq_refl PadNonempty KeyNonempty
      with "LFT PROPH UNIQ TIME [] Lpad Invctx PadRef [] R G Cap PairKey Guard Reader Close Frame Credits").
    - by rewrite /elctx_interp.
    - done.
  Qed.
End pair_setup_wp.

Section public_setup.
  Context {Σ : gFunctors} `{!typePreG Σ}.

  (* Φ is selected in each initialized Outer and remains in that same Outer.
     The observer is a genuine resource elimination, not a final-state premise. *)
  Theorem raw_pair_reader_observed e h p pad q key
      (Φ : lrustGS Σ → val → iProp Σ) (φ : val → state → Prop) k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    (∀ (Outer : lrustGS Σ) (TG : typeG Σ) tid κpad κkey (R G : iProp Σ),
      @type_lrustGS Σ TG = Outer → κpad ≠ llft_empty → κkey ≠ llft_empty →
      llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
      llctx_interp [κpad ⊑ₗ []] -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      model_reference pad p tid κpad -∗
      ⌜Timeless R ∧ Timeless G⌝ -∗ R -∗ G -∗
      shared_key_cap (length key) κkey q (bytes_of_list key) 1 0 tid -∗
      ModelPairInput.setup.shared_key_view key q tid κkey -∗
      (G &&{↑NllftG}&&> llctx_interp [κkey ⊑ₗ []]) -∗
      key_reader (length key) κkey q (bytes_of_list key) 1 0 tid R -∗
      (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) -∗
      initial_fragments (pair_outside h p pad q key) -∗ £k -∗
      @wp (iProp Σ) expr val stuckness
        (@wp' HasLc lrust_lang Σ (@lrustGS_irisGS Σ Outer)) NotStuck ⊤ e (Φ Outer)) →
    (∀ (Outer : lrustGS Σ) σ v,
      heap_ctx σ -∗ Φ Outer v ={⊤,∅}=∗ ⌜φ v σ⌝) →
    adequate NotStuck (pair_setup e) (initialized_state h) φ.
  Proof using All.
    intros ClosedE PadRange KeyRange PadCells KeyCells Disjoint Callback Observe.
    apply (initialized_observation_adequacy (Σ:=Σ) _ _ _ (2+k)).
    intros Outer. iIntros "Fragments Credits Time".
    iExists (Φ Outer). iSplitL "Fragments Credits Time".
    - iApply (raw_pair_reader_context_wp Outer e h p pad q key (Φ Outer) k with "Fragments Credits Time"); try done.
      intros TG tid κpad κkey R G SameOuter PadNonempty KeyNonempty. subst Outer.
      iApply (Callback _ TG tid κpad κkey R G eq_refl PadNonempty KeyNonempty).
    - iIntros (σ v) "Heap Post". iApply (Observe Outer σ v with "Heap Post").
  Qed.
End public_setup.
