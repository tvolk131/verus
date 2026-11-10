use vstd::prelude::*;
verus! {
proof fn checked_identity() { assert(1int==1int); }
pub fn client(buf:&mut[u8;10],mut key:&[u8]) {
for (b,h) in buf.iter_mut().zip(key) { *b=1; }
}
}
