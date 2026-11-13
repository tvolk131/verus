use vstd::prelude::*;
use persistence94::*;

verus! {

pub proof fn two_phase_whole_header_rewrite_preserves_prepared_payload(
    before_barrier: StorageImage, prepared: StorageImage, rewritten: StorageImage,
    interrupted: StorageImage, header_offset: int, header_bytes: Seq<u8>, selector: int)
    requires valid_storage(before_barrier), sync_success(before_barrier, prepared),
        header_offset <= selector < header_offset + header_bytes.len(),
        issue_write(prepared, rewritten, header_offset, header_bytes),
        forall|index: int| 0 <= index < header_bytes.len() && header_offset + index != selector ==>
            #[trigger] header_bytes[index] == before_barrier.visible[header_offset + index],
        persist_some(rewritten, interrupted),
    ensures valid_storage(interrupted),
        interrupted.durable[selector] == before_barrier.visible[selector]
            || interrupted.durable[selector] == header_bytes[selector - header_offset],
        forall|index: int| 0 <= index < before_barrier.visible.len() && index != selector ==>
            #[trigger] interrupted.durable[index] == before_barrier.visible[index],
{
    successful_sync_has_clean_image(before_barrier, prepared);
    sync_preserves_storage(before_barrier, prepared);
    write_preserves_storage(prepared, rewritten, header_offset, header_bytes);
    persistence_preserves_storage(rewritten, interrupted);
    assert forall|index: int| 0 <= index < before_barrier.visible.len() && index != selector implies
        #[trigger] interrupted.durable[index] == before_barrier.visible[index] by {
        if header_offset <= index < header_offset + header_bytes.len() {
            assert(header_bytes[index - header_offset] == before_barrier.visible[index]);
        }
        assert(rewritten.permitted[index] == Set::empty().insert(before_barrier.visible[index]));
        assert(rewritten.permitted[index].contains(interrupted.durable[index]));
    }
    assert(rewritten.permitted[selector]
        == Set::empty().insert(before_barrier.visible[selector]).insert(header_bytes[selector - header_offset]));
    assert(rewritten.permitted[selector].contains(interrupted.durable[selector]));
}

pub proof fn successful_sync_of_full_header_publishes_every_field(
    before: StorageImage, rewritten: StorageImage, acknowledged: StorageImage,
    header_offset: int, header_bytes: Seq<u8>)
    requires valid_storage(before), issue_write(before, rewritten, header_offset, header_bytes),
        sync_success(rewritten, acknowledged),
    ensures valid_storage(acknowledged),
        forall|index: int| 0 <= index < header_bytes.len() ==>
            #[trigger] acknowledged.durable[header_offset + index] == header_bytes[index],
        forall|index: int| 0 <= index < before.visible.len()
            && !(header_offset <= index < header_offset + header_bytes.len()) ==>
            #[trigger] acknowledged.durable[index] == before.visible[index],
{
    write_preserves_storage(before, rewritten, header_offset, header_bytes);
    sync_preserves_storage(rewritten, acknowledged);
}

pub proof fn atomic_selector_keeps_mode_and_primary_in_one_generation(
    old_byte: u8, new_byte: u8, recovered_byte: u8)
    requires recovered_byte == old_byte || recovered_byte == new_byte,
    ensures ((recovered_byte & 1u8) == (old_byte & 1u8)
            && (recovered_byte & 4u8) == (old_byte & 4u8))
        || ((recovered_byte & 1u8) == (new_byte & 1u8)
            && (recovered_byte & 4u8) == (new_byte & 4u8)),
{
}

}
