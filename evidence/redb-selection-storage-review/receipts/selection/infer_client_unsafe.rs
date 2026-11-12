extern crate selection_checked;
use selection_checked::*;
use vstd::prelude::*;
verus! {
pub fn imported_selection(h: &mut UnrepairedDatabaseHeader) -> Result<bool>
{ h.select_primary_slot() }
}
