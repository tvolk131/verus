use vstd::prelude::*;
verus! {

pub fn client<'a>(buf:&mut[u8;10],key:&mut &'a[u8],other:&'a[u8]) {
for (b,h) in buf.iter_mut().zip(*key) {*key=other; *b=1;}
}
}
