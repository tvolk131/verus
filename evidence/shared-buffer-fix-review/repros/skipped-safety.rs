use vstd::prelude::*;
verus! { pub fn client(buf: &mut [u8;10]) {
buf[0]=42; for b in buf.iter_mut(){*b+=1;}
} }
