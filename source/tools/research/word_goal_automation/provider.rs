use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn identity(x: u64) -> u64 { x }
pub open spec fn rights(word: u64) -> Seq<bool> { vstd::word::view(word, 64) }
}
