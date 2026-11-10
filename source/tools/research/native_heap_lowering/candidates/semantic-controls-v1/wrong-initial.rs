use vstd::prelude::*;
#[path="../kernel-v4/kernel.rs"] mod kernel;
use kernel::*;
verus! {
pub open spec fn result(b:Seq<u8>) -> State { initial(b.update(0,0),7,9) }
pub proof fn rejected(b:Seq<u8>) requires b.len()==128,b[0]!=0, ensures result(b).bytes==b, {}
}
