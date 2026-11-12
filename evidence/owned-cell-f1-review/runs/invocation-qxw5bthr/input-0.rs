use vstd::prelude::*;
use core::cell::UnsafeCell;
verus! {
pub fn owned_roundtrip(value: u64) -> (result: u64)
    ensures result == value,
{
    UnsafeCell::new(value).into_inner()
}

pub fn exclusive_update(initial: u64, next: u64, frame: u64) -> (result: (u64, u64))
    ensures result == (next, frame),
{
    let mut cell = UnsafeCell::new(initial);
    *cell.get_mut() = next;
    (cell.into_inner(), frame)
}

pub fn exclusive_borrow(cell: &mut UnsafeCell<u64>, next: u64) -> (result: u64)
    ensures result == next,
{
    *cell.get_mut() = next;
    *cell.get_mut()
}
}
