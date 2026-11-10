use vstd::prelude::*;
verus! {
// No precondition is added; frontend must refuse the unsupported primitive.
pub unsafe fn raw_deref(p: *mut u8) -> (r: u8) { unsafe { *p } }
}
