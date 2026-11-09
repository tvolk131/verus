use vstd::prelude::*;
use leaf_decoder::{self as leaf, LeafAccessor, Value};
use lookup_client::{canonical_u32, ordered_leaf, search_u32};

verus! {

pub fn generic_partition_bytes(page: &[u8], key_width: Option<usize>, value_width: Option<usize>, query: &[u8])
    requires ordered_leaf(page@, key_width, value_width), query.len() == 4,
{
    let accessor = LeafAccessor::new(page, key_width, value_width);
    proof { assert(accessor.sorted_for::<u32>()); }
    let (index, present) = accessor.position::<u32>(query);
    assert(0 <= index <= leaf::pair_count(page@));
    proof {
        assert forall|before: int| 0 <= before < index
            implies <u32 as Value>::meaning(#[trigger] leaf::encoded_key(page@, key_width, value_width, before))
                < <u32 as Value>::meaning(query@) by {
            assert(<u32 as Value>::meaning(accessor.key_bytes_at(before))
                < <u32 as Value>::meaning(query@));
        }
        assert forall|after: int| index as int + (if present { 1int } else { 0int }) <= after < leaf::pair_count(page@)
            implies <u32 as Value>::meaning(#[trigger] leaf::encoded_key(page@, key_width, value_width, after))
                > <u32 as Value>::meaning(query@) by {
            assert(<u32 as Value>::meaning(accessor.key_bytes_at(after))
                > <u32 as Value>::meaning(query@));
        }
    }
    assert(forall|before: int| 0 <= before < index
        ==> <u32 as Value>::meaning(#[trigger] leaf::encoded_key(page@, key_width, value_width, before))
            < <u32 as Value>::meaning(query@));
    assert(forall|after: int| index as int + (if present { 1int } else { 0int }) <= after < leaf::pair_count(page@)
        ==> <u32 as Value>::meaning(#[trigger] leaf::encoded_key(page@, key_width, value_width, after))
            > <u32 as Value>::meaning(query@));
    if present {
        proof { canonical_u32(accessor.key_bytes_at(index as int), query@); }
        assert(leaf::encoded_key(page@, key_width, value_width, index as int) == query@);
    }
    let result = search_u32(page, key_width, value_width, query);
    assert(result.is_some() == exists|candidate: int| 0 <= candidate < leaf::pair_count(page@)
        && #[trigger] leaf::encoded_key(page@, key_width, value_width, candidate) == query@);
    if result.is_some() {
        assert(result.unwrap().0@ == query@);
        assert(exists|candidate: int| 0 <= candidate < leaf::pair_count(page@)
            && #[trigger] leaf::encoded_key(page@, key_width, value_width, candidate) == query@
            && leaf::encoded_value(page@, key_width, value_width, candidate) == result.unwrap().1@);
    }
}

pub fn midpoint_general(lower: usize, upper: usize) {
    let forward = lower.midpoint(upper);
    let reverse = upper.midpoint(lower);
    assert(forward == reverse);
    assert(forward as int == (lower as int + upper as int) / 2);
    if lower < upper {
        assert(lower <= forward < upper);
    } else if upper < lower {
        assert(upper <= forward < lower);
    } else {
        assert(forward == lower);
    }
    let extreme = usize::MAX.midpoint(usize::MAX);
    assert(extreme == usize::MAX);
}

pub fn raw_singleton_spare_tail(value0: u8, value1: u8, tail0: u8, tail1: u8, header: u8) {
    let page: &[u8] = &[1, header, 1, 0, 14, 0, 0, 0, 0, 1, 0, 0, value0, value1, tail0, tail1];
    let query: &[u8] = &[0, 1, 0, 0];
    let missing: &[u8] = &[1, 1, 0, 0];
    proof {
        leaf::concrete_le16(page@, 2);
        leaf::concrete_le32(page@, 4);
        assert(leaf::pair_count(page@) == 1);
        assert(leaf::encoded_key(page@, Some(4), None, 0) == query@);
        assert(leaf::encoded_value(page@, Some(4), None, 0) == seq![value0, value1]);
        assert(ordered_leaf(page@, Some(4), None));
    }
    let present = search_u32(page, Some(4), None, query);
    assert(exists|candidate: int| 0 <= candidate < leaf::pair_count(page@)
        && #[trigger] leaf::encoded_key(page@, Some(4), None, candidate) == query@);
    assert(present.is_some());
    assert(present.unwrap().0@ == query@);
    assert(present.unwrap().1@ == seq![value0, value1]);
    let absent = search_u32(page, Some(4), None, missing);
    assert(absent.is_none());
}

}
