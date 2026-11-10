use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(e:Entry,p:Physical)
 requires e.valid(), physical_ok(p,e), 0 < p.index < p.length,
 ensures projection(e,p)[0].0 == e.pad[0],
{}
}
