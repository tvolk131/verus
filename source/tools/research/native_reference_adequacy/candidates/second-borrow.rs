use vstd::prelude::*;
verus! {
pub proof fn hold(tracked owner: &mut [u8]) { }
pub fn simultaneous_native_entry(buf: &mut [u8;128]) {
    let slice: &mut [u8] = buf;
    let tracked owner = &mut *slice;
    let _it = slice.iter_mut();
    proof { hold(owner); }
}
}
