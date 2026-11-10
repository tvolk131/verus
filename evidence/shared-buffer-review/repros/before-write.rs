use vstd::prelude::*;
verus! {
    pub fn before_write(buf: &mut [u8;10]) {
        buf[0] = 42;
        for b in buf.iter_mut() { *b = 1; }
    }
}
