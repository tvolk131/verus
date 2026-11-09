use vstd::prelude::*;
verus! {
// Audit only: these are the original authored representation definitions.
// This theorem is not called by either accepted automation fixture.
spec fn original_member(word: u64, j: int) -> bool {
    (word & (1u64 << (j as u64))) != 0
}
spec fn original_allocated(word: u64, len: u32) -> Seq<bool> {
    Seq::new(len as nat, |j: int| original_member(word, j))
}
proof fn representations_agree(word: u64, len: u32)
    ensures
        forall|j: int| original_member(word, j) == vstd::word::member(word, j),
        original_allocated(word, len) == vstd::word::view(word, len as nat),
{
    assert(original_allocated(word, len) =~= vstd::word::view(word, len as nat));
}
}
fn main() {}
