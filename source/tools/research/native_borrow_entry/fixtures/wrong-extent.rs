#![crate_type = "lib"]

pub fn xor_key(ipad: &mut [u8; 127], key: &[u8]) {
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
}
