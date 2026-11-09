use vstd::prelude::*;
use headerslots94::*;

verus! {

pub fn imported_slot_clone(header: &TransactionHeader) -> (result: TransactionHeader)
    ensures result == *header,
{
    header.clone()
}

pub fn imported_header_clone(header: &DatabaseHeader) -> (result: DatabaseHeader)
    ensures result == *header,
{
    checked_database_header_clone(header)
}

pub fn imported_header_clone_twice(header: &DatabaseHeader) -> (result: DatabaseHeader)
    ensures result == *header,
{
    let first = checked_database_header_clone(header);
    checked_database_header_clone(&first)
}

pub fn imported_reader_clone(header: &DatabaseHeader) -> (result: TransactionHeader)
    requires header.primary_slot < 2,
    ensures result == header.transaction_slots@[
        (if header.primary_unflushed { header.primary_slot ^ 1 } else { header.primary_slot }) as int],
{
    let copied = checked_database_header_clone(header);
    copied.published_slot().clone()
}

}
