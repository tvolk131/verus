use vstd::prelude::*;
verus! {

pub fn client<'a>(buf:&mut[u8;10],mut key:&'a[u8],other:&'a[u8]) {
for (b,h) in buf.iter_mut().zip(key) {let alias=&mut key; *alias=other; *b=1;}
}
}
