use vstd::prelude::*;
use vstd::cell::PCell;
verus! {
// Existing permission-model control only: this creates a different PCell.
// It does not manufacture permission for a native Mutex's UnsafeCell field.
pub fn existing_permission_control(value: u64) -> (result: u64)
    ensures result == value,
{
    let (cell, Tracked(permission)) = PCell::new(value);
    *cell.borrow(Tracked(&permission))
}
}
