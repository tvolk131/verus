use vstd::prelude::*;
mod word_laws;
verus! {
// A permission flag word, not allocator state: granting one right preserves
// a distinct right and is idempotent for existing rights.
fn grant(rights: u64, requested: u64, other: u64) -> (r: u64)
    requires requested < 64, other < 64, other != requested,
    ensures
        (r & (1u64 << requested)) != 0,
        word_laws::word_view(r, 64).count_prefix(|b: bool| b, 64) ==
            word_laws::word_view(rights, 64).count_prefix(|b: bool| b, 64) +
                if word_laws::word_view(rights, 64)[requested as int] { 0nat } else { 1nat },
        ((r & (1u64 << other)) != 0) == ((rights & (1u64 << other)) != 0),
{
    let result = rights | (1u64 << requested);
    proof { word_laws::singleton_view_update(rights, requested, 64); word_laws::or_singleton_membership(rights, requested); }
    result
}
fn already_granted(rights: u64, requested: u64) -> (r: u64)
    requires requested < 64, rights & (1u64 << requested) != 0,
    ensures r & (1u64 << requested) != 0,
{
    let result = rights | (1u64 << requested);
    proof { word_laws::singleton_view_update(rights, requested, 64); word_laws::or_singleton_membership(rights, requested); }
    result
}
}
fn main() {}
