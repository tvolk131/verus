use vstd::prelude::*;
verus! {
// A permission flag word, not allocator state: granting one right preserves
// a distinct right and is idempotent for existing rights.
fn grant(rights: u64, requested: u64, other: u64) -> (r: u64)
    requires requested < 64, other < 64, other != requested,
    ensures
        vstd::word::member(r, requested as int),
        vstd::word::view(r, 64).count_prefix(|b: bool| b, 64) ==
            vstd::word::view(rights, 64).count_prefix(|b: bool| b, 64) +
                if vstd::word::view(rights, 64)[requested as int] { 0nat } else { 1nat },
        vstd::word::member(r, other as int) == vstd::word::member(rights, other as int),
{
    let result = rights | (1u64 << requested);
    result
}
fn already_granted(rights: u64, requested: u64) -> (r: u64)
    requires requested < 64, rights & (1u64 << requested) != 0,
    ensures vstd::word::member(r, requested as int),
{
    let result = rights | (1u64 << requested);
    result
}
}
fn main() {}
