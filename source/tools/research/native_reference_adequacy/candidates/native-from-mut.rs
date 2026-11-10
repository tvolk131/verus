use vstd::prelude::*;
verus! {
// Exact constructor operation before cast/end pointer arithmetic.
pub fn native_from_mut(buf: &mut [u8;128]) {
    let slice: &mut [u8] = buf;
    let _p = core::ptr::NonNull::from_mut(slice);
}
}
