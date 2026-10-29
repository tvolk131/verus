//! Run with: verus verus_infer_range.rs --infer-goals --no-cheating
use vstd::prelude::*;

verus! {
struct Buffer { values: Vec<u8>, stamp: u8 }

// The author chooses the behavior. --infer-goals proposes checked loop facts.
fn fill(buffer: &mut Buffer, start: u16, end: u16)
    requires end < old(buffer).values@.len(),
    ensures
        final(buffer).values@.len() == old(buffer).values@.len(),
        final(buffer).stamp == old(buffer).stamp,
        forall|i: int| start <= i <= end ==>
            #[trigger] final(buffer).values@[i] == 1,
        forall|i: int| 0 <= i < old(buffer).values@.len() && (i < start || i > end) ==>
            #[trigger] final(buffer).values@[i] == old(buffer).values@[i],
{
    for i in start..=end {
        buffer.values[i as usize] = 1;
    }
}
}
fn main() {}
