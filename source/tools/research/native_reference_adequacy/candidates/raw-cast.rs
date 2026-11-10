use vstd::prelude::*;
verus! {
// The first reference-to-raw operation, before any Offset or dereference.
pub fn raw_cast(buf: &mut [u8;128]) {
    let _p = buf as *mut [u8;128];
}
}
