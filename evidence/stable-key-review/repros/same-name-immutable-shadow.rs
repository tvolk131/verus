use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8]) {
let key=key; for (b,h) in buf.iter_mut().zip(key) { *b=1; }
}
}
