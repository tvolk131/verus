use vstd::prelude::*;
#[verus_infer] pub fn field(bytes:&[u8],start:usize,end:usize)->u8{let field=&bytes[start..end];field[0]}
verus!{fn caller(){let _=field(&[1,2],2,2);}}
