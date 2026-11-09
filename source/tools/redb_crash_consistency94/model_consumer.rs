use vstd::prelude::*;
use persistence94::*;

verus! {

pub proof fn imported_crash_trace_preserves_committed_bytes(
    states: Seq<StorageImage>, steps: Seq<StorageStep>, original: Seq<u8>, protected: Set<int>)
    requires valid_trace(states, steps, protected), valid_storage(states[0]),
        protected_bytes(states[0], original, protected),
    ensures forall|index: int| protected.contains(index) ==>
        #[trigger] states.last().durable[index] == original[index],
{
    trace_preserves_protection(states, steps, original, protected);
}

}
