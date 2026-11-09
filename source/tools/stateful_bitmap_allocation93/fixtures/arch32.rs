use vstd::prelude::*;
verus! {
global size_of usize == 4;
fn generic_mask(bit:usize)->(result:u64) requires bit==0, ensures result==0, {(1u64<<bit)-1}
}
