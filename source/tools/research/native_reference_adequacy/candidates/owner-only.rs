use vstd::prelude::*;
verus! {
// Resolving this owned reference does not resolve the separately owned referent.
pub fn owner_only(x: &mut &mut u8) {
    proof { assert(has_resolved(x)); }
}
}
