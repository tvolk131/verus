use vstd::prelude::*;
use rootcodec94::*;
use headerslots94::*;

verus! {

pub fn swap_is_not_stuttering(header: &mut DatabaseHeader)
    requires old(header).primary_slot < 2,
{
    let original = header.primary_slot;
    header.swap_primary_slot();
    assert(header.primary_slot == original);
}

pub fn write_does_not_replace_primary(header: &mut DatabaseHeader, transaction_id: TransactionId)
    requires old(header).primary_slot < 2,
        old(header).transaction_slots@[old(header).primary_slot as int].transaction_id != transaction_id,
{
    header.write_secondary_slot(transaction_id, None, None);
    let primary = header.primary_slot();
    assert(primary.transaction_id == transaction_id);
}

pub fn unflushed_read_is_not_new_primary(header: &DatabaseHeader)
    requires header.primary_slot < 2, header.primary_unflushed,
        header.transaction_slots@[0].transaction_id != header.transaction_slots@[1].transaction_id,
{
    let published = header.published_slot();
    assert(published.transaction_id == header.transaction_slots@[header.primary_slot as int].transaction_id);
}

pub fn rewritten_secondary_is_not_corrupt(header: &mut DatabaseHeader, transaction_id: TransactionId)
    requires old(header).primary_slot < 2,
{
    header.write_secondary_slot(transaction_id, None, None);
    let secondary = header.secondary_slot();
    assert(secondary.corrupt_bytes.is_some());
}

pub fn id_exhaustion_is_not_ignored() {
    let last = TransactionId::new(u64::MAX);
    let _invalid = last.next();
}

pub fn missing_slot_bound_is_not_accepted(header: &mut DatabaseHeader)
    requires old(header).primary_slot == 2,
{
    header.swap_primary_slot();
}

}
