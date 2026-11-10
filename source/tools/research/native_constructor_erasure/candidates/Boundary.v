From NativeMirLrust Require Import fragment.

(* Reuse the actual completed descriptor definition, not a copy. This is a
   checked limitation of that definition, never a counterexample to Rust. *)
Definition root_descriptor_attempt : Prop :=
  exists ds sigma outs sigma',
    fragment.source_step (Unsupported Reservation) ds sigma outs sigma'.

Theorem root_descriptor_attempt_impossible : ~ root_descriptor_attempt.
Proof.
  intros [ds [sigma [outs [sigma' H]]]].
  exact (fragment.unsupported_rejected Reservation ds sigma outs sigma' H).
Qed.

Theorem activation_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported Activation) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem unsize_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported UnsizeReference) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem raw_formation_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported RawReferenceCreation) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem retag_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported Retag) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem storage_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported StorageLifetime) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem fake_read_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported FakeRead) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem drop_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported Drop) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem unwind_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported Unwind) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
Theorem phase_not_in_completed_semantics ds sigma outs sigma' :
  ~ fragment.source_step (Unsupported SourcePhase) ds sigma outs sigma'.
Proof. apply fragment.unsupported_rejected. Qed.
