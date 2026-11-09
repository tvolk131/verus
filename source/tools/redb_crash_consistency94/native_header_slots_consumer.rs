use vstd::prelude::*;
use rootcodec94::*;
use headerslots94::*;

verus! {

pub fn swap_twice_restores_header(header: &mut DatabaseHeader)
    requires old(header).primary_slot < 2,
    ensures *final(header) == *old(header),
{
    let ghost initial = *header;
    header.swap_primary_slot();
    header.swap_primary_slot();
    proof {
        other_slot_bounds(initial.primary_slot);
        assert(header.transaction_slots@ == initial.transaction_slots@);
        vstd::array::axiom_array_ext_equal(header.transaction_slots, initial.transaction_slots);
    }
}

pub fn publish_unflushed_keeps_previous_reader_slot(
    header: &mut DatabaseHeader, transaction_id: TransactionId,
    user_root: Option<BtreeHeader>, system_root: Option<BtreeHeader>)
    requires old(header).primary_slot < 2,
    ensures final(header).primary_slot == old(header).primary_slot ^ 1,
        final(header).primary_unflushed,
        final(header).transaction_slots@[final(header).primary_slot as int].transaction_id == transaction_id,
        final(header).transaction_slots@[final(header).primary_slot as int].user_root == user_root,
        final(header).transaction_slots@[final(header).primary_slot as int].system_root == system_root,
        final(header).transaction_slots@[(final(header).primary_slot ^ 1) as int]
            == old(header).transaction_slots@[old(header).primary_slot as int],
{
    let ghost previous_primary = header.transaction_slots@[header.primary_slot as int];
    let ghost previous_index = header.primary_slot;
    header.write_secondary_slot(transaction_id, user_root, system_root);
    header.swap_primary_slot();
    header.primary_unflushed = true;
    proof { other_slot_bounds(previous_index); }
    let published = header.published_slot();
    assert(*published == previous_primary);
}

pub fn overwrite_resets_only_new_slot_corruption(
    header: &mut DatabaseHeader, transaction_id: TransactionId,
    user_root: Option<BtreeHeader>, system_root: Option<BtreeHeader>)
    requires old(header).primary_slot < 2,
    ensures final(header).transaction_slots@[(final(header).primary_slot ^ 1) as int].corrupt_bytes.is_none(),
        final(header).transaction_slots@[final(header).primary_slot as int].corrupt_bytes
            == old(header).transaction_slots@[old(header).primary_slot as int].corrupt_bytes,
{
    header.write_secondary_slot(transaction_id, user_root, system_root);
}

pub fn finite_width_id_step(value: u64) -> (result: u64)
    requires value < u64::MAX,
    ensures result == value + 1,
{
    let mut transaction_id = TransactionId::new(value);
    let next = transaction_id.increment();
    assert(next == transaction_id);
    transaction_id.raw_id()
}

pub fn initialized_slot(transaction_id: TransactionId) -> (result: TransactionHeader)
    ensures result.user_root.is_none(), result.system_root.is_none(), result.corrupt_bytes.is_none(),
        result.transaction_id == transaction_id, result.version == FILE_FORMAT_VERSION3,
{
    TransactionHeader::new(transaction_id)
}

}
