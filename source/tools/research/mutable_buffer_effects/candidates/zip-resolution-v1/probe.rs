use vstd::prelude::*;
use vstd::std_specs::iter::{IteratorSpec, zip_iter_fst};
verus! {
// Diagnostic only: deliberately checks the missing law, without assuming it.
pub proof fn zip_inner_resolved<'a, T>(
    it: core::iter::Zip<core::slice::IterMut<'a, T>, core::slice::Iter<'a, T>>,
)
    requires has_resolved(it),
    ensures has_resolved(zip_iter_fst(it)),
{
}
}
