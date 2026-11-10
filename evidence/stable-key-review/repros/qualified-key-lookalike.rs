use vstd::prelude::*;
verus! {
pub mod keys {use vstd::prelude::*; verus! {pub fn zip(key:&[u8])->(r:&[u8]) ensures r@==key@ {key}}}
pub fn client(buf:&mut[u8;10],key:&[u8]) {
for (b,h) in buf.iter_mut().zip(keys::zip(key)) { *b=1; }
}
}
