From ModelInputObservation Require Export setup.
From lrust.typing Require Import borrow type_context programs inv_context soundness.
From lrust.lifetime Require Import lifetime_full.
From lrust.lang Require Import proofmode notation lifting.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition byte_values (zs : list Z) : list val :=
  (λ z : Z, LitV (LitInt z)) <$> zs.
Definition byte_map (p : loc) (zs : list Z) : gmap loc val := interval_map p (byte_values zs).
Definition pair_outside h p pad q key := h ∖ (byte_map p pad ∪ byte_map q key).
Definition pair_setup (e : expr) : expr := (Skip;; Skip;; Share;; e)%E.

Section pair_resources.
  Context `{!typeG Σ}.

  Definition shared_key_view (key : list Z) (q : loc) tid κ : iProp Σ :=
    tctx_interp tid +[#q ◁ &shr{κ} (bytes_ty (length key))]
      -[((q, repeat [] (length key)), bytes_of_list key)].

  Lemma shared_key_view_own key q tid κ :
    shared_key_view key q tid κ ⊣⊢
    ∃ d, ⧖d ∗ ty_own (&shr{κ} (bytes_ty (length key)))
      ((q, repeat [] (length key)), bytes_of_list key) d d tid [FVal #q].
  Proof. unfold shared_key_view, tctx_interp. by rewrite /= right_id (tctx_hasty_val' tid #q #q); last done. Qed.

  Lemma pair_lifetimes_recombine κpad κkey :
    llctx_interp [κpad ⊑ₗ []] ∗ llctx_interp [κkey ⊑ₗ []] ⊣⊢
    llctx_interp [κpad ⊑ₗ []; κkey ⊑ₗ []].
  Proof. by rewrite /llctx_interp /= !right_id. Qed.

  Lemma raw_pair_payload h p pad q key :
    Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    initial_fragments h ⊣⊢
      (p, repeat [] (length pad)) #↦!∗ (FVal <$> byte_values pad) ∗
      (q, repeat [] (length key)) #↦!∗ (FVal <$> byte_values key) ∗
      initial_fragments (pair_outside h p pad q key).
  Proof.
    intros PadRange KeyRange PadCells KeyCells Disjoint.
    rewrite (byte_interior_payload _ _ _ PadRange PadCells) /byte_values.
    rewrite (bi.pure_True _ PadRange) left_id.
    have KeyRest : ∀ (i : nat) (z : Z), key !! i = Some z →
      (h ∖ byte_map p pad) !! (q +ₗ i) = Some #z.
    { intros i z Hi. apply lookup_difference_Some. split; first by apply KeyCells.
      eapply map_disjoint_Some_r; first exact Disjoint. rewrite /byte_map.
      apply interval_lookup. by apply list_lookup_fmap_Some; exists z. }
    rewrite (byte_interior_payload _ _ _ KeyRange KeyRest).
    rewrite (bi.pure_True _ KeyRange) left_id /pair_outside.
    rewrite /byte_map /byte_values.
    have Diff : ∀ (h a b : gmap loc val), (h ∖ a) ∖ b = h ∖ (a ∪ b).
    { intros h0 a b. apply map_eq. intros l.
      apply option_eq. intros v. rewrite !lookup_difference_Some lookup_union_None. naive_solver. }
    by rewrite Diff.
  Qed.

  (* A real Share instruction consumes the internally constructed unique key.
     Its returned tctx is forced to the exact constant view via prophecy validity. *)
  Lemma model_key_share key q tid κ :
    llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
    llctx_interp [κ ⊑ₗ []] -∗
    invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
    ⧖1 -∗ model_reference key q tid κ -∗
    WP Share {{ _, llctx_interp [κ ⊑ₗ []] ∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) ∗
      shared_key_view key q tid κ }}.
  Proof.
    iIntros "#LFT #PROPH #UNIQ #TIME #E L I #T Ref".
    iDestruct "Ref" as (ξi ζi idx) "[Own _]".
    set (m := ((q, repeat [] (length key)), bytes_of_list key, ξi, 0%nat, 0%nat, idx)).
    iAssert (tctx_interp tid +[#q ◁ &uniq{κ} (bytes_ty (length key))] -[m])
      with "[Own]" as "Unique".
    { unfold tctx_interp. rewrite /= right_id (tctx_hasty_val' tid #q #q); last done.
      iExists 1%nat. iFrame "T". rewrite /ty_own.
      iDestruct "Own" as "[G %Phys]". iSplit; last done.
      iDestruct (ty_gho_depth_mono _ 1 0 1 1 with "G") as "[$ _]"; lia. }
    iApply wp_fupd. iApply (wp_wand with "[L I Unique]").
    { iApply (type_share_instr #q κ (bytes_ty (length key)) [] [κ ⊑ₗ []]
      (InvCtx [] static AtomicClosed) (fresh_local_alive κ) tid
      (λ xl _ _, xl = -[((q, repeat [] (length key)), bytes_of_list key)])
      with "LFT TIME PROPH UNIQ E L I Unique []").
      iApply proph_obs_true. done. }
    iIntros (v) "Post". iDestruct "Post" as (xl) "[L [I [Key Obs]]]".
    iMod (proph_obs_sat with "PROPH Obs") as %[π Eq]; first solve_ndisj.
    subst xl. iModIntro. iFrame.
  Qed.
End pair_resources.

Section pair_setup_wp.
  Context {Σ : gFunctors} `{!typePreG Σ} (Outer : lrustGS Σ).
  Existing Instance Outer.

  Theorem raw_pair_model_context_wp e h p pad q key (Φ : val → iProp Σ) k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    (∀ (TG : typeG Σ) tid κpad κkey,
      @type_lrustGS Σ TG = Outer → κpad ≠ llft_empty → κkey ≠ llft_empty →
      llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
      llctx_interp [κpad ⊑ₗ []] -∗ llctx_interp [κkey ⊑ₗ []] -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      model_reference pad p tid κpad -∗ shared_key_view key q tid κkey -∗
      initial_fragments (pair_outside h p pad q key) -∗ £k -∗ WP e @ ⊤ {{ Φ }}) →
    initial_fragments h -∗ £(1+k) -∗ time_ctx -∗
    WP pair_setup e @ ⊤ {{ Φ }}.
  Proof using All.
    intros ClosedE PadRange KeyRange PadCells KeyCells Disjoint Callback.
    iIntros "Fragments [LifeCredit Credits] #TIME".
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
    wp_bind Share. iApply (wp_wand with "[Lkey Invctx KeyRef]").
    { iApply (@model_key_share Σ TG key q tid κkey with "LFT PROPH UNIQ TIME [] Lkey Invctx T1 KeyRef").
      by rewrite /elctx_interp. }
    iIntros (v) "[Lkey [Invctx Key]]". wp_pure (Let _ _ _).
    iMod (@bytes_reference_construct Σ TG pad p tid κpad ⊤ with "LFT PROPH UNIQ T1 PadTicket PadPayload")
      as "PadRef"; [done|solve_ndisj|].
    iApply (Callback TG tid κpad κkey eq_refl PadNonempty KeyNonempty
      with "LFT PROPH UNIQ TIME [] Lpad Lkey Invctx PadRef Key Frame Credits").
    by rewrite /elctx_interp.
  Qed.
