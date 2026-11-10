From ModelPairInput Require Export setup.
From ModelSharedByteRead Require Export session observation.
From lrust.typing Require Import borrow.
From lrust.lifetime Require Import lifetime_full.
From guarding Require Import guard tactics.
Set Default Proof Using "Type".
Open Scope Z_scope.

Section handoff.
  Context `{!typeG Σ}.

  (* This is an exact conversion, NOT a bound on the fresh existential. *)
  Lemma pair_view_raw_exact key q tid κ :
    ModelPairInput.setup.shared_key_view key q tid κ ⊣⊢
    ∃ d, ⧖d ∗ ModelSharedByteRead.shared_read.shared_key_view
      (length key) κ q (bytes_of_list key) d d tid.
  Proof. apply ModelPairInput.setup.shared_key_view_own. Qed.

  (* A fixed view has a fixed preparation budget and keeps R/G linear. *)
  Lemma fixed_view_reader_session key q tid κ :
    llft_ctx -∗ elctx_interp [] -∗ llctx_interp [κ ⊑ₗ []] -∗
    ModelSharedByteRead.shared_read.shared_key_view
      (length key) κ q (bytes_of_list key) 1 0 tid -∗ £1 ={⊤}=∗
    ∃ (R G : iProp Σ), ⌜Timeless R ∧ Timeless G⌝ ∗ R ∗ G ∗
      shared_key_cap (length key) κ q (bytes_of_list key) 1 0 tid ∗
      (G &&{↑NllftG}&&> llctx_interp [κ ⊑ₗ []]) ∗
      key_reader (length key) κ q (bytes_of_list key) 1 0 tid R ∗
      (R -∗ G ={⊤}=∗ llctx_interp [κ ⊑ₗ []]).
  Proof.
    iIntros "#LFT #E L #View Credit".
    iApply (shared_key_reader_session with "LFT E L View Credit").
    apply fresh_local_alive.
  Qed.

  Lemma empty_key_no_read (i : nat) : ¬ (i < length ([] : list Z))%nat.
  Proof. simpl. lia. Qed.
End handoff.
