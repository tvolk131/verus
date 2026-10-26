use vstd::prelude::*;
#[verus_infer]
fn add(x: usize, n: usize) -> usize { x + n }
verus! { fn caller() { let _ = add(usize::MAX, 1); } }
