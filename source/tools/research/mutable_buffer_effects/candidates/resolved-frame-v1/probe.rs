use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
// Diagnostic only: deliberately checks the missing law, without assuming it.
pub proof fn remaining_resolved<'a, T>(it: core::slice::IterMut<'a, T>)
    requires has_resolved(it),
    ensures forall|j: int| 0 <= j < it.remaining().len() ==>
        *final(it.remaining()[j]) == *it.remaining()[j],
{
}
}
