use vstd::prelude::*;
use vstd::std_specs::iter::{IteratorSpec, zip_iter_fst};
verus! {
pub fn iterator_resolves(buf: &mut [u8]) {
    let it = buf.iter_mut();
    proof { assert(has_resolved(it)); }
}
pub fn zip_resolves(buf: &mut [u8], key: &[u8]) {
    let it = buf.iter_mut().zip(key);
    proof { assert(has_resolved(it)); }
}
}
