extern crate eligible;
use eligible::*;
use vstd::prelude::*;
verus! {
pub fn imported_finalize(h: UnrepairedDatabaseHeader) -> (r: Result<DatabaseHeader>)
    ensures r.is_Err() == !h.primary_corrupted,
        match r {
            Ok(header) => header == h.inner,
            Err(StorageError::Corrupted(msg)) => msg@ == "Primary commit slot is corrupted"@,
            _ => false,
        },
{ h.finalize_transaction_slots() }
pub fn imported_selection(h: &mut UnrepairedDatabaseHeader) -> Result<bool>
    requires old(h).inner.primary_slot < 2, (old(h).inner.primary_slot ^ 1) < 2,
        old(h).inner.primary_slot != (old(h).inner.primary_slot ^ 1),
{ h.select_primary_slot() }
}
