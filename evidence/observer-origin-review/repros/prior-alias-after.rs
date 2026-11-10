use vstd::prelude::*; verus! { fn write(buf:&mut[u8;10]) {buf[0]=42;} pub fn client(buf:&mut[u8;10]) { for b in buf.iter_mut() { *b = 1; } let alias = &mut *buf; write(alias); } }
