use vstd::prelude::*;
use persistence94::*;
use filelength94::*;

verus! {

pub proof fn issued_length_is_not_durable() {
    let before = clean_file(seq![0u8, 0u8, 0u8, 0u8], 2);
    let resized = FileImage { bytes: written(before.bytes, 2, Seq::new(2, |index: int| 0u8)),
        durable_len: 2, visible_len: 4,
        permitted_lengths: before.permitted_lengths.union(intermediate_lengths(2, 4)) };
    clean_file_is_valid(seq![0u8, 0u8, 0u8, 0u8], 2);
    intermediate_length_membership(2, 4);
    assert(resize_issued(before, resized, 4));
    resize_preserves_file(before, resized, 4);
    assert(resized.durable_len == 4);
}

pub proof fn pending_length_is_not_atomic() {
    intermediate_length_membership(2, 5);
    assert(!intermediate_lengths(2, 5).contains(3));
}

pub proof fn unsafe_shrink_cannot_preserve_live_byte(
    before: FileImage, after: FileImage, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected), selected.contains(3),
        resize_issued(before, after, 2),
{
    safe_resize_preserves_live_protection(before, after, 2, original, selected);
}

pub proof fn invalid_range_cannot_be_issued(
    before: FileImage, after: FileImage, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected),
        before.visible_len == 2, before.bytes.visible.len() == 4,
        issue_write(before.bytes, after.bytes, 3, seq![1u8]),
        after.durable_len == before.durable_len, after.visible_len == before.visible_len,
        after.permitted_lengths == before.permitted_lengths,
{
    in_bounds_write_preserves_file(before, after, 3, seq![1u8]);
}

pub proof fn failed_sync_does_not_imply_rollback() {
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
    assert(persisted.bytes.durable == before.bytes.durable);
}

pub proof fn failed_sync_does_not_acknowledge() {
    let before = clean_file(seq![1u8, 2u8], 2);
    let issued = FileImage { bytes: written(before.bytes, 1, seq![9u8]), durable_len: 2, visible_len: 2,
        permitted_lengths: before.permitted_lengths };
    clean_file_is_valid(seq![1u8, 2u8], 2);
    write_transition_exists(before.bytes, 1, seq![9u8]);
    in_bounds_write_preserves_file(before, issued, 1, seq![9u8]);
    assert(allowed_file_step(issued, issued, FileStep::FailedSync, Set::empty()));
    assert(issued.bytes.durable[1] == 9u8);
}

}
