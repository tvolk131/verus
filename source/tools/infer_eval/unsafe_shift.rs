use vstd::prelude::*;
#[verus_infer] fn shifted(x:u32,n:i32)->u32{x<<n}
verus!{fn caller(){let _=shifted(0,32);}}
