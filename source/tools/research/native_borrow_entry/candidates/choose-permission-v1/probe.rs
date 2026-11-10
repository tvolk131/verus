use vstd::prelude::*;
use vstd::raw_ptr::PointsTo;
verus! {
// Sole input: an ordinary executable exclusive initialized array borrow.
// choose can name a logical permission, but cannot produce a tracked resource.
pub fn enter(buf: &mut [u8; 128]) -> (out: Tracked<PointsTo<[u8; 128]>>) {
    let tracked p = choose|p: PointsTo<[u8; 128]>| p.is_init() && p.value() == *buf;
    Tracked(p)
}
}
