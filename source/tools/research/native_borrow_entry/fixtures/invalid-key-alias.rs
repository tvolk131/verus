#![crate_type = "lib"]

pub fn xor_key(ipad: &mut [u8; 128], key: &[u8]) {
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
}

pub fn invalid_key_alias(ipad: &mut [u8;128]) {
    let key: &[u8] = &ipad[..];
    xor_key(ipad,key);
}
