use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8],other:&[u8]) {
let mut k=key; for (b,h) in buf.iter_mut().zip(k) {k=other; *b=1;}
}
}
