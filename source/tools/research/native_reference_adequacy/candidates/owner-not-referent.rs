use vstd::prelude::*;
verus! {
// Intentionally false generic strengthening, not a native counterexample.
pub fn owner_not_referent(x: &mut &mut u8) {
    proof {
        assert(has_resolved(x));
        assert(has_resolved(*x));
    }
}
}
