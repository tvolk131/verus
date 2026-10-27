use vstd::prelude::*;
#[verus_infer(precision="sound")] pub fn kind(flags:u8)->u8{flags&15}
verus!{fn caller(){let r=kind(255);assert(r<=14);}}