End pair_setup_wp.

Section public_setup.
  Context {Σ : gFunctors} `{!typePreG Σ}.

  (* Φ is selected in each initialized Outer and remains in that same Outer.
     The observer is a genuine resource elimination, not a final-state premise. *)
  Theorem raw_pair_model_observed e h p pad q key
      (Φ : lrustGS Σ → val → iProp Σ) (φ : val → state → Prop) k :
    Closed [] e →
    Forall (λ z, 0 ≤ z < 256) pad → Forall (λ z, 0 ≤ z < 256) key →
    (∀ (i : nat) (z : Z), pad !! i = Some z → h !! (p +ₗ i) = Some #z) →
    (∀ (i : nat) (z : Z), key !! i = Some z → h !! (q +ₗ i) = Some #z) →
    byte_map p pad ##ₘ byte_map q key →
    (∀ (Outer : lrustGS Σ) (TG : typeG Σ) tid κpad κkey,
      @type_lrustGS Σ TG = Outer → κpad ≠ llft_empty → κkey ≠ llft_empty →
      llft_ctx -∗ proph_ctx -∗ uniq_ctx -∗ time_ctx -∗ elctx_interp [] -∗
      llctx_interp [κpad ⊑ₗ []] -∗ llctx_interp [κkey ⊑ₗ []] -∗
      invctx_interp tid ⊤ [] (InvCtx [] static AtomicClosed) -∗
      model_reference pad p tid κpad -∗ shared_key_view key q tid κkey -∗
      initial_fragments (pair_outside h p pad q key) -∗ £k -∗
      @wp (iProp Σ) expr val stuckness
        (@wp' HasLc lrust_lang Σ (@lrustGS_irisGS Σ Outer)) NotStuck ⊤ e (Φ Outer)) →
    (∀ (Outer : lrustGS Σ) σ v,
      heap_ctx σ -∗ Φ Outer v ={⊤,∅}=∗ ⌜φ v σ⌝) →
    adequate NotStuck (pair_setup e) (initialized_state h) φ.
  Proof using All.
    intros ClosedE PadRange KeyRange PadCells KeyCells Disjoint Callback Observe.
    apply (initialized_observation_adequacy (Σ:=Σ) _ _ _ (1+k)).
    intros Outer. iIntros "Fragments Credits Time".
    iExists (Φ Outer). iSplitL "Fragments Credits Time".
    - iApply (raw_pair_model_context_wp Outer e h p pad q key (Φ Outer) k with "Fragments Credits Time"); try done.
      intros TG tid κpad κkey SameOuter PadNonempty KeyNonempty. subst Outer.
      iApply (Callback _ TG tid κpad κkey eq_refl PadNonempty KeyNonempty).
    - iIntros (σ v) "Heap Post". iApply (Observe Outer σ v with "Heap Post").
  Qed.
End public_setup.
