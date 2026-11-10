use vstd::prelude::*;
verus! {

pub fn client(buf: &mut [u8;10]) {
for b in buf.iter_mut() { *b = 1; } match &mut *buf { alias => {alias[0]=42;} }
}
}
