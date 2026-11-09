use vstd::prelude::*;
use persistence94::*;

verus! {

pub proof fn reject_selector_publication_without_first_barrier() {
    let initial = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    let payload_issued = written(initial, 0, seq![1u8]);
    write_transition_exists(initial, 0, seq![1u8]);
    let published = written(payload_issued, 1, seq![1u8]);
    write_transition_exists(payload_issued, 1, seq![1u8]);
    let interrupted = StorageImage { durable: seq![0u8, 1u8], visible: published.visible,
        permitted: published.permitted };
    assert forall|index: int| 0 <= index < interrupted.durable.len() implies
        #[trigger] published.permitted[index].contains(interrupted.durable[index]) by {
        if index == 0 {} else { assert(index == 1); }
    }
    assert(persist_some(published, interrupted));
    persistence_preserves_storage(published, interrupted);
    assert(interrupted.durable[1] == 1u8);
    assert(interrupted.durable[0] == published.visible[0]);
}

pub proof fn reject_acknowledgment_without_final_sync() {
    let initial = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    let payload_issued = written(initial, 0, seq![1u8]);
    write_transition_exists(initial, 0, seq![1u8]);
    let prepared = clean_storage(payload_issued.visible);
    sync_transition_exists(payload_issued);
    let published = written(prepared, 1, seq![1u8]);
    write_transition_exists(prepared, 1, seq![1u8]);
    assert(published.visible[1] == 1u8);
    assert(published.durable[1] == 1u8);
}

pub proof fn reject_pending_overwrite_as_protected_frame() {
    let original = seq![0u8];
    let before = clean_storage(original);
    clean_storage_is_valid(original);
    let selected = set![0int];
    assert(protected_bytes(before, original, selected));
    let issued = written(before, 0, seq![1u8]);
    write_transition_exists(before, 0, seq![1u8]);
    assert(issued.durable == before.durable);
    assert(protected_bytes(issued, original, selected));
}

}
