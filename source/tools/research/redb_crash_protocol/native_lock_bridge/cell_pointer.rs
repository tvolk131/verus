use vstd::prelude::*;
verus! {
pub fn native_cell_pointer(cell: &std::cell::UnsafeCell<u64>) -> *mut u64 {
    cell.get()
}
}
