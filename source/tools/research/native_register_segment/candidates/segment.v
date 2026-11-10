From NativeMirLrust Require Export simulation.
From lrust.lang Require Import proofmode.
From stdpp Require Import gmap.
Set Default Proof Using "Type".
Open Scope Z_scope.

Definition registers := gmap nat descriptor.
Inductive instruction :=
| AssignPure (destination : nat) (op : operation) (sources : list nat).
Definition register_arguments (r : registers) (names : list nat) (ds : list descriptor) : Prop :=
  Forall2 (λ name d, r !! name = Some d) names ds.
Inductive register_step : instruction → registers → state → registers → state → Prop :=
| RegisterCompute dst op src ds out r σ τ :
    register_arguments r src ds → source_step op ds σ [out] τ →
    register_step (AssignPure dst op src) r σ (<[dst := out]>r) τ.
Inductive segment_run : list instruction → registers → state → registers → state → Prop :=
| SegmentNil r σ : segment_run [] r σ r σ
| SegmentCons i code r σ mid τ out υ :
    register_step i r σ mid τ → segment_run code mid τ out υ →
    segment_run (i :: code) r σ out υ.

Lemma register_step_heap i r σ out τ : register_step i r σ out τ → τ = σ.
Proof.
  intros Step. inversion Step; subst.
  match goal with H : source_step _ _ _ _ _ |- _ => inversion H; subst; done end.
Qed.

Lemma register_step_other dst op src r σ out τ name :
  register_step (AssignPure dst op src) r σ out τ → name ≠ dst →
  out !! name = r !! name.
Proof. intros Step Other. inversion Step; subst. apply lookup_insert_ne. congruence. Qed.

Lemma segment_run_heap code r σ out τ : segment_run code r σ out τ → τ = σ.
Proof.
  intros Run. induction Run; first done.
  have Eq := register_step_heap _ _ _ _ _ H. congruence.
Qed.

(* The requested adjacent assignments; storage and all other nodes are outside
   this segment, with their interpretation obligations still open. *)
Definition end_segment : list instruction :=
  [AssignPure 6 UnwrapThin [3%nat]; AssignPure 5 ByteOffset [6%nat;2%nat]].
Definition end_registers (r : registers) (p : pointer) (n : Z) : registers :=
  <[5%nat := RawThin (shifted p n)]>(<[6%nat := RawThin p]>r).

Theorem end_segment_exec r σ p n :
  r !! 3%nat = Some (NonNullThin p) → r !! 2%nat = Some (Usize n) →
  offset_valid p n → segment_run end_segment r σ (end_registers r p n) σ.
Proof.
  intros Start Length Valid. unfold end_segment,end_registers.
  eapply SegmentCons.
  - eapply RegisterCompute.
    + constructor; first exact Start. constructor.
    + apply StepUnwrapThin. exact (proj1 Valid).
  - eapply SegmentCons.
    + eapply RegisterCompute.
      * constructor. { by rewrite lookup_insert. }
        constructor. { rewrite lookup_insert_ne; [exact Length|lia]. } constructor.
      * apply StepByteOffset. exact Valid.
    + constructor.
Qed.

Lemma end_segment_outputs r p n :
  end_registers r p n !! 6%nat = Some (RawThin p) ∧
  end_registers r p n !! 5%nat = Some (RawThin (shifted p n)).
Proof. unfold end_registers. rewrite lookup_insert_ne; [by rewrite !lookup_insert|lia]. Qed.

Lemma end_segment_frame r p n name : name ≠ 5%nat → name ≠ 6%nat →
  end_registers r p n !! name = r !! name.
Proof.
  intros H5 H6. unfold end_registers.
  rewrite lookup_insert_ne; last congruence.
  apply lookup_insert_ne. congruence.
Qed.

Definition end_code : val :=
  (λ: ["start"; "len"],
    let: "r6" := (λ: ["data"], "data") ["start"] in
    "r6" +ₗ "len")%V.

Lemma chain_pure n m (a b c : expr) :
  PureExec True n a b → PureExec True m b c → PureExec True (n+m) a c.
Proof.
  rewrite /PureExec. intros First Second _. eapply nsteps_trans; [apply First;done|apply Second;done].
Qed.
Lemma end_code_four_steps p n :
  PureExec True 4
    (App (of_val end_code) (Lit (LitLoc (pointer_loc p)) :: Lit (LitInt n) :: nil))
    (Lit (LitLoc (pointer_loc (shifted p n)))).
Proof.
  rewrite /end_code pointer_loc_shift.
  eapply (chain_pure 1 3); first tc_solve.
  eapply (chain_pure 1 2).
  - eapply (pure_exec_fill [AppRCtx
      (RecV BAnon [BNamed "r6"] (BinOp OffsetOp (Var "r6") (Lit (LitInt n)))) [] []]).
    tc_solve.
  - eapply (chain_pure 1 1); tc_solve.
Qed.

Section target.
  Context `{!lrustGS Σ}.
  Lemma wp_end_code p n (R : iProp Σ) :
    R -∗ WP (App (of_val end_code)
      (Lit (LitLoc (pointer_loc p)) :: Lit (LitInt n) :: nil))
      {{ v, ⌜v = LitV (LitLoc (pointer_loc (shifted p n)))⌝ ∗ R }}.
  Proof.
    iIntros "R". wp_rec. wp_rec. wp_let. wp_op.
    rewrite pointer_loc_shift. iFrame. done.
  Qed.

  Theorem end_segment_refinement r σ p n (R : iProp Σ) :
    r !! 3%nat = Some (NonNullThin p) → r !! 2%nat = Some (Usize n) → offset_valid p n →
    R -∗ ⌜segment_run end_segment r σ (end_registers r p n) σ⌝ ∗
      WP (App (of_val end_code)
        (Lit (LitLoc (pointer_loc p)) :: Lit (LitInt n) :: nil))
        {{ v, ⌜end_registers r p n !! 5%nat = Some (RawThin (shifted p n)) ∧
          v = LitV (LitLoc (pointer_loc (shifted p n)))⌝ ∗ R }}.
  Proof.
    iIntros (Start Length Valid) "R". iSplit; first by iPureIntro; apply end_segment_exec.
    iApply (wp_wand with "[R]").
    { iApply (wp_end_code p n R with "R"). }
    iIntros (v) "[%Eq R]". iFrame. iPureIntro. split; last done.
    exact (proj2 (end_segment_outputs r p n)).
  Qed.
End target.
