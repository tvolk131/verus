use vstd::prelude::*;
#[path="../kernel-v4/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State { let s=initial(b,7,9); put(s,1,Val::Ref{p:Ptr{origin:8,loan:9,offset:0},len:128,slice:false}) }
pub proof fn rejected(b:Seq<u8>) requires b.len()==128, ensures wf(result(b)), {}
}
