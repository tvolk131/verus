From NativeMirLrust Require Export simulation.
Open Scope Z_scope.
Definition selected_operations : list operation := [Metadata; WrapWide; UnwrapWide; Thin; WrapThin; UnwrapThin; ByteOffset; U8ZstBranch].
Lemma selected_operations_exact : selected_operations =
 [Metadata; WrapWide; UnwrapWide; Thin; WrapThin; UnwrapThin; ByteOffset; U8ZstBranch].
Proof. reflexivity. Qed.
Lemma selected_operations_supported : Forall (λ op, ∀ why, op ≠ Unsupported why) selected_operations.
Proof. repeat constructor; intros why H; discriminate. Qed.
Definition selected_edges w start : list (operation * list descriptor * list descriptor) :=
 let p := Pointer w start in [
 (Metadata, [SliceDesc p 128], [Usize 128]);
 (WrapWide, [RawWide p 128], [NonNullWide p 128]);
 (UnwrapWide, [NonNullWide p 128], [RawWide p 128]);
 (Thin, [RawWide p 128], [RawThin p]);
 (WrapThin, [RawThin p], [NonNullThin p]);
 (UnwrapThin, [NonNullThin p], [RawThin p]);
 (ByteOffset, [RawThin p; Usize 128], [RawThin (shifted p 128)]);
 (U8ZstBranch, [Boolean false], [Usize 2])].

Lemma selected_edges_checked w start σ :
  window_valid w → 0 < start → start + 128 < window_extent w →
  Forall (λ edge, source_step edge.1.1 edge.1.2 σ edge.2 σ) (selected_edges w start).
Proof.
  intros W S E.
  destruct (interior128_offset w start W S E) as [Off _].
  assert (PV : pointer_valid (Pointer w start)) by exact (proj1 Off).
  assert (SL : slice_valid (Pointer w start) 128).
  { split; first done. split; [lia|exact (proj2 (proj2 Off))]. }
  unfold selected_edges.
  constructor. { apply StepMetadata. exact SL. }
  constructor. { apply StepWrapWide. exact SL. }
  constructor. { apply StepUnwrapWide. exact SL. }
  constructor. { apply StepThin. exact SL. }
  constructor. { apply StepWrapThin. exact PV. }
  constructor. { apply StepUnwrapThin. exact PV. }
  constructor. { apply StepByteOffset. exact Off. }
  constructor. { constructor. }
  constructor.
Qed.

Theorem selected_edges_head_simulation w start σ :
  window_valid w → 0 < start → start + 128 < window_extent w →
  Forall (λ edge, ∃ es,
    translate edge.1.1 (input_words edge.1.2) = Some es ∧
    Forall2 (λ e v, head_step e σ [] (of_val v) σ [])
      es (input_words edge.2)) (selected_edges w start).
Proof.
  intros W S E.
  pose proof (selected_edges_checked w start σ W S E) as Steps.
  eapply Forall_impl; first exact Steps.
  intros [[op ins] outs] Step.
  destruct (supported_operation_head_simulation op ins σ outs σ Step)
    as [_ [es [Tr Sim]]]. exists es. split; first done.
  eapply Forall2_impl; first exact Sim. intros e v [Head _]. exact Head.
Qed.
