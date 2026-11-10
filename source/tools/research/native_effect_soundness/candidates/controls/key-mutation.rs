use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub fn bad(p:*mut u8,Ghost(e):Ghost<Entry>,Ghost(i):Ghost<int>,
 Tracked(keys):Tracked<&mut Map<int,PointsTo<u8>>>,b:u8)
 requires e.valid(),key_owner(*old(keys),e),0 <= i < e.count(),p==e.reads[i],b!=e.key[i],
 ensures key_owner(*final(keys),e),
{ let tracked t=keys.tracked_borrow_mut(i); let r=ptr_mut_ref(p,Tracked(t)); *r=b; }
}
