use vstd::prelude::*;
verus!{global size_of usize == 4;}
#[verus_infer(precision="sound")] pub fn flags(x:usize)->usize{(x&15)>>1}
#[verus_infer(precision="sound")] pub fn signed(x:isize)->isize{x&15}
#[verus_infer(precision="sound")] pub fn shifted(x:usize)->usize{if x<16{x>>1}else{0}}
verus!{fn caller(x:usize,y:isize){let r=flags(x);assert(r<=7);let r=signed(y);assert(0<=r && r<=15);let r=shifted(x);assert(r<=7);}}
