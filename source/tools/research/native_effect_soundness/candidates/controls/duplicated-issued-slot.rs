use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(tracked mut w:World) -> (tracked out:World)
 requires w.wf(), w.c.physical.index+1 < w.c.physical.length,
 ensures out.c.physical.index == w.c.physical.index+1,
{ world_issue(&mut w); world_issue(&mut w); w }
}
