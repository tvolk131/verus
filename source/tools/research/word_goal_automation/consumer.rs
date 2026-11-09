use vstd::prelude::*;
verus! {
fn grant(word: u64, index: u64) -> (r: u64)
    requires index < 64,
    ensures provider::rights(r) == provider::rights(word).update(index as int, true),
{
    let original = provider::identity(word);
    original | (1u64 << index)
}
}
fn main() {}
