use vstd::prelude::*;
verus! {
fn make_key(key:&[u8])->(r:&[u8]) ensures r@==key@ {key}
pub fn client(buf:&mut[u8;10],key:&[u8]) {
for (b,h) in buf.iter_mut().zip(make_key(key)) {*b=1;}
}
}
