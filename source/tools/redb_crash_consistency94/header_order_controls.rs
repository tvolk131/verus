use vstd::prelude::*;
use persistence94::*;
use headerorder94::*;

verus! {

pub proof fn reject_whole_header_without_first_barrier() {
    let initial = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    let before_barrier = written(initial, 0, seq![1u8]);
    write_transition_exists(initial, 0, seq![1u8]);
    let rewritten = written(before_barrier, 0, seq![1u8, 1u8]);
    write_transition_exists(before_barrier, 0, seq![1u8, 1u8]);
    assert(persist_some(rewritten, rewritten));
    two_phase_whole_header_rewrite_preserves_prepared_payload(
        before_barrier, before_barrier, rewritten, rewritten, 0, seq![1u8, 1u8], 1);
}

pub proof fn reject_changing_nonselector_after_prepare_barrier() {
    let prepared = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    sync_transition_exists(prepared);
    let rewritten = written(prepared, 0, seq![9u8, 1u8]);
    write_transition_exists(prepared, 0, seq![9u8, 1u8]);
    assert(persist_some(rewritten, rewritten));
    two_phase_whole_header_rewrite_preserves_prepared_payload(
        prepared, prepared, rewritten, rewritten, 0, seq![9u8, 1u8], 1);
}

pub proof fn reject_issued_selector_as_final_acknowledgment() {
    let prepared = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    sync_transition_exists(prepared);
    let rewritten = written(prepared, 0, seq![0u8, 1u8]);
    write_transition_exists(prepared, 0, seq![0u8, 1u8]);
    assert(persist_some(rewritten, rewritten));
    two_phase_whole_header_rewrite_preserves_prepared_payload(
        prepared, prepared, rewritten, rewritten, 0, seq![0u8, 1u8], 1);
    assert(rewritten.durable[1] == 1u8);
}

pub proof fn reject_final_sync_contract_without_sync() {
    let prepared = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    let rewritten = written(prepared, 0, seq![0u8, 1u8]);
    write_transition_exists(prepared, 0, seq![0u8, 1u8]);
    successful_sync_of_full_header_publishes_every_field(prepared, rewritten, rewritten, 0, seq![0u8, 1u8]);
}

pub proof fn reject_mixed_generation_primary_and_mode() {
    atomic_selector_keeps_mode_and_primary_in_one_generation(0u8, 5u8, 1u8);
}

}
