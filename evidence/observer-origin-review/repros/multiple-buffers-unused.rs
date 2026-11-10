use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],other:&mut[u8;10]) {
let _unused=other.iter_mut(); for b in buf.iter_mut() { *b = 1; }
}
}
