use vstd::prelude::*;
verus! {
fn write(buf:&mut[u8;10])->(r:usize) ensures r==0 {buf[0]=42; 0}
pub fn client(buf:&mut[u8;10], other:&[u8;10]) {
let _x=other[write(buf)]; for b in buf.iter_mut() { *b = 1; }
}
}
