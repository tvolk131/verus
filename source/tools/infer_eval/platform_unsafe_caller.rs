use vstd::prelude::*;
#[verus_infer] fn divide(x: isize, y: isize) -> isize { x / y }
verus! { fn caller() { let _=divide(isize::MIN,-1); } }
