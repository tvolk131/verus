use vstd::prelude::*;
verus! {

pub fn client(buf: &mut [u8;10]) {
for b in {buf[0]=42; buf.iter_mut()} {*b=1;}
}
}
