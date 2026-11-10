#![crate_type = "lib"]
#[inline(never)]
fn touch(ipad: &mut [u8; 128]) { ipad[127] ^= 1; }
pub fn xor_key(ipad: &mut [u8; 128], key: &[u8]) {
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
    let effect: fn(&mut [u8; 128]) = std::hint::black_box(touch);
    effect(ipad);
}
