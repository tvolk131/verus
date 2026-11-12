extern crate selection_checked;
use selection_checked::*;
use vstd::prelude::*;
verus! {
pub fn imported_selection(h: &mut UnrepairedDatabaseHeader) -> (r: Result<bool>)
    requires old(h).inner.primary_slot < 2, (old(h).inner.primary_slot ^ 1) < 2,
        old(h).inner.primary_slot != (old(h).inner.primary_slot ^ 1),
    ensures r.is_Err() == selection_errors(*old(h)),
        match r { Ok(kept) => kept == !selection_switches(*old(h)), _ => true },
        final(h).inner.primary_slot == (if selection_switches(*old(h)) {
            old(h).inner.primary_slot ^ 1 } else { old(h).inner.primary_slot }),
        final(h).inner.transaction_slots == old(h).inner.transaction_slots,
{ h.select_primary_slot() }
}
