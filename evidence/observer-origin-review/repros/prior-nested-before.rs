use vstd::prelude::*;
verus! { pub fn client(buf: &mut [u8;10]) {
if true { match 1u8 { 1 => {buf[0]=42;}, _ => {} } } for b in buf.iter_mut() { *b = 1; }
} }
