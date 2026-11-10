use vstd::prelude::*; verus! { fn write(b:&mut u8){*b=42;} fn client(buf:&mut[u8;10]){for b in buf.iter_mut() { *b = 1; } write(&mut buf[0]);} }
