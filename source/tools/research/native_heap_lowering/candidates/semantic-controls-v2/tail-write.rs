use vstd::prelude::*;
#[path="../kernel-v7/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State { let s=put(initial(b,7,9),2,Val::Raw{p:Ptr{origin:7,loan:9,offset:127},meta:None}); step(s,Op::Store(2,0)) }
pub proof fn rejected(b:Seq<u8>) requires b.len()==128,b[127]!=0, ensures result(b).bytes==b, {}
}
