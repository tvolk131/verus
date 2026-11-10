use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8],other:&[u8],choose:bool) {
for (b,h) in buf.iter_mut().zip(if choose {key} else {other}) {*b=1;}
}
}
