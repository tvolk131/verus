From lrust.typing Require Import soundness borrow.
From NativeSemanticBridge Require Import borrow_handoff byte_array raw_view model_limits.
Set Printing Width 1000.
Redirect "00-statement" About lrust.lang.adequacy.lrust_adequacy.
Redirect "00-assumptions" Print Assumptions lrust.lang.adequacy.lrust_adequacy.
Redirect "01-statement" About lrust.typing.soundness.type_soundness_closed.
Redirect "01-assumptions" Print Assumptions lrust.typing.soundness.type_soundness_closed.
Redirect "02-statement" About lrust.typing.soundness.main_type.
Redirect "02-assumptions" Print Assumptions lrust.typing.soundness.main_type.
Redirect "03-statement" About lrust.lang.heap.heap_ctx.
Redirect "03-assumptions" Print Assumptions lrust.lang.heap.heap_ctx.
Redirect "04-statement" About lrust.lang.lifting.lrustGS_irisGS.
Redirect "04-assumptions" Print Assumptions lrust.lang.lifting.lrustGS_irisGS.
Redirect "05-statement" About lrust.util.non_atomic_cell_map.non_atomic_map_alloc_empty.
Redirect "05-assumptions" Print Assumptions lrust.util.non_atomic_cell_map.non_atomic_map_alloc_empty.
Redirect "06-statement" About lrust.util.non_atomic_cell_map.non_atomic_map_insert.
Redirect "06-assumptions" Print Assumptions lrust.util.non_atomic_cell_map.non_atomic_map_insert.
Redirect "07-statement" About lrust.lifetime.lifetime_full.llft_alloc.
Redirect "07-assumptions" Print Assumptions lrust.lifetime.lifetime_full.llft_alloc.
Redirect "08-statement" About lrust.prophecy.prophecy.proph_init.
Redirect "08-assumptions" Print Assumptions lrust.prophecy.prophecy.proph_init.
Redirect "09-statement" About lrust.typing.uniq_cmra.uniq_init.
Redirect "09-assumptions" Print Assumptions lrust.typing.uniq_cmra.uniq_init.
Redirect "10-statement" About lrust.typing.uniq_util.alloc_uniq_body.
Redirect "10-assumptions" Print Assumptions lrust.typing.uniq_util.alloc_uniq_body.
Redirect "11-statement" About lrust.typing.borrow.type_uniqbor_instr.
Redirect "11-assumptions" Print Assumptions lrust.typing.borrow.type_uniqbor_instr.
Redirect "12-statement" About lrust.typing.borrow.tctx_reborrow_uniq.
Redirect "12-assumptions" Print Assumptions lrust.typing.borrow.tctx_reborrow_uniq.
Redirect "13-statement" About lrust.typing.type_context.tctx_elt_interp.
Redirect "13-assumptions" Print Assumptions lrust.typing.type_context.tctx_elt_interp.
Redirect "14-statement" About lrust.typing.programs.typed_instr.
Redirect "14-assumptions" Print Assumptions lrust.typing.programs.typed_instr.
Redirect "15-statement" About lrust.typing.type.ty_own.
Redirect "15-assumptions" Print Assumptions lrust.typing.type.ty_own.
Redirect "16-statement" About lrust.typing.uniq_bor.uniq_bor.
Redirect "16-assumptions" Print Assumptions lrust.typing.uniq_bor.uniq_bor.
Redirect "17-statement" About NativeSemanticBridge.borrow_handoff.source_loan_access.
Redirect "17-assumptions" Print Assumptions NativeSemanticBridge.borrow_handoff.source_loan_access.
Redirect "18-statement" About NativeSemanticBridge.borrow_handoff.source_loan_release.
Redirect "18-assumptions" Print Assumptions NativeSemanticBridge.borrow_handoff.source_loan_release.
Redirect "19-statement" About NativeSemanticBridge.borrow_handoff.source_context_access.
Redirect "19-assumptions" Print Assumptions NativeSemanticBridge.borrow_handoff.source_context_access.
Redirect "20-statement" About NativeSemanticBridge.byte_array.byte_list_length.
Redirect "20-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_list_length.
Redirect "21-statement" About NativeSemanticBridge.byte_array.bytes_size.
Redirect "21-assumptions" Print Assumptions NativeSemanticBridge.byte_array.bytes_size.
Redirect "22-statement" About NativeSemanticBridge.byte_array.bytes_physical.
Redirect "22-assumptions" Print Assumptions NativeSemanticBridge.byte_array.bytes_physical.
Redirect "23-statement" About NativeSemanticBridge.byte_array.bytes_initialized.
Redirect "23-assumptions" Print Assumptions NativeSemanticBridge.byte_array.bytes_initialized.
Redirect "24-statement" About NativeSemanticBridge.byte_array.byte_array128_size.
Redirect "24-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_array128_size.
Redirect "25-statement" About NativeSemanticBridge.byte_array.byte_array128_release.
Redirect "25-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_array128_release.
Redirect "26-statement" About NativeSemanticBridge.byte_array.byte_array128_scoped_handoff.
Redirect "26-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_array128_scoped_handoff.
Redirect "27-statement" About NativeSemanticBridge.byte_array.byte_array128_context_handoff.
Redirect "27-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_array128_context_handoff.
Redirect "28-statement" About NativeSemanticBridge.raw_view.typed_source_raw_view.
Redirect "28-assumptions" Print Assumptions NativeSemanticBridge.raw_view.typed_source_raw_view.
Redirect "29-statement" About NativeSemanticBridge.raw_view.typed_source_raw_offset.
Redirect "29-assumptions" Print Assumptions NativeSemanticBridge.raw_view.typed_source_raw_offset.
Redirect "30-statement" About NativeSemanticBridge.raw_view.typed_source_loan_release.
Redirect "30-assumptions" Print Assumptions NativeSemanticBridge.raw_view.typed_source_loan_release.
Redirect "31-statement" About NativeSemanticBridge.model_limits.arbitrary_raw_pointer_typed.
Redirect "31-assumptions" Print Assumptions NativeSemanticBridge.model_limits.arbitrary_raw_pointer_typed.
Redirect "32-statement" About NativeSemanticBridge.model_limits.raw_read_empty_head_stuck.
Redirect "32-assumptions" Print Assumptions NativeSemanticBridge.model_limits.raw_read_empty_head_stuck.
Redirect "33-statement" About NativeSemanticBridge.model_limits.model_offset_unrestricted.
Redirect "33-assumptions" Print Assumptions NativeSemanticBridge.model_limits.model_offset_unrestricted.
Redirect "34-statement" About NativeSemanticBridge.model_limits.empty_heap_offset_adequate.
Redirect "34-assumptions" Print Assumptions NativeSemanticBridge.model_limits.empty_heap_offset_adequate.
Redirect "35-statement" About NativeSemanticBridge.byte_array.byte_ty_obligation_1.
Redirect "35-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_1.
Redirect "36-statement" About NativeSemanticBridge.byte_array.byte_ty_obligation_2.
Redirect "36-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_2.
Redirect "37-statement" About NativeSemanticBridge.byte_array.byte_ty_obligation_3.
Redirect "37-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_3.
Redirect "38-statement" About NativeSemanticBridge.byte_array.byte_ty_obligation_4.
Redirect "38-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_4.
Redirect "39-statement" About NativeSemanticBridge.byte_array.byte_ty_obligation_5.
Redirect "39-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_5.
Redirect "40-statement" About NativeSemanticBridge.byte_array.byte_ty_obligation_6.
Redirect "40-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty_obligation_6.
Redirect "41-statement" About NativeSemanticBridge.byte_array.byte_ty.
Redirect "41-assumptions" Print Assumptions NativeSemanticBridge.byte_array.byte_ty.
Fail Check non_atomic_map_alloc_heap.
