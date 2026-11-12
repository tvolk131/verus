extern crate supported;
use supported::*;
use vstd::prelude::*;
verus! {
pub fn imported_publication(h: &DatabaseHeader) -> (r: &TransactionHeader)
    requires true,
    ensures *r == h.transaction_slots[(if h.primary_unflushed { h.primary_slot ^ 1 } else { h.primary_slot }) as int],
{ h.published_slot() }
}
