use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub fn bad(p:*mut u8,Ghost(i):Ghost<int>,Tracked(c):Tracked<&mut Carrier>,b:u8)
 requires old(c).wf(), old(c).physical.index <= i < 128,
  p == old(c).entry.pad[i], b != old(c).entry.before[i],
 ensures final(c).wf(),
{ let tracked t=c.retained.tracked_borrow_mut(i); let r=ptr_mut_ref(p,Tracked(t)); *r=b; }
}
