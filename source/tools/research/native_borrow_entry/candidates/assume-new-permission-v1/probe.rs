use vstd::prelude::*;
use vstd::raw_ptr::PointsTo;
verus! {
pub fn forbidden_entry(buf: &mut [u8;128]) -> Tracked<PointsTo<[u8;128]>> {
    Tracked::assume_new()
}
}
