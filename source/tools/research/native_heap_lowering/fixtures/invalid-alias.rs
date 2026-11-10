#![crate_type="lib"]
pub fn xor_key(buf:&mut [u8;128]) { let first=&mut *buf; let _it=buf.iter_mut(); first[0]=1; }
