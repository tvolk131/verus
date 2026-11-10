#![crate_type="lib"]
pub fn xor_key(buf:&mut [u8;128]) { let _it=buf.iter_mut(); }
pub struct Pair {pub pad:[u8;128], pub key:[u8;256]}
pub fn caller(pair:&mut Pair) -> u8 { let key=&pair.key[..]; xor_key(&mut pair.pad); key[255] }
