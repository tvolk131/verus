use vstd::prelude::*;
verus! {

pub fn client(buf: &mut [u8;10]) {
let VERUS_iter_init=1u8; let VERUS_iter_exec=2u8; let VERUS_it=3u8; for b in buf.iter_mut() { *b = 1; } let _x=VERUS_iter_init+VERUS_iter_exec+VERUS_it;
}
}
