From ModelPairReader Require Export setup whole_key.
From ModelOrdinarySharedXor Require Export xor_na.
From lrust.typing Require Import borrow type_context programs inv_context soundness.
From lrust.lifetime Require Import lifetime_full.
From lrust.lang Require Import proofmode notation lifting.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section adapter.
  Context `{!typeG Σ}.
  (* The old linear halves are consumed. A persistent SC reader alone cannot
     read: it still requires its old R, which this rule does not return. *)
  Lemma sc_session_to_na n κ q x d g tid E L (Rsc Gsc Frame : iProp Σ) :
    lctx_lft_alive E L κ →
    llft_ctx -∗ elctx_interp E -∗
    shared_key_cap n κ q x d g tid -∗
    Rsc -∗ Gsc -∗ (Rsc -∗ Gsc ={⊤}=∗ llctx_interp L) -∗ Frame
    ={⊤}=∗ ∃ (R G : iProp Σ), ⌜Timeless R ∧ Timeless G⌝ ∗
      R ∗ G ∗ shared_key_cap n κ q x d g tid ∗ Frame ∗
      (G &&{↑NllftG}&&> llctx_interp L) ∗
      key_reader_na n κ q x d g tid R ∗
      (R -∗ G ={⊤}=∗ llctx_interp L).
  Proof.
    iIntros (Alive) "#LFT #E #Cap Rsc Gsc Close Frame".
    iMod ("Close" with "Rsc Gsc") as "L".
    iMod (shared_key_reader_na_session_prepared with "LFT E L Cap")
      as (R G) "[Facts [R [G [_ [Guard [Reader Close]]]]]]"; first done.
    iModIntro. iExists R, G. iFrame. iFrame "Cap".
  Qed.
End adapter.

Section pair_setup_wp.
  Context {Σ : gFunctors} `{!typePreG Σ} (Outer : lrustGS Σ).
  Existing Instance Outer.

  Theorem raw_pair_reader_na_context_wp e h p pad q key (Φ : val → iProp Σ) k :
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
      key_reader_na (length key) κkey q (bytes_of_list key) 1 0 tid R -∗
      (R -∗ G ={⊤}=∗ llctx_interp [κkey ⊑ₗ []]) -∗
      initial_fragments (pair_outside h p pad q key) -∗ £k -∗ WP e @ ⊤ {{ Φ }}) →
    initial_fragments h -∗ £(2+k) -∗ time_ctx -∗
    WP pair_setup e @ ⊤ {{ Φ }}.
  Proof using All.
    intros ClosedE PadRange KeyRange PadCells KeyCells Disjoint Callback.
    iIntros "Fragments Credits Time".
    iApply (raw_pair_reader_context_wp Outer e h p pad q key Φ k
      with "Fragments Credits Time"); try done.
    intros TG tid κpad κkey Rsc Gsc SameOuter PadNonempty KeyNonempty.
    subst Outer.
    iIntros "#LFT #PROPH #UNIQ #TIME #E Lpad Invctx PadRef %Timeless Rsc Gsc #Cap #PairKey #Guard #Reader Close Frame Credits".
    iMod (sc_session_to_na _ _ _ _ _ _ _ [] [κkey ⊑ₗ []] Rsc Gsc
      (llctx_interp [κpad ⊑ₗ []] ∗
       invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) ∗
       model_reference pad p tid κpad ∗
       initial_fragments (pair_outside h p pad q key) ∗ £k)
      with "LFT E Cap Rsc Gsc Close [Lpad Invctx PadRef Frame Credits]")
      as (R G) "[%Fresh [R [G [_ [[Lpad [Invctx [PadRef [Frame Credits]]]] [#NewGuard [#NewReader NewClose]]]]]]]".
    { apply fresh_local_alive. }
    { by iFrame. }
    iApply (Callback TG tid κpad κkey R G eq_refl PadNonempty KeyNonempty
      with "LFT PROPH UNIQ TIME E Lpad Invctx PadRef [] R G Cap PairKey NewGuard NewReader NewClose Frame Credits").
    done.
  Qed.
End pair_setup_wp.

Section public_setup.
  Context {Σ : gFunctors} `{!typePreG Σ}.

  (* Φ is selected in each initialized Outer and remains in that same Outer.
     The observer is a genuine resource elimination, not a final-state premise. *)
  Theorem raw_pair_reader_na_observed e h p pad q key
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
      key_reader_na (length key) κkey q (bytes_of_list key) 1 0 tid R -∗
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
    - iApply (raw_pair_reader_na_context_wp Outer e h p pad q key (Φ Outer) k with "Fragments Credits Time"); try done.
      intros TG tid κpad κkey R G SameOuter PadNonempty KeyNonempty. subst Outer.
      iApply (Callback _ TG tid κpad κkey R G eq_refl PadNonempty KeyNonempty).
    - iIntros (σ v) "Heap Post". iApply (Observe Outer σ v with "Heap Post").
  Qed.
End public_setup.
