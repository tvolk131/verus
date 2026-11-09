extern crate bounded92;
use vstd::prelude::*;
use bounded92::*;

verus! {
pub fn hierarchy_positive(tree: &mut BtreeBitmap) -> (result: Option<u32>)
    requires old(tree).invariant(), old(tree).levels().len() == 2,
        old(tree).levels().last().model_len() == 2,
        !bit(old(tree).levels().last().words(), 0), !bit(old(tree).levels().last().words(), 1),
    ensures result == Some(0u32), !bit(final(tree).levels()[0].words(), 0),
{
    let ghost before = tree.levels();
    let result = tree.alloc();
    proof {
        assert(result.is_some());
        assert(result.unwrap() == 0);
        assert(shape(before));
        assert(before[1].covered() && before[1].padded());
        assert(group_delta(before.last(), tree.levels().last(), 0, true));
        group_edit(before.last(), tree.levels().last(), 0, true);
        assert(!bit(tree.levels().last().words(), 1));
        if tree.levels().last().words()[0] == u64::MAX { full_word(tree.levels().last().words(), 1); }
        assert(!bit(tree.levels()[0].words(), 0));
    }
    result
}

pub fn physical_and_ancestor_frame(tree: &mut BtreeBitmap, query: Ghost<int>, height: u32, word: usize) -> (result: Option<u32>)
    requires old(tree).invariant(),
        0 <= query@ < 64 * old(tree).levels().last().words().len(),
        height < old(tree).levels().len() - 1,
        word < old(tree).levels()[height as int].words().len(),
    ensures final(tree).invariant(),
        result.is_none() ==> *final(tree) == *old(tree),
        result.is_some() ==> bit(final(tree).levels().last().words(), query@) ==
            (query@ == result.unwrap() || bit(old(tree).levels().last().words(), query@)),
        result.is_some() && word != path(result.unwrap() as int, old(tree).levels().len() as int, height as int) / 64 ==>
            final(tree).levels()[height as int].words()[word as int] == old(tree).levels()[height as int].words()[word as int],
        forall|level: int| 0 <= level < old(tree).levels().len() ==>
            #[trigger] final(tree).levels()[level].model_len() == old(tree).levels()[level].model_len()
                && final(tree).levels()[level].words().len() == old(tree).levels()[level].words().len(),
{
    let ghost before = tree.levels();
    let result = tree.alloc();
    proof {
        if let Some(selected) = result {
            assert(shape(before));
            assert(before[before.len() as int - 1].covered() && before[before.len() as int - 1].padded());
            group_edit(before.last(), tree.levels().last(), selected as int, true);
            assert forall|level: int| 0 <= level < before.len()
                implies #[trigger] tree.levels()[level].model_len() == before[level].model_len()
                    && tree.levels()[level].words().len() == before[level].words().len() by {
                if level < before.len() - 1 { path_valid(before, selected as int, level); }
            }
        }
    }
    result
}

pub fn valid_padded_first(group: &U64GroupedBitmap) -> (result: Option<u32>)
    requires group.covered(), group.padded(), 0 < group.model_len() <= 64,
    ensures result.is_some() ==> result.unwrap() < group.model_len(),
{
    let result = group.first_unset(0, 64);
    proof { if let Some(selected) = result { selected_valid(*group, 0, selected as int); } }
    result
}
}
