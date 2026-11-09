extern crate bounded92;
use vstd::prelude::*;
use bounded92::*;

verus! {
pub fn fresh_allocation(tree: &mut BtreeBitmap, query: u32) -> (result: Option<u32>)
    requires old(tree).invariant(), query < old(tree).levels().last().model_len(),
    ensures final(tree).invariant(),
        result.is_none() <==> prefix_full(old(tree).levels().last(), old(tree).levels().last().model_len() as int),
        result.is_none() ==> *final(tree) == *old(tree),
        result.is_some() ==> result.unwrap() < old(tree).levels().last().model_len(),
        result.is_some() ==> !bit(old(tree).levels().last().words(), result.unwrap() as int),
        result.is_some() ==> prefix_full(old(tree).levels().last(), result.unwrap() as int),
        result.is_some() ==> set_effect(old(tree).levels(), final(tree).levels(), result.unwrap() as int),
        result.is_some() ==> bit(final(tree).levels().last().words(), query as int) ==
            (query == result.unwrap() || bit(old(tree).levels().last().words(), query as int)),
{
    let ghost before = tree.levels();
    let result = tree.alloc();
    proof {
        if let Some(selected) = result {
            assert(shape(before));
            assert(before[before.len() as int - 1].covered() && before[before.len() as int - 1].padded());
            assert(group_delta(before.last(), tree.levels().last(), selected as int, true));
            group_edit(before.last(), tree.levels().last(), selected as int, true);
        }
    }
    result
}

pub fn fresh_search(tree: &BtreeBitmap) -> (result: Option<u32>)
    requires tree.invariant(),
    ensures result.is_some() ==> result.unwrap() < tree.levels().last().model_len(),
        result.is_some() ==> !bit(tree.levels().last().words(), result.unwrap() as int),
        result.is_some() ==> prefix_full(tree.levels().last(), result.unwrap() as int),
        result.is_none() <==> prefix_full(tree.levels().last(), tree.levels().last().model_len() as int),
{
    tree.find_first_unset()
}

pub fn fresh_allocation_zero_length(tree: &mut BtreeBitmap) -> (result: Option<u32>)
    requires old(tree).invariant(), old(tree).levels().last().model_len() == 0,
    ensures result.is_none(), *final(tree) == *old(tree), final(tree).invariant(),
{
    tree.alloc()
}
}
