#![crate_type="lib"]
pub fn xor_key(buf:&mut [u8;128]) { { let _it=buf.iter_mut(); } buf[127]=3; }
