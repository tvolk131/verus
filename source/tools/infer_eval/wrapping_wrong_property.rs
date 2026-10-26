use vstd::prelude::*;
#[verus_infer] fn total(x:u8,y:u8)->u8{x.wrapping_add(y)}
verus!{fn caller(){let r=total(255,1);assert(r==255);}}
