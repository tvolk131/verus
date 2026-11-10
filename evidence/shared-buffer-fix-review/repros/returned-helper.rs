use vstd::prelude::*; verus! { fn id(b:&mut[u8;10])->&mut[u8;10]{b} fn client(buf:&mut[u8;10]){for b in buf.iter_mut() { *b = 1; } id(buf)[0]=42;} }
