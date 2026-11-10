use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8],other:&[u8]) {
for (b,h) in buf.iter_mut().zip({let key=other; key}) { *b=1; }
}
}
