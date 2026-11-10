#![crate_type = "lib"]

pub fn xor_key(ipad: &[u8; 128], key: &[u8]) {
    for (b_i, b_h) in ipad.iter().zip(key) {
        let _value = *b_i ^ *b_h;
    }
}
