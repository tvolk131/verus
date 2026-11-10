#![crate_type="lib"]
#![allow(invalid_reference_casting)]
pub fn xor_key(ipad:&mut [u8;128],key:&[u8]) {
    if !key.is_empty() { unsafe { *(key.as_ptr() as *mut u8) = 0; } }
    for (b_i,b_h) in ipad.iter_mut().zip(key) { *b_i ^= *b_h; }
}
