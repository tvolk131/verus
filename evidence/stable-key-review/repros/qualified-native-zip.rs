use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8]) {
for (b,h) in core::iter::Iterator::zip(buf.iter_mut(),key) {*b=1;}
}
}
