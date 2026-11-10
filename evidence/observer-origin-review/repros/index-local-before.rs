use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10], other:&[u8;10]) {
let _x=other[{buf[0]=42; 0}]; for b in buf.iter_mut() { *b = 1; }
}
}
