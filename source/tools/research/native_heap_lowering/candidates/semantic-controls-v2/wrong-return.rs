use vstd::prelude::*;
#[path="../kernel-v7/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State {step(initial(b,7,9),Op::Return(1))}
pub proof fn rejected(b:Seq<u8>) requires b.len()==128, ensures result(b).ok, {}
}
