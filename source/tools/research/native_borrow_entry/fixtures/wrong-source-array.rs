#![crate_type = "lib"]

pub fn xor_key(ipad: &mut [u8; 128], key: &[u8]) {
    let mut other = [0u8;128];
    for (b_i, b_h) in other.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
}
