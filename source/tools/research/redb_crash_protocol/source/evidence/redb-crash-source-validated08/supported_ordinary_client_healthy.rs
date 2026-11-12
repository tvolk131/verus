extern crate supported;
use supported::*;
use vstd::prelude::*;
verus! {
pub fn imported_publication(h: &DatabaseHeader) -> (r: &TransactionHeader)
    requires h.primary_slot < 2, (h.primary_slot ^ 1) < 2, h.primary_slot != (h.primary_slot ^ 1),
    ensures *r == h.transaction_slots[(if h.primary_unflushed { h.primary_slot ^ 1 } else { h.primary_slot }) as int],
{ h.published_slot() }
}
