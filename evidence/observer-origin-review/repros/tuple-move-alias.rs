use vstd::prelude::*;
verus! {

pub fn client(buf: &mut [u8;10]) {
for b in buf.iter_mut() { *b = 1; } let (alias,) = (buf,); alias[0]=42;
}
}
