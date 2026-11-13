use vstd::prelude::*;
use persistence94::*;
use filelength94::*;

verus! {

pub proof fn safe_resize_and_crash(
    before: FileImage, resized: FileImage, crashed: FileImage,
    new_length: int, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected),
        resize_issued(before, resized, new_length), file_power_restart(resized, crashed),
        forall|index: int| selected.contains(index) ==> index < new_length,
    ensures valid_file(crashed), forall|index: int| selected.contains(index) ==>
        0 <= index < crashed.durable_len && #[trigger] crashed.bytes.durable[index] == original[index],
{
    safe_resize_preserves_live_protection(before, resized, new_length, original, selected);
    restart_preserves_live_protection(resized, crashed, original, selected);
    live_selected_bytes_survive(crashed, original, selected);
}

pub proof fn sync_then_power_loss_retains_publication(before: FileImage, synced: FileImage, restarted: FileImage)
    requires valid_file(before), file_sync(before, synced), file_power_restart(synced, restarted),
    ensures restarted.visible_len == before.visible_len,
        restarted.bytes.visible.subrange(0, restarted.visible_len)
            == before.bytes.visible.subrange(0, before.visible_len),
{
    successful_sync_publishes_bytes_and_length(before, synced);
}

pub proof fn intermediate_crash_lengths_are_admitted() {
    let before = clean_file(seq![1u8, 2u8, 0u8, 0u8, 0u8], 2);
    let resized = FileImage { bytes: written(before.bytes, 2, Seq::new(3, |index: int| 0u8)),
        durable_len: 2, visible_len: 5,
        permitted_lengths: before.permitted_lengths.union(intermediate_lengths(2, 5)) };
    let persisted = FileImage { bytes: resized.bytes, durable_len: 3, visible_len: 5,
        permitted_lengths: resized.permitted_lengths };
    clean_file_is_valid(seq![1u8, 2u8, 0u8, 0u8, 0u8], 2);
    intermediate_length_membership(2, 5);
    assert(resize_issued(before, resized, 5));
    resize_preserves_file(before, resized, 5);
    assert(file_persistence(resized, persisted));
    persistence_preserves_file(resized, persisted);
    assert(persisted.durable_len != before.durable_len && persisted.durable_len != resized.visible_len);
}

pub proof fn failed_sync_can_have_completed_effects() {
    let before = clean_file(seq![1u8, 2u8], 2);
    let issued = FileImage { bytes: written(before.bytes, 1, seq![9u8]), durable_len: 2, visible_len: 2,
        permitted_lengths: before.permitted_lengths };
    let persisted = FileImage { bytes: StorageImage { durable: seq![1u8, 9u8],
        visible: issued.bytes.visible, permitted: issued.bytes.permitted }, durable_len: 2, visible_len: 2,
        permitted_lengths: issued.permitted_lengths };
    clean_file_is_valid(seq![1u8, 2u8], 2);
    write_transition_exists(before.bytes, 1, seq![9u8]);
    assert(in_bounds_write(before, issued, 1, seq![9u8]));
    in_bounds_write_preserves_file(before, issued, 1, seq![9u8]);
    assert(issued.bytes.visible =~= seq![1u8, 9u8]);
    assert(allowed_file_step(issued, persisted, FileStep::FailedSync, Set::empty()));
    assert(persisted.bytes.durable[1] == 9u8 && before.bytes.durable[1] == 2u8);
}

pub proof fn arbitrary_file_trace_read_at_position(
    states: Seq<FileImage>, steps: Seq<FileStep>, original: Seq<u8>, selected: Set<int>,
    position: int, index: int)
    requires allowed_file_trace(states, steps, selected), valid_file(states[0]),
        live_protection(states[0], original, selected),
        0 <= position < states.len(), selected.contains(index),
    ensures 0 <= index < states[position].durable_len,
        states[position].bytes.durable[index] == original[index],
{
    trace_preserves_live_protection(states, steps, original, selected, position as nat);
    live_selected_bytes_survive(states[position], original, selected);
}

pub proof fn subset_interrupted_write_is_valid() {
    let before = clean_file(seq![1u8, 2u8], 2);
    let requested = seq![8u8, 9u8];
    let touched = Set::empty().insert(1);
    let partial = partially_issued_bytes(before, 0, requested, touched);
    let after = FileImage { bytes: written(before.bytes, 0, partial),
        durable_len: 2, visible_len: 2, permitted_lengths: before.permitted_lengths };
    clean_file_is_valid(seq![1u8, 2u8], 2);
    write_transition_exists(before.bytes, 0, partial);
    assert(allowed_file_step(before, after, FileStep::InterruptedWrite { offset: 0, requested, touched },
        Set::empty()));
    assert(after.bytes.visible[0] == 1u8 && after.bytes.visible[1] == 9u8);
    assert(after.bytes.durable == before.bytes.durable);
}

}
