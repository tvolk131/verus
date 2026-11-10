use vstd::prelude::*;
verus! {

pub fn client(buf: &mut [u8;10]) {
let _x=buf[0]; for b in buf.iter_mut() { *b = 1; }
}
}
