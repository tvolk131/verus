From ModelCallerOrdinaryWriter Require Export interface.
From lrust.typing Require Import soundness inv_context programs_util borrow uniq_util.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section caller.
  Context `{!typeG Σ}.

  Theorem caller_xor_na_verified n k E L I tid iκs κ κkey p q
      (x : ~~(bytes_sort n)) (key : ~~(bytes_sort k)) m d g Frame :
    lctx_lft_alive E L κ →
    lctx_lft_alive E L (ty_lft (&uniq{κ} (bytes_ty n))%T) →
    lctx_lft_alive E L κkey →
    uniq_bor_loc m = (p,repeat [] n) → uniq_bor_current m = x →
    ⊢ caller_xor_na_contract n k E L I tid iκs κ κkey p q x key m d g Frame.
  Proof.
    intros Alive ParentAlive KeyAlive Loc Cur.
    rewrite /caller_xor_na_contract /caller_xor_na_procedure.
    iIntros "#LFT #TIME #PROPH #UNIQ #E L I Parent #View Credits Frame".
    iApply (wp_wand with "[L I Parent Credits Frame]").
    { iApply (shorter_run_resources E L I tid iκs #p (bytes_ty n) κ m
        (xor_bytes n x (byte_list k key)) (caller_xor_na_body n k p q)
        (shared_key_view k κkey q key d g tid ∗ £(caller_xor_na_credit n k g))%I
        (⌜Forall byte_value (xor_result (byte_list n x) (byte_list k key))⌝)%I Frame
        with "LFT TIME PROPH UNIQ E L I Parent [Credits] Frame");
        [done|done|rewrite /caller_xor_na_body; solve_closed| |by iFrame "View Credits"].
      intros α c CLoc CCur. rewrite Loc in CLoc. rewrite Cur in CCur.
      iIntros "#LFT' #TIME' #PROPH' #UNIQ' #E' L I Child [#V Credits]".
      rewrite /caller_xor_na_credit.
      iDestruct "Credits" as "[Prepare [Open LoopCredits]]".
      iMod (shared_key_reader_na_session k κkey q key d g tid E (α ⊑ₗ [κ] :: L)
        with "LFT' E' L V Prepare")
        as (R G) "[%Timeless [R [G [#Cap [#GuardL [#Reader Join]]]]]]".
      { by apply ordinary_alive_under_child. }
      destruct Timeless as [TR TG].
      iPoseProof "Cap" as "[_ %RK]".
      iApply wp_fupd.
      iApply (wp_wand with "[G Child Open R LoopCredits]").
      { iApply (byte_child_transform_guarded n E (α ⊑ₗ [κ] :: L) α p x
          (xor_bytes n x (byte_list k key)) c tid (caller_xor_na_body n k p q) G
          (key_reader_na k κkey q key d g tid R ∗ shared_key_cap k κkey q key d g tid ∗
           R ∗ £(shared_xor_na_cost (Nat.min n k) g))%I
          (R ∗ ⌜Forall byte_value (xor_result (byte_list n x) (byte_list k key))⌝)%I
          with "LFT' TIME' UNIQ' E' GuardL G Child Open [R LoopCredits]");
          [by apply child_alive|done|done| | |by iFrame "Reader Cap R LoopCredits"].
        - intros RX. rewrite xor_bytes_exact. by apply xor_result_bytes.
        - intros RX. iIntros "Pad [#Read [#K [R Credits]]]".
          rewrite /caller_xor_na_body.
          iPoseProof (wp_shared_xor_na_min k κkey q key d g tid R p
              (byte_list n x) True RX with "Read K") as "Loop".
          iEval (rewrite byte_list_length) in "Loop".
          iApply ("Loop" with "[Pad R Credits]").
          { by iFrame. }
          iNext. iIntros "[Pad [R [#Cap' [_ %Facts]]]]".
          rewrite xor_bytes_exact. iFrame "Pad R". iPureIntro. tauto. }
      iIntros (v) "[[R %Range] [%c' [G [Child #Updated]]]]".
      iMod ("Join" with "R G") as "L".
      iModIntro. iSplit; first done. iExists c'. iFrame "L I Child Updated". }
    iIntros (v) "[Frame [%Range [L [I [%m' [Parent #Recovered]]]]]]".
    iFrame "Frame View E L I". iExists m'. iFrame "Parent Recovered".
    iPureIntro. split; first apply xor_bytes_exact.
    split; first by rewrite xor_result_length byte_list_length.
    have Len : Nat.min n k = Nat.min (length (byte_list n x)) (length (byte_list k key))
      by rewrite !byte_list_length.
    rewrite Len. repeat split; auto using xor_result_prefix, xor_result_suffix.
  Qed.
End caller.
