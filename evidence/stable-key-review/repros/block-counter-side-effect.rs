use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8]) {
let mut counter=0u8; for (b,h) in buf.iter_mut().zip({counter=7; key}) { *b=1; }
}
}
