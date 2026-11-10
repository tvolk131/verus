#![crate_type="lib"]
struct D<'a>(&'a mut [u8;128]);
impl Drop for D<'_> { fn drop(&mut self) { self.0[127]=3; } }
pub fn xor_key(buf:&mut [u8;128]) { { let _it=buf.iter_mut(); } let _d=D(buf); }
