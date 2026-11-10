use vstd::prelude::*;
#[path="../kernel-v4/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State { let s=put(initial(b,7,9),101,Val::Num(128)); step(s,Op::Enter(1,101)) }
pub proof fn rejected(b:Seq<u8>) requires b.len()==128, ensures result(b).ok, {}
}
