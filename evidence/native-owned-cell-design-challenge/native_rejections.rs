//! Each cfg is a separate ordinary Rust safety/privacy rejection, not an admission test.
use std::cell::UnsafeCell;
#[cfg(sync)]
fn check_sync() { fn requires_sync<T: Sync>() {} requires_sync::<UnsafeCell<u64>>(); }
#[cfg(overlap)]
fn overlapping(cell: &mut UnsafeCell<u64>) { let a = cell.get_mut(); let b = cell.get_mut(); *a = *b; }
#[cfg(escape)]
fn escaping<'a>() -> &'a mut u64 { let mut cell = UnsafeCell::new(0); cell.get_mut() }
#[cfg(private)]
fn private_field(cell: UnsafeCell<u64>) -> u64 { cell.value }
fn main() {}
