use vstd::prelude::*;
verus! {

pub fn client(buf:&mut[u8;10],other:&mut[u8;10]) {
for b in buf.iter_mut() { *b = 1; } for b in other.iter_mut(){*b=2;}
}
}
