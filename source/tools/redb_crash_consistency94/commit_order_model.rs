use vstd::prelude::*;
use persistence94::*;

verus! {

pub proof fn two_phase_payload_precedes_selector_publication(
    before_barrier: StorageImage, prepared: StorageImage, published: StorageImage, interrupted: StorageImage,
    selector: int, new_value: u8)
    requires valid_storage(before_barrier), 0 <= selector < before_barrier.visible.len(),
        sync_success(before_barrier, prepared),
        issue_write(prepared, published, selector, seq![new_value]), persist_some(published, interrupted),
    ensures valid_storage(interrupted),
        interrupted.durable[selector] == before_barrier.visible[selector]
            || interrupted.durable[selector] == new_value,
        forall|index: int| 0 <= index < before_barrier.visible.len() && index != selector ==>
            #[trigger] interrupted.durable[index] == before_barrier.visible[index],
{
    successful_sync_has_clean_image(before_barrier, prepared);
    sync_preserves_storage(before_barrier, prepared);
    single_byte_update_cannot_mix_selector_bits(before_barrier.visible, selector, new_value, published, interrupted);
    write_preserves_storage(prepared, published, selector, seq![new_value]);
    persistence_preserves_storage(published, interrupted);
}

pub proof fn successful_final_sync_durably_publishes_visible_payload(
    before_publication: StorageImage, published: StorageImage, acknowledged: StorageImage,
    selector: int, new_value: u8)
    requires valid_storage(before_publication), 0 <= selector < before_publication.visible.len(),
        issue_write(before_publication, published, selector, seq![new_value]),
        sync_success(published, acknowledged),
    ensures valid_storage(acknowledged), acknowledged.durable[selector] == new_value,
        forall|index: int| 0 <= index < before_publication.visible.len() && index != selector ==>
            #[trigger] acknowledged.durable[index] == before_publication.visible[index],
{
    write_preserves_storage(before_publication, published, selector, seq![new_value]);
    sync_preserves_storage(published, acknowledged);
}

pub proof fn acknowledged_payload_survives_later_failure_and_recovery_steps(
    acknowledged: StorageImage, states: Seq<StorageImage>, steps: Seq<StorageStep>, selected: Set<int>)
    requires valid_storage(acknowledged), acknowledged == clean_storage(acknowledged.visible),
        states[0] == acknowledged, valid_trace(states, steps, selected),
        forall|index: int| #[trigger] selected.contains(index) ==> 0 <= index < acknowledged.visible.len(),
    ensures valid_storage(states.last()),
        forall|index: int| selected.contains(index) ==>
            #[trigger] states.last().durable[index] == acknowledged.durable[index],
{
    clean_image_protects_every_selected_byte(acknowledged.visible, selected);
    trace_preserves_protection(states, steps, acknowledged.visible, selected);
}

pub proof fn one_phase_partial_persistence_witness() -> (result: (StorageImage, StorageImage))
    ensures valid_storage(result.0), valid_storage(result.1), persist_some(result.0, result.1),
        result.0.visible == seq![1u8, 1u8], result.1.durable == seq![0u8, 1u8],
{
    let initial = clean_storage(seq![0u8, 0u8]);
    clean_storage_is_valid(seq![0u8, 0u8]);
    let payload_issued = written(initial, 0, seq![1u8]);
    write_transition_exists(initial, 0, seq![1u8]);
    let selector_issued = written(payload_issued, 1, seq![1u8]);
    write_transition_exists(payload_issued, 1, seq![1u8]);
    let interrupted = StorageImage { durable: seq![0u8, 1u8], visible: selector_issued.visible,
        permitted: selector_issued.permitted };
    assert(selector_issued.permitted[0].contains(0u8));
    assert(selector_issued.permitted[1].contains(1u8));
    assert forall|index: int| 0 <= index < interrupted.durable.len() implies
        #[trigger] selector_issued.permitted[index].contains(interrupted.durable[index]) by {
        if index == 0 {} else { assert(index == 1); }
    }
    persistence_preserves_storage(selector_issued, interrupted);
    (selector_issued, interrupted)
}

}
