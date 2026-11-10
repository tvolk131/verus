use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],key:&[u8]) {
for (b,h) in <[u8]>::iter_mut(buf).zip(key) {*b=1;}
}
}
