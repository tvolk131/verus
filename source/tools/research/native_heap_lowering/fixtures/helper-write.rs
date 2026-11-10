#![crate_type="lib"]
fn helper(buf:&mut [u8;128]) { buf[127]=3; }
pub fn xor_key(buf:&mut [u8;128]) { { let _it=buf.iter_mut(); } helper(buf); }
