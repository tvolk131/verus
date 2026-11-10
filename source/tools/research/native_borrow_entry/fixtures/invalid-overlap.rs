#![crate_type = "lib"]

pub fn xor_key(ipad: &mut [u8; 128], key: &[u8]) {
    let alias = &mut *ipad;
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
    alias[127] = 1;
}
