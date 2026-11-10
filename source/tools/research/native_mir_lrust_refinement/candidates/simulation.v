From NativeMirLrust Require Export fragment.
Set Default Proof Using "Type".

Lemma word_execution_step e v σ : word_execution e v →
  head_step e σ [] (of_val v) σ [].
Proof.
  intros W. inversion W; subst.
  - eapply BetaS.
    + constructor. { rewrite to_of_val. eexists. reflexivity. }
      constructor. { rewrite to_of_val. eexists. reflexivity. } constructor.
    + done.
    + done.
  - eapply BetaS.
    + constructor. { rewrite to_of_val. eexists. reflexivity. }
      constructor. { rewrite to_of_val. eexists. reflexivity. } constructor.
    + done.
    + done.
  - eapply BetaS.
    + constructor. { rewrite to_of_val. eexists. reflexivity. } constructor.
    + done.
    + done.
  - apply BinOpS. constructor.
  - apply CaseS; done.
Qed.

Lemma word_execution_deterministic e v σ κ e' σ' fs : word_execution e v →
  head_step e σ κ e' σ' fs →
  κ = [] ∧ e' = of_val v ∧ σ' = σ ∧ fs = [].
Proof.
  intros W Step. inversion W; subst;
    inversion Step; subst; simpl in *; try congruence.
  - repeat split; congruence.
  - repeat split; congruence.
  - repeat split; congruence.
  - match goal with H : bin_op_eval _ _ _ _ _ |- _ => inversion H; subst end. done.
  - repeat split; congruence.
Qed.

Theorem supported_operation_head_simulation op ds σ outs σ' :
  source_step op ds σ outs σ' →
  σ' = σ ∧ ∃ es, translate op (input_words ds) = Some es ∧
  Forall2 (λ e v, head_step e σ [] (of_val v) σ [] ∧
    ∀ κ e' τ fs, head_step e σ κ e' τ fs →
      κ = [] ∧ e' = of_val v ∧ τ = σ ∧ fs = []) es (input_words outs).
Proof.
  intros Step. destruct (supported_operation_simulation _ _ _ _ _ Step)
    as [Eq [es [Tr Sim]]]. split; first done. exists es. split; first done.
  eapply Forall2_impl; first exact Sim.
  intros e v W. split; first by apply word_execution_step.
  intros. by eapply word_execution_deterministic.
Qed.
