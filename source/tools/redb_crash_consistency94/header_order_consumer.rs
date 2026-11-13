use vstd::prelude::*;
use persistence94::*;
use headerorder94::*;

verus! {

pub proof fn imported_whole_header_publication(
    before: StorageImage, prepared: StorageImage, rewritten: StorageImage,
    interrupted: StorageImage, offset: int, bytes: Seq<u8>, selector: int)
    requires valid_storage(before), sync_success(before, prepared),
        offset <= selector < offset + bytes.len(), issue_write(prepared, rewritten, offset, bytes),
        forall|index: int| 0 <= index < bytes.len() && offset + index != selector ==>
            #[trigger] bytes[index] == before.visible[offset + index],
        persist_some(rewritten, interrupted),
    ensures interrupted.durable[selector] == before.visible[selector]
            || interrupted.durable[selector] == bytes[selector - offset],
        forall|index: int| 0 <= index < before.visible.len() && index != selector ==>
            #[trigger] interrupted.durable[index] == before.visible[index],
{
    two_phase_whole_header_rewrite_preserves_prepared_payload(
        before, prepared, rewritten, interrupted, offset, bytes, selector);
}

pub proof fn imported_primary_and_mode_are_jointly_atomic(old_byte: u8, new_byte: u8, recovered: u8)
    requires recovered == old_byte || recovered == new_byte,
    ensures ((recovered & 1u8) == (old_byte & 1u8) && (recovered & 4u8) == (old_byte & 4u8))
        || ((recovered & 1u8) == (new_byte & 1u8) && (recovered & 4u8) == (new_byte & 4u8)),
{
    atomic_selector_keeps_mode_and_primary_in_one_generation(old_byte, new_byte, recovered);
}

}
