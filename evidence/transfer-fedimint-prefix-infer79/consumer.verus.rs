use vstd::prelude::*;
use std::ops::Range;
use prefix_provider::*;
verus! {
#[verifier::infer]
pub fn composed(prefix: &Vec<u8>, first: &[u8], second: &[u8]) -> Vec<u8> {
    let intermediate = key_contents(prefix, first);
    key_contents(&intermediate, second)
}

pub fn exact_composed(prefix: &Vec<u8>, first: &[u8], second: &[u8]) -> (result: Vec<u8>)
    ensures result@ == prefix_view(prefix_view(prefix@, first@), second@),
{
    composed(prefix, first, second)
}

pub fn imported_range(prefix: &Vec<u8>, bounds: Range<&[u8]>) -> (result: Range<Vec<u8>>)
    ensures result.start@ == prefix_view(prefix@, bounds.start@),
        result.end@ == prefix_view(prefix@, bounds.end@),
{
    range_contents(prefix, bounds)
}

pub fn imported_separation(left: &Vec<u8>, right: &Vec<u8>, first: &[u8], second: &[u8])
    -> (result: (Vec<u8>, Vec<u8>))
    requires prefix_incomparable(left@, right@),
    ensures result.0@ != result.1@,
{
    let first_key = key_contents(left, first);
    let second_key = key_contents(right, second);
    proof { disjoint_namespaces(left@, right@, first@, second@); }
    (first_key, second_key)
}
}
