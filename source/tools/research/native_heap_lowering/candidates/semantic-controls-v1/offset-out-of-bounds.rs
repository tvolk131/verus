use vstd::prelude::*;
#[path="../kernel-v4/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State { let s=put(put(initial(b,7,9),2,Val::Raw{p:Ptr{origin:7,loan:9,offset:0},meta:None}),3,Val::Num(129)); step(s,Op::Offset(4,2,3)) }
pub proof fn rejected(b:Seq<u8>) requires b.len()==128, ensures result(b).ok, {}
}
