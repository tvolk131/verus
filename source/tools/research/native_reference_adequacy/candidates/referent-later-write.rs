use vstd::prelude::*;
verus! {
pub fn owner_only(x: &mut &mut u8) {
    proof { assert(has_resolved(x)); }
}
// Safe, checked caller: the inner reference is used after x resolves.
pub fn caller() {
    let mut byte = 0u8;
    let mut r = &mut byte;
    owner_only(&mut r);
    *r = 1;
    assert(byte == 1);
}
}
