use vstd::prelude::*;
verus! { pub fn client(buf: &mut [u8;10]) {
let ghost snap = buf@; for b in buf.iter_mut() { *b = 1; }
} }
