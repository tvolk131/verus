use vstd::prelude::*;
verus! { global size_of usize == 8; }
#[verus_infer] fn cast(x: i128) -> isize { x as isize }
#[verus_infer] fn checked(x: isize, y: isize) -> Option<isize> { x.checked_add(y) }
#[verus_infer(precision="sound")] fn scale(x: isize) -> isize { x / 10 }
verus! { fn caller() {
    let r=cast(4294967297); assert(r==4294967297);
    let r=checked(isize::MAX,1); assert(r==None);
    let r=scale(isize::MIN); assert(-922337203685477580<=r && r<=922337203685477580);
} }
