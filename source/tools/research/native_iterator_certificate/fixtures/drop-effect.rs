#![crate_type = "lib"]
struct Tail<'a>(&'a mut u8);
impl Drop for Tail<'_> {
    fn drop(&mut self) { *self.0 ^= 1; }
}
pub fn xor_key(ipad: &mut [u8; 128], key: &[u8]) {
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
    let _tail = Tail(&mut ipad[127]);
}
