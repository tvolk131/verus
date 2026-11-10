use vstd::prelude::*;
#[path="../kernel-v7/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> Seq<u8> {run(initial(b,7,9),seq![Op::Unit(0),Op::Return(0)]).bytes}
pub proof fn rejected(b:Seq<u8>) requires b.len()==128,b[0]!=0, ensures result(b)==b.update(0,0), {}
}
