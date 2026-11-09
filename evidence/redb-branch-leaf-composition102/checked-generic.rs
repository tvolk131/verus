use vstd::prelude::*;
use leaf_decoder as leaf;
use branch_routing023 as branch;
use leaf_decoder::Value;
use pagecodec023::PageNumber;

verus! {
pub proof fn meaning_bridge(bytes: Seq<u8>)
    requires bytes.len() == 4,
    ensures <u32 as leaf::Value>::meaning(bytes)
        == <u32 as leaf_lookup023::Value>::meaning(bytes),
{
}

pub open spec fn meaning(bytes: Seq<u8>) -> int {
    <u32 as Value>::meaning(bytes)
}

pub open spec fn interval(bytes: Seq<u8>, width: Option<usize>, child: int, key: Seq<u8>) -> bool {
    &&& 0 <= child <= branch::key_count(bytes)
    &&& (child > 0 ==> meaning(branch::encoded_key(bytes, width, child - 1)) < meaning(key))
    &&& (child < branch::key_count(bytes) ==> meaning(key) <= meaning(branch::encoded_key(bytes, width, child)))
}

pub open spec fn supplied(bytes: Seq<u8>, leaves: Seq<&[u8]>, kw: Option<usize>, vw: Option<usize>) -> bool {
    &&& branch_clients023::ordered_bytes::<u32>(bytes, kw)
    &&& leaves.len() == branch::key_count(bytes) + 1
    &&& forall|child: int| 0 <= child < leaves.len()
        ==> lookup_client::ordered_leaf(#[trigger] leaves[child]@, kw, vw)
    &&& forall|child: int, index: int| 0 <= child < leaves.len()
        && 0 <= index < leaf::pair_count(leaves[child]@)
        ==> interval(bytes, kw, child, #[trigger] leaf::encoded_key(leaves[child]@, kw, vw, index))
}

pub open spec fn union_has(leaves: Seq<&[u8]>, kw: Option<usize>, vw: Option<usize>, query: Seq<u8>) -> bool {
    exists|child: int, index: int| 0 <= child < leaves.len()
        && 0 <= index < leaf::pair_count(leaves[child]@)
        && #[trigger] leaf::encoded_key(leaves[child]@, kw, vw, index) == query
}

pub proof fn matching_child(bytes: Seq<u8>, leaves: Seq<&[u8]>, kw: Option<usize>, vw: Option<usize>, query: Seq<u8>, selected: int, child: int, index: int)
    requires supplied(bytes, leaves, kw, vw), query.len() == 4,
        branch_clients023::routed_bytes::<u32>(bytes, kw, query, selected),
        0 <= child < leaves.len(), 0 <= index < leaf::pair_count(leaves[child]@),
        leaf::encoded_key(leaves[child]@, kw, vw, index) == query,
    ensures child == selected,
{
    meaning_bridge(query);
    assert(interval(bytes, kw, child, leaf::encoded_key(leaves[child]@, kw, vw, index)));
    if child < selected {
        let separator = branch::encoded_key(bytes, kw, child);
        assert(<u32 as leaf_lookup023::Value>::valid(separator));
        meaning_bridge(separator);
        assert(<u32 as leaf_lookup023::Value>::meaning(separator) < <u32 as leaf_lookup023::Value>::meaning(query));
        assert(meaning(query) <= meaning(separator));
    } else if selected < child {
        let separator = branch::encoded_key(bytes, kw, child - 1);
        assert(<u32 as leaf_lookup023::Value>::valid(separator));
        meaning_bridge(separator);
        assert(<u32 as leaf_lookup023::Value>::meaning(separator) >= <u32 as leaf_lookup023::Value>::meaning(query));
        assert(meaning(separator) < meaning(query));
    }
}

pub fn route_then_search<'a>(bytes: &[u8], leaves: &'a [&'a [u8]], kw: Option<usize>, vw: Option<usize>, query: &[u8])
    -> (result: (usize, PageNumber, Option<(&'a [u8], &'a [u8])>))
    requires supplied(bytes@, leaves@, kw, vw), query.len() == 4,
    ensures
        result.0 < leaves.len(), result.0 <= branch::key_count(bytes@),
        result.1 == branch::decoded_child(bytes@, result.0 as int), pagecodec023::canonical_page(result.1),
        interval(bytes@, kw, result.0 as int, query@),
        forall|child: int, index: int| 0 <= child < leaves.len()
            && 0 <= index < leaf::pair_count(leaves@[child]@)
            && #[trigger] leaf::encoded_key(leaves@[child]@, kw, vw, index) == query@
            ==> child == result.0,
        result.2.is_some() == union_has(leaves@, kw, vw, query@),
        result.2.is_some() ==> result.2.unwrap().0@ == query@,
        result.2.is_some() ==> exists|index: int| 0 <= index < leaf::pair_count(leaves@[result.0 as int]@)
            && #[trigger] leaf::encoded_key(leaves@[result.0 as int]@, kw, vw, index) == query@
            && leaf::encoded_value(leaves@[result.0 as int]@, kw, vw, index) == result.2.unwrap().1@,
{
    let (selected, page) = branch_clients023::route_u32(bytes, kw, query);
    proof {
        meaning_bridge(query@);
        if selected > 0 {
            meaning_bridge(branch::encoded_key(bytes@, kw, selected as int - 1));
        }
        if selected < branch::key_count(bytes@) {
            meaning_bridge(branch::encoded_key(bytes@, kw, selected as int));
        }
        assert(interval(bytes@, kw, selected as int, query@));
        assert forall|child: int, index: int| 0 <= child < leaves.len()
            && 0 <= index < leaf::pair_count(leaves@[child]@)
            && #[trigger] leaf::encoded_key(leaves@[child]@, kw, vw, index) == query@
            implies child == selected by {
            matching_child(bytes@, leaves@, kw, vw, query@, selected as int, child, index);
        }
    }
    let hit = lookup_client::search_u32(leaves[selected], kw, vw, query);
    proof {
        if hit.is_some() {
            let index = choose|index: int| 0 <= index < leaf::pair_count(leaves@[selected as int]@)
                && #[trigger] leaf::encoded_key(leaves@[selected as int]@, kw, vw, index) == query@;
            assert(union_has(leaves@, kw, vw, query@));
        } else if union_has(leaves@, kw, vw, query@) {
            let (child, index) = choose|child: int, index: int| 0 <= child < leaves.len()
                && 0 <= index < leaf::pair_count(leaves@[child]@)
                && #[trigger] leaf::encoded_key(leaves@[child]@, kw, vw, index) == query@;
            matching_child(bytes@, leaves@, kw, vw, query@, selected as int, child, index);
            assert(exists|candidate: int| 0 <= candidate < leaf::pair_count(leaves@[selected as int]@)
                && #[trigger] leaf::encoded_key(leaves@[selected as int]@, kw, vw, candidate) == query@);
            assert(false);
        }
    }
    (selected, page, hit)
}
}
