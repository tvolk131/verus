use vstd::prelude::*;
verus! {
pub struct Holder<'a> {pub key:&'a[u8]}
pub fn client(buf:&mut[u8;10],holder:Holder) {
for (b,h) in buf.iter_mut().zip(holder.key) { *b=1; }
}
}
