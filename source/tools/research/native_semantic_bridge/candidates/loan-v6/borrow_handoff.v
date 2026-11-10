From lrust.typing Require Import uniq_bor uniq_util borrow pptr.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".

(* Model-level handoff: source typing supplies the unique reference resource.
   This is NOT a native Rust cast rule and has no publication consumer. *)
Section handoff.
  Context `{!typeG Σ}.

  Lemma source_loan_access {𝔄} (ty : type 𝔄) κ
      (l : cloc) (x : ~~𝔄) ξi di gi idx d g tid (p : loc) E G :
    Timeless G → ↑Nllft ⊆ E →
    llft_ctx -∗ uniq_ctx -∗ (G &&{↑NllftG}&&> @[κ]) -∗
    ty_own (&uniq{κ} ty) (l, x, ξi, di, gi, idx) d g tid [FVal #p] -∗
    G ={E}=∗
      ⌜p = l.1⌝ ∗
      ▷ (⧖(S di `max` gi) ∗ ty_gho ty x di gi tid ∗ l #↦!∗ ty_phys ty x tid) ∗
      ⧗1 ∗
      (▷ (⧖(S di `max` gi) ∗ ty_gho ty x di gi tid ∗ l #↦!∗ ty_phys ty x tid) -∗
        ⧗1 ={↑Nllft}=∗
        ty_own (&uniq{κ} ty) (l, x, ξi, di, gi, idx) d g tid [FVal #p] ∗ G).
  Proof.
    iIntros (??) "#LFT #UNIQ #Guard [Loan %Phys] G".
    simpl in Phys. inversion Phys; subst p.
    iDestruct "Loan" as "[#Incl [#Bounds [Body [#Base #Pers]]]]".
    iMod (uniq_body_acc with "LFT UNIQ Guard Body G") as "[Payload [Ticket Close]]";
      [done|].
    iModIntro. iSplit; first done. iFrame "Payload Ticket".
    iIntros "Payload Ticket".
    iMod ("Close" with "Payload Ticket") as "[Body G]".
    iModIntro. iFrame "G". rewrite /ty_own /=. iFrame "Body". iFrame "#". done.
  Qed.

  Lemma source_loan_release {𝔄} E L κ (ty : type 𝔄) :
    lctx_lft_alive E L κ →
    resolve E L (&uniq{κ} ty)
      (λ m π, uniq_bor_future m π = vπ (uniq_bor_current m) π).
  Proof.
    intros Alive. eapply resolve_impl.
    - exact (uniq_resolve E L κ ty Alive).
    - intros m π. destruct m as [[[[[l x] ξi] di] gi] idx]. done.
  Qed.
  (* Public scoped form: the lifetime token is obtained from the ordinary
     typing context, not passed by the client as a second permission. *)
  Lemma source_context_access {𝔄} (ty : type 𝔄) κ E L
      (l : cloc) (x : ~~𝔄) ξi di gi idx d g tid (p : loc) :
    lctx_lft_alive E L κ →
    llft_ctx -∗ uniq_ctx -∗ elctx_interp E -∗ llctx_interp L -∗
    ty_own (&uniq{κ} ty) (l, x, ξi, di, gi, idx) d g tid [FVal #p]
    ={⊤}=∗
      ⌜p = l.1⌝ ∗
      ▷ (⧖(S di `max` gi) ∗ ty_gho ty x di gi tid ∗ l #↦!∗ ty_phys ty x tid) ∗
      ⧗1 ∗
      (▷ (⧖(S di `max` gi) ∗ ty_gho ty x di gi tid ∗ l #↦!∗ ty_phys ty x tid) -∗
        ⧗1 ={⊤}=∗
        ty_own (&uniq{κ} ty) (l, x, ξi, di, gi, idx) d g tid [FVal #p] ∗ llctx_interp L).
  Proof.
    iIntros (Alive) "#LFT #UNIQ E L Source".
    iMod (llctx_interp_make_guarded with "L") as (γ) "[H1 [H2 [#GuardL #Back]]]";
      first solve_ndisj.
    iMod (lctx_lft_alive_get_guards _ _ _ _ _ Alive with "GuardL E H1")
      as "[H1 #GuardK]"; first solve_ndisj.
    iDestruct (guards_transitive with "GuardL GuardK") as "#Guard".
    iMod (source_loan_access with "LFT UNIQ Guard Source H1")
      as "[Phys [Payload [Ticket Close]]]"; [solve_ndisj|].
    iModIntro. iFrame "Phys Payload Ticket". iIntros "Payload Ticket".
    iPoseProof ("Close" with "Payload Ticket") as "Release".
    iMod (fupd_mask_mono with "Release") as "[Source H1]"; first solve_ndisj.
    iPoseProof ("Back" with "H1 H2") as "Back2".
    iMod (fupd_mask_mono with "Back2") as "L"; first solve_ndisj.
    iModIntro. iFrame.
  Qed.
End handoff.

Print Assumptions source_loan_access.
Print Assumptions source_loan_release.

Print Assumptions source_context_access.
