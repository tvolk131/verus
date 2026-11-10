use vstd::prelude::*;
verus! {

pub fn client(buf: &mut [u8;10]) {
for b in buf.iter_mut() { *b = 1; } let a=&mut *buf; let b=&mut *buf; a[0]=1; b[0]=2;
}
}
