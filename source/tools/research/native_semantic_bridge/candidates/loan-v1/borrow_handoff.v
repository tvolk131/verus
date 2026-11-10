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
      [done|done|].
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
    intros Alive. apply (resolve_impl _ _ _
      (λ m π, snd (vπ m π) = fst (vπ m π))).
    - exact (uniq_resolve E L κ ty Alive).
    - intros m π. destruct m as [[[[[l x] ξi] di] gi] idx]. done.
  Qed.
End handoff.

Print Assumptions source_loan_access.
Print Assumptions source_loan_release.
