use vstd::prelude::*;
verus! {
fn make_key<'a>(key:&'a[u8],counter:&mut u8)->(r:&'a[u8]) ensures r@==key@, *final(counter)==7 { *counter=7; key }
pub fn client(buf:&mut[u8;10],key:&[u8]) {
let mut counter=0u8; for (b,h) in buf.iter_mut().zip(make_key(key,&mut counter)) { *b=1; }
}
}
