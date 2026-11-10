use vstd::prelude::*;
verus! {
pub fn simultaneous_native_entry(buf: &mut [u8;128]) {
    let tracked owner = &mut *buf;
    let _it = buf.iter_mut();
    proof { let tracked slot = &mut owner[127]; assert(*slot <= 255); }
}
}
