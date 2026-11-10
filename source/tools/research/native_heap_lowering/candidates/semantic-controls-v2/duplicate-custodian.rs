use vstd::prelude::*;
#[path="../kernel-v7/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State {let s=put(initial(b,7,9),101,get(initial(b,7,9),1));let t=step(s,Op::Enter(1,101));step(t,Op::Enter(1,101))}
pub proof fn rejected(b:Seq<u8>) requires b.len()==128, ensures result(b).ok, {}
}
