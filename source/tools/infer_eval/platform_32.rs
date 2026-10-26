use vstd::prelude::*;
verus! { global size_of usize == 4; }
#[verus_infer] fn cast(x: i128) -> isize { x as isize }
#[verus_infer] fn checked(x: isize, y: isize) -> Option<isize> { x.checked_add(y) }
#[verus_infer(precision="sound")] fn scale(x: isize) -> isize { x / 10 }
verus! { fn caller() {
    assert((4294967297i128 as isize) == 1) by(bit_vector);
    let r=cast(4294967297); assert(r==1);
    let r=checked(isize::MAX,1); assert(r==None);
    let r=scale(isize::MIN); assert(-214748364<=r && r<=214748364);
} }
