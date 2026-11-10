From ModelMutableWriter Require Export resource_protocol child.
From ModelStateObservation Require Import observation.
From NativeSemanticBridge Require Import borrow_handoff raw_view.
From lrust.typing Require Import soundness inv_context programs_util borrow uniq_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section normal_exit.
  Context `{!typeG Σ}.

  (* Generic ordinary function-end/continuation rule; it consumes only the
     pending reference, not an opaque owner and not the live caller parent. *)
  Lemma pending_reference_normal_release {𝔄} E L I p κ (ty : type 𝔄) :
    lctx_lft_alive E L κ ->
    typed_instr E L I +[p ◁ &uniq{κ} ty] Skip (λ _, +[])
      (λ post '-[a], λ mask π,
        uniq_bor_future a π = vπ (uniq_bor_current a) π -> post -[] mask π).
  Proof. intros Alive. apply type_resolve_instr. by apply source_loan_release. Qed.

  (* No fresh world, fresh parent lifetime, future=current, or empty ancestor
     history. The caller continues with its original future and updated current.
     This is the existing MODEL reborrow protocol, not the MIR body. *)
  Lemma model_drop_continues {𝔄} E L I tid iκs p (ty : type 𝔄) κ m
      (Frame : iProp Σ) e Φ :
    Closed [] e ->
    lctx_lft_alive E L κ ->
    lctx_lft_alive E L (ty_lft (&uniq{κ} ty)%T) ->
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
    tctx_elt_interp tid (p ◁ &uniq{κ} ty) m -∗ Frame -∗
    (∀ m', llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
      tctx_elt_interp tid (p ◁ &uniq{κ} ty) m' -∗
      ⟨π, vπ m' π = (vπ (uniq_bor_current m) π, uniq_bor_future m π)⟩ -∗
      Frame -∗ WP e {{ Φ }}) -∗
    WP ((Newlft;; ReBor;; Endlft;; Skip);; e) {{ Φ }}.
  Proof.
    iIntros (ClosedE Alive ParentAlive) "#LFT #TIME #PROPH #UNIQ #E L I Parent Frame Continue".
    wp_bind (Newlft;; ReBor;; Endlft;; Skip)%E.
    iApply (wp_wand with "[L I Parent Frame]").
    { iApply (shorter_no_write E L I tid iκs p ty κ m Frame
        with "LFT TIME PROPH UNIQ E L I Parent Frame"); done. }
    iIntros (v) "[Frame [L [I [%m' [Parent #Recovered]]]]]".
    wp_seq. iApply ("Continue" with "L I Parent Recovered Frame").
  Qed.

  (* Actual model heap authority agrees with the SAME typed original reference's
     current payload at any live interior p. Reclose without resolving the
     original or changing its future. This is not source-current adequacy. *)
  Lemma live_original_current_observation n E L κ p (x : ~~(bytes_sort n)) m tid σ :
    lctx_lft_alive E L κ ->
    uniq_bor_loc m = (p,repeat [] n) -> uniq_bor_current m = x ->
    llft_ctx -∗ uniq_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty n)) m -∗
    heap_ctx σ -∗ £1 ={⊤}=∗
      llctx_interp L ∗ tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty n)) m ∗
      heap_ctx σ ∗ ⌜interval_observed σ p ((λ z : Z, LitV (LitInt z)) <$> byte_list n x)⌝.
  Proof.
    intros Alive Loc Cur.
    destruct m as [[[[[l y] ξi] di] gi] idx]. simpl in Loc, Cur. subst l y.
    iIntros "#LFT #UNIQ #E L Original Heap Credit".
    iDestruct "Original" as (v d) "[%Path [#Td Own]]".
    simpl in Path. inversion Path; subst v.
    iMod (source_context_access (bytes_ty n) κ E L (p,repeat [] n) x ξi di gi idx
      d d tid p Alive with "LFT UNIQ E L Own") as "[_ [Payload [Ticket Close]]]".
    iMod (lc_fupd_elim_later with "Credit Payload") as "[#Time [Ghost P]]".
    iEval (rewrite bytes_vector_mapsto) in "P".
    iDestruct (interval_observe with "Heap P") as "%Observed".
    iAssert (▷ (⧖(S di `max` gi) ∗ ty_gho (bytes_ty n) x di gi tid ∗
      (p,repeat [] n) #↦!∗ ty_phys (bytes_ty n) x tid))%I with "[P Ghost]" as "Payload".
    { iNext. iFrame "Time Ghost". by rewrite bytes_vector_mapsto. }
    iMod ("Close" with "Payload Ticket") as "[Own L]".
    iModIntro. iFrame "L Heap". iSplit; last done.
    iExists #p,d. iFrame "Td Own". done.
  Qed.
  (* Normal function-end cut: observe before consuming this pending value,
     resolve it by the existing checked rule, and pass the observation plus
     original caller/ancestor frame to a continuation. No parent is resolved.
     Source use still needs the explicit original-operand interpretation. *)
  Lemma pending_heap_normal_exit n E L I tid iκs κ p (x : ~~(bytes_sort n)) m σ
      (Frame : iProp Σ) e Φ :
    Closed [] e -> lctx_lft_alive E L κ ->
    uniq_bor_loc m = (p,repeat [] n) -> uniq_bor_current m = x ->
    llft_ctx -∗ time_ctx -∗ proph_ctx -∗ uniq_ctx -∗ elctx_interp E -∗
    llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗
    tctx_elt_interp tid (#p ◁ &uniq{κ} (bytes_ty n)) m -∗
    heap_ctx σ -∗ £1 -∗ Frame -∗
    (llctx_interp L -∗ invctx_interp tid ⊤ iκs I -∗ heap_ctx σ -∗ Frame -∗
      ⟨π, uniq_bor_future m π = vπ x π⟩ -∗
      ⌜interval_observed σ p ((λ z : Z, LitV (LitInt z)) <$> byte_list n x)⌝ -∗
      WP e {{ Φ }}) -∗
    WP (Skip;; e) {{ Φ }}.
  Proof.
    iIntros (ClosedE Alive Loc Cur)
      "#LFT #TIME #PROPH #UNIQ #E L I Pending Heap Credit Frame Continue".
    iApply fupd_wp.
    iMod (live_original_current_observation n E L κ p x m tid σ Alive Loc Cur
      with "LFT UNIQ E L Pending Heap Credit") as "[L [Pending [Heap %Observed]]]".
    iModIntro. wp_bind Skip.
    iApply (wp_wand with "[L I Pending]").
    { iApply (type_resolve_instr #p (&uniq{κ} (bytes_ty n))%T
        (λ a π, uniq_bor_future a π = vπ (uniq_bor_current a) π) E L I
        (source_loan_release _ _ _ _ Alive)
        tid (λ _ _ π, uniq_bor_future m π = vπ x π) ⊤ iκs -[m]
        with "LFT TIME PROPH UNIQ E L I [Pending] []").
      - simpl. iFrame.
      - iApply proph_obs_true. intros π Release. by rewrite Cur in Release. }
    iIntros (v) "[%xl [L [I [_ #Released]]]]".
    wp_seq. iApply ("Continue" with "L I Heap Frame Released []"). done.
  Qed.
End normal_exit.
