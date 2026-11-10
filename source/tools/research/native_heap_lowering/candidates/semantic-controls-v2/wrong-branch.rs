use vstd::prelude::*;
#[path="../kernel-v7/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State {step(put(initial(b,7,9),2,Val::Num(0)),Op::Select(2,0,1,2,1))}
pub proof fn rejected(b:Seq<u8>) requires b.len()==128, ensures result(b).ok, {}
}
