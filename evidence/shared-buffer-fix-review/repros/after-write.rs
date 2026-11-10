use vstd::prelude::*;
verus! {
    pub fn after_write(buf: &mut [u8;10]) {
        for b in buf.iter_mut() { *b = 1; }
        buf[0] = 42;
    }
}
