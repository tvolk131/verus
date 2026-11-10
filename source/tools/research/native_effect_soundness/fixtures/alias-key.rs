#![crate_type="lib"]
pub fn xor_key(ipad:&mut [u8;128],_key:&[u8]) {
    let key=unsafe { std::slice::from_raw_parts(ipad.as_ptr(),128) };
    for (b_i,b_h) in ipad.iter_mut().zip(key) { *b_i ^= *b_h; }
}
