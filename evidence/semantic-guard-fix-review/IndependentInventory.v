From NativeSemanticBridge Require Import borrow_handoff byte_array raw_view model_limits.
Goal True. idtac "AUDIT-BEGIN borrow_handoff.source_loan_access". exact I. Qed.
Check NativeSemanticBridge.borrow_handoff.source_loan_access.
Print Assumptions NativeSemanticBridge.borrow_handoff.source_loan_access.
Goal True. idtac "AUDIT-END borrow_handoff.source_loan_access". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN borrow_handoff.source_loan_release". exact I. Qed.
Check NativeSemanticBridge.borrow_handoff.source_loan_release.
Print Assumptions NativeSemanticBridge.borrow_handoff.source_loan_release.
Goal True. idtac "AUDIT-END borrow_handoff.source_loan_release". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN borrow_handoff.source_context_access". exact I. Qed.
Check NativeSemanticBridge.borrow_handoff.source_context_access.
Print Assumptions NativeSemanticBridge.borrow_handoff.source_context_access.
Goal True. idtac "AUDIT-END borrow_handoff.source_context_access". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_list_length". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_list_length.
Print Assumptions NativeSemanticBridge.byte_array.byte_list_length.
Goal True. idtac "AUDIT-END byte_array.byte_list_length". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.bytes_size". exact I. Qed.
Check NativeSemanticBridge.byte_array.bytes_size.
Print Assumptions NativeSemanticBridge.byte_array.bytes_size.
Goal True. idtac "AUDIT-END byte_array.bytes_size". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.bytes_physical". exact I. Qed.
Check NativeSemanticBridge.byte_array.bytes_physical.
Print Assumptions NativeSemanticBridge.byte_array.bytes_physical.
Goal True. idtac "AUDIT-END byte_array.bytes_physical". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.bytes_initialized". exact I. Qed.
Check NativeSemanticBridge.byte_array.bytes_initialized.
Print Assumptions NativeSemanticBridge.byte_array.bytes_initialized.
Goal True. idtac "AUDIT-END byte_array.bytes_initialized". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_array128_size". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_array128_size.
Print Assumptions NativeSemanticBridge.byte_array.byte_array128_size.
Goal True. idtac "AUDIT-END byte_array.byte_array128_size". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_array128_release". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_array128_release.
Print Assumptions NativeSemanticBridge.byte_array.byte_array128_release.
Goal True. idtac "AUDIT-END byte_array.byte_array128_release". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_array128_scoped_handoff". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_array128_scoped_handoff.
Print Assumptions NativeSemanticBridge.byte_array.byte_array128_scoped_handoff.
Goal True. idtac "AUDIT-END byte_array.byte_array128_scoped_handoff". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_array128_context_handoff". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_array128_context_handoff.
Print Assumptions NativeSemanticBridge.byte_array.byte_array128_context_handoff.
Goal True. idtac "AUDIT-END byte_array.byte_array128_context_handoff". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN raw_view.typed_source_raw_view". exact I. Qed.
Check NativeSemanticBridge.raw_view.typed_source_raw_view.
Print Assumptions NativeSemanticBridge.raw_view.typed_source_raw_view.
Goal True. idtac "AUDIT-END raw_view.typed_source_raw_view". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN raw_view.typed_source_raw_offset". exact I. Qed.
Check NativeSemanticBridge.raw_view.typed_source_raw_offset.
Print Assumptions NativeSemanticBridge.raw_view.typed_source_raw_offset.
Goal True. idtac "AUDIT-END raw_view.typed_source_raw_offset". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN raw_view.typed_source_loan_release". exact I. Qed.
Check NativeSemanticBridge.raw_view.typed_source_loan_release.
Print Assumptions NativeSemanticBridge.raw_view.typed_source_loan_release.
Goal True. idtac "AUDIT-END raw_view.typed_source_loan_release". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN model_limits.arbitrary_raw_pointer_typed". exact I. Qed.
Check NativeSemanticBridge.model_limits.arbitrary_raw_pointer_typed.
Print Assumptions NativeSemanticBridge.model_limits.arbitrary_raw_pointer_typed.
Goal True. idtac "AUDIT-END model_limits.arbitrary_raw_pointer_typed". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN model_limits.raw_read_empty_head_stuck". exact I. Qed.
Check NativeSemanticBridge.model_limits.raw_read_empty_head_stuck.
Print Assumptions NativeSemanticBridge.model_limits.raw_read_empty_head_stuck.
Goal True. idtac "AUDIT-END model_limits.raw_read_empty_head_stuck". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN model_limits.model_offset_unrestricted". exact I. Qed.
Check NativeSemanticBridge.model_limits.model_offset_unrestricted.
Print Assumptions NativeSemanticBridge.model_limits.model_offset_unrestricted.
Goal True. idtac "AUDIT-END model_limits.model_offset_unrestricted". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN model_limits.empty_heap_offset_adequate". exact I. Qed.
Check NativeSemanticBridge.model_limits.empty_heap_offset_adequate.
Print Assumptions NativeSemanticBridge.model_limits.empty_heap_offset_adequate.
Goal True. idtac "AUDIT-END model_limits.empty_heap_offset_adequate". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty_obligation_1". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty_obligation_1.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_1.
Goal True. idtac "AUDIT-END byte_array.byte_ty_obligation_1". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty_obligation_2". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty_obligation_2.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_2.
Goal True. idtac "AUDIT-END byte_array.byte_ty_obligation_2". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty_obligation_3". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty_obligation_3.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_3.
Goal True. idtac "AUDIT-END byte_array.byte_ty_obligation_3". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty_obligation_4". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty_obligation_4.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_4.
Goal True. idtac "AUDIT-END byte_array.byte_ty_obligation_4". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty_obligation_5". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty_obligation_5.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_5.
Goal True. idtac "AUDIT-END byte_array.byte_ty_obligation_5". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty_obligation_6". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty_obligation_6.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_6.
Goal True. idtac "AUDIT-END byte_array.byte_ty_obligation_6". exact I. Qed.
Goal True. idtac "AUDIT-BEGIN byte_array.byte_ty". exact I. Qed.
Check NativeSemanticBridge.byte_array.byte_ty.
Print Assumptions NativeSemanticBridge.byte_array.byte_ty.
Goal True. idtac "AUDIT-END byte_array.byte_ty". exact I. Qed.
Fail Check NativeSemanticBridge.byte_array.byte_ty_obligation_7.
