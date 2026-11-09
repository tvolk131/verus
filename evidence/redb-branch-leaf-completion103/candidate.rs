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

pub proof fn decoded_four(bytes: Seq<u8>)
    requires bytes.len() == 4,
    ensures meaning(bytes) == bytes[0] as int + 256 * bytes[1] as int + 65536 * bytes[2] as int + 16777216 * bytes[3] as int,
{
    leaf::concrete_le32(bytes, 0);
    assert(bytes.subrange(0, 4) =~= bytes);
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

pub open spec fn branch_image(variable: bool) -> Seq<u8> {
    if variable { seq![2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0] } else { seq![2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0] }
}

pub open spec fn leaf_image(child: int, variable: bool, first: u8, second: u8) -> Seq<u8> {
    if variable {
        if child == 0 { seq![1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second] } else if child == 1 { seq![1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, first, second] } else { seq![1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, first, second] }
    } else {
        if child == 0 { seq![1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second] } else if child == 1 { seq![1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, first, second] } else { seq![1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, first, second] }
    }
}

pub open spec fn fixture_kw(variable: bool) -> Option<usize> { if variable { None } else { Some(4usize) } }
pub open spec fn fixture_vw(variable: bool) -> Option<usize> { if variable { None } else { Some(1usize) } }

pub open spec fn fixture_key(child: int, index: int) -> Seq<u8> {
    if child == 0 { if index == 0 { seq![1u8, 0, 0, 0] } else { seq![0u8, 1, 0, 0] } }
    else if child == 1 { if index == 0 { seq![1u8, 1, 0, 0] } else { seq![0u8, 0, 1, 0] } }
    else { if index == 0 { seq![1u8, 0, 1, 0] } else { seq![255u8, 255, 255, 255] } }
}

pub proof fn fixture_leaf_fixed_0(page: Seq<u8>, first: u8, second: u8)
    requires 0 <= 0 < 3, page == leaf_image(0, false, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(false), fixture_vw(false)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(false), fixture_vw(false), index) == fixture_key(0, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(false), fixture_vw(false), index) == seq![if index == 0 { first } else { second }],
{
    let kw = fixture_kw(false);
    let vw = fixture_vw(false);
    leaf::concrete_le16(page, 2);
    if false {
        leaf::concrete_le32(page, 4);
        leaf::concrete_le32(page, 8);
        leaf::concrete_le32(page, 12);
        leaf::concrete_le32(page, 16);
    }
    assert forall|boundary: int| 0 <= boundary <= 2 implies {
        &&& leaf::metadata_end(page, kw, vw) <= #[trigger] leaf::key_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::key_boundary(page, kw, vw, boundary) <= u32::MAX
        &&& leaf::key_boundary(page, kw, vw, 2) <= #[trigger] leaf::value_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::value_boundary(page, kw, vw, boundary) <= u32::MAX
    } by { assert(boundary == 0 || boundary == 1 || boundary == 2); }
    assert forall|left: int, right: int| 0 <= left <= right <= 2 implies {
        &&& #[trigger] leaf::key_boundary(page, kw, vw, left) <= #[trigger] leaf::key_boundary(page, kw, vw, right)
        &&& #[trigger] leaf::value_boundary(page, kw, vw, left) <= #[trigger] leaf::value_boundary(page, kw, vw, right)
    } by {
        assert(left == 0 || left == 1 || left == 2);
        assert(right == 0 || right == 1 || right == 2);
    }
    assert(leaf::leaf_bytes_wf(page, kw, vw));
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_key(page, kw, vw, index) == fixture_key(0, index) by {
        assert(index == 0 || index == 1);
        assert(0 == 0 || 0 == 1 || 0 == 2);
        assert(leaf::encoded_key(page, kw, vw, index) =~= fixture_key(0, index));
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_value(page, kw, vw, index) == seq![if index == 0 { first } else { second }] by {
        assert(index == 0 || index == 1);
        assert(leaf::encoded_value(page, kw, vw, index) =~= seq![if index == 0 { first } else { second }]);
    }
    decoded_four(fixture_key(0, 0));
    decoded_four(fixture_key(0, 1));
    assert forall|left: int, right: int| 0 <= left < right < 2 implies meaning(#[trigger] leaf::encoded_key(page, kw, vw, left)) < meaning(#[trigger] leaf::encoded_key(page, kw, vw, right)) by {
        assert(left == 0 && right == 1);
    }
    assert(lookup_client::ordered_leaf(page, kw, vw));
}

pub proof fn fixture_leaf_fixed_1(page: Seq<u8>, first: u8, second: u8)
    requires 0 <= 1 < 3, page == leaf_image(1, false, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(false), fixture_vw(false)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(false), fixture_vw(false), index) == fixture_key(1, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(false), fixture_vw(false), index) == seq![if index == 0 { first } else { second }],
{
    let kw = fixture_kw(false);
    let vw = fixture_vw(false);
    leaf::concrete_le16(page, 2);
    if false {
        leaf::concrete_le32(page, 4);
        leaf::concrete_le32(page, 8);
        leaf::concrete_le32(page, 12);
        leaf::concrete_le32(page, 16);
    }
    assert forall|boundary: int| 0 <= boundary <= 2 implies {
        &&& leaf::metadata_end(page, kw, vw) <= #[trigger] leaf::key_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::key_boundary(page, kw, vw, boundary) <= u32::MAX
        &&& leaf::key_boundary(page, kw, vw, 2) <= #[trigger] leaf::value_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::value_boundary(page, kw, vw, boundary) <= u32::MAX
    } by { assert(boundary == 0 || boundary == 1 || boundary == 2); }
    assert forall|left: int, right: int| 0 <= left <= right <= 2 implies {
        &&& #[trigger] leaf::key_boundary(page, kw, vw, left) <= #[trigger] leaf::key_boundary(page, kw, vw, right)
        &&& #[trigger] leaf::value_boundary(page, kw, vw, left) <= #[trigger] leaf::value_boundary(page, kw, vw, right)
    } by {
        assert(left == 0 || left == 1 || left == 2);
        assert(right == 0 || right == 1 || right == 2);
    }
    assert(leaf::leaf_bytes_wf(page, kw, vw));
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_key(page, kw, vw, index) == fixture_key(1, index) by {
        assert(index == 0 || index == 1);
        assert(1 == 0 || 1 == 1 || 1 == 2);
        assert(leaf::encoded_key(page, kw, vw, index) =~= fixture_key(1, index));
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_value(page, kw, vw, index) == seq![if index == 0 { first } else { second }] by {
        assert(index == 0 || index == 1);
        assert(leaf::encoded_value(page, kw, vw, index) =~= seq![if index == 0 { first } else { second }]);
    }
    decoded_four(fixture_key(1, 0));
    decoded_four(fixture_key(1, 1));
    assert forall|left: int, right: int| 0 <= left < right < 2 implies meaning(#[trigger] leaf::encoded_key(page, kw, vw, left)) < meaning(#[trigger] leaf::encoded_key(page, kw, vw, right)) by {
        assert(left == 0 && right == 1);
    }
    assert(lookup_client::ordered_leaf(page, kw, vw));
}

pub proof fn fixture_leaf_fixed_2(page: Seq<u8>, first: u8, second: u8)
    requires 0 <= 2 < 3, page == leaf_image(2, false, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(false), fixture_vw(false)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(false), fixture_vw(false), index) == fixture_key(2, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(false), fixture_vw(false), index) == seq![if index == 0 { first } else { second }],
{
    let kw = fixture_kw(false);
    let vw = fixture_vw(false);
    leaf::concrete_le16(page, 2);
    if false {
        leaf::concrete_le32(page, 4);
        leaf::concrete_le32(page, 8);
        leaf::concrete_le32(page, 12);
        leaf::concrete_le32(page, 16);
    }
    assert forall|boundary: int| 0 <= boundary <= 2 implies {
        &&& leaf::metadata_end(page, kw, vw) <= #[trigger] leaf::key_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::key_boundary(page, kw, vw, boundary) <= u32::MAX
        &&& leaf::key_boundary(page, kw, vw, 2) <= #[trigger] leaf::value_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::value_boundary(page, kw, vw, boundary) <= u32::MAX
    } by { assert(boundary == 0 || boundary == 1 || boundary == 2); }
    assert forall|left: int, right: int| 0 <= left <= right <= 2 implies {
        &&& #[trigger] leaf::key_boundary(page, kw, vw, left) <= #[trigger] leaf::key_boundary(page, kw, vw, right)
        &&& #[trigger] leaf::value_boundary(page, kw, vw, left) <= #[trigger] leaf::value_boundary(page, kw, vw, right)
    } by {
        assert(left == 0 || left == 1 || left == 2);
        assert(right == 0 || right == 1 || right == 2);
    }
    assert(leaf::leaf_bytes_wf(page, kw, vw));
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_key(page, kw, vw, index) == fixture_key(2, index) by {
        assert(index == 0 || index == 1);
        assert(2 == 0 || 2 == 1 || 2 == 2);
        assert(leaf::encoded_key(page, kw, vw, index) =~= fixture_key(2, index));
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_value(page, kw, vw, index) == seq![if index == 0 { first } else { second }] by {
        assert(index == 0 || index == 1);
        assert(leaf::encoded_value(page, kw, vw, index) =~= seq![if index == 0 { first } else { second }]);
    }
    decoded_four(fixture_key(2, 0));
    decoded_four(fixture_key(2, 1));
    assert forall|left: int, right: int| 0 <= left < right < 2 implies meaning(#[trigger] leaf::encoded_key(page, kw, vw, left)) < meaning(#[trigger] leaf::encoded_key(page, kw, vw, right)) by {
        assert(left == 0 && right == 1);
    }
    assert(lookup_client::ordered_leaf(page, kw, vw));
}

pub proof fn fixture_leaf_variable_0(page: Seq<u8>, first: u8, second: u8)
    requires 0 <= 0 < 3, page == leaf_image(0, true, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(true), fixture_vw(true)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(true), fixture_vw(true), index) == fixture_key(0, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(true), fixture_vw(true), index) == seq![if index == 0 { first } else { second }],
{
    let kw = fixture_kw(true);
    let vw = fixture_vw(true);
    leaf::concrete_le16(page, 2);
    if true {
        leaf::concrete_le32(page, 4);
        leaf::concrete_le32(page, 8);
        leaf::concrete_le32(page, 12);
        leaf::concrete_le32(page, 16);
    }
    assert forall|boundary: int| 0 <= boundary <= 2 implies {
        &&& leaf::metadata_end(page, kw, vw) <= #[trigger] leaf::key_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::key_boundary(page, kw, vw, boundary) <= u32::MAX
        &&& leaf::key_boundary(page, kw, vw, 2) <= #[trigger] leaf::value_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::value_boundary(page, kw, vw, boundary) <= u32::MAX
    } by { assert(boundary == 0 || boundary == 1 || boundary == 2); }
    assert forall|left: int, right: int| 0 <= left <= right <= 2 implies {
        &&& #[trigger] leaf::key_boundary(page, kw, vw, left) <= #[trigger] leaf::key_boundary(page, kw, vw, right)
        &&& #[trigger] leaf::value_boundary(page, kw, vw, left) <= #[trigger] leaf::value_boundary(page, kw, vw, right)
    } by {
        assert(left == 0 || left == 1 || left == 2);
        assert(right == 0 || right == 1 || right == 2);
    }
    assert(leaf::leaf_bytes_wf(page, kw, vw));
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_key(page, kw, vw, index) == fixture_key(0, index) by {
        assert(index == 0 || index == 1);
        assert(0 == 0 || 0 == 1 || 0 == 2);
        assert(leaf::encoded_key(page, kw, vw, index) =~= fixture_key(0, index));
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_value(page, kw, vw, index) == seq![if index == 0 { first } else { second }] by {
        assert(index == 0 || index == 1);
        assert(leaf::encoded_value(page, kw, vw, index) =~= seq![if index == 0 { first } else { second }]);
    }
    decoded_four(fixture_key(0, 0));
    decoded_four(fixture_key(0, 1));
    assert forall|left: int, right: int| 0 <= left < right < 2 implies meaning(#[trigger] leaf::encoded_key(page, kw, vw, left)) < meaning(#[trigger] leaf::encoded_key(page, kw, vw, right)) by {
        assert(left == 0 && right == 1);
    }
    assert(lookup_client::ordered_leaf(page, kw, vw));
}

pub proof fn fixture_leaf_variable_1(page: Seq<u8>, first: u8, second: u8)
    requires 0 <= 1 < 3, page == leaf_image(1, true, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(true), fixture_vw(true)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(true), fixture_vw(true), index) == fixture_key(1, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(true), fixture_vw(true), index) == seq![if index == 0 { first } else { second }],
{
    let kw = fixture_kw(true);
    let vw = fixture_vw(true);
    leaf::concrete_le16(page, 2);
    if true {
        leaf::concrete_le32(page, 4);
        leaf::concrete_le32(page, 8);
        leaf::concrete_le32(page, 12);
        leaf::concrete_le32(page, 16);
    }
    assert forall|boundary: int| 0 <= boundary <= 2 implies {
        &&& leaf::metadata_end(page, kw, vw) <= #[trigger] leaf::key_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::key_boundary(page, kw, vw, boundary) <= u32::MAX
        &&& leaf::key_boundary(page, kw, vw, 2) <= #[trigger] leaf::value_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::value_boundary(page, kw, vw, boundary) <= u32::MAX
    } by { assert(boundary == 0 || boundary == 1 || boundary == 2); }
    assert forall|left: int, right: int| 0 <= left <= right <= 2 implies {
        &&& #[trigger] leaf::key_boundary(page, kw, vw, left) <= #[trigger] leaf::key_boundary(page, kw, vw, right)
        &&& #[trigger] leaf::value_boundary(page, kw, vw, left) <= #[trigger] leaf::value_boundary(page, kw, vw, right)
    } by {
        assert(left == 0 || left == 1 || left == 2);
        assert(right == 0 || right == 1 || right == 2);
    }
    assert(leaf::leaf_bytes_wf(page, kw, vw));
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_key(page, kw, vw, index) == fixture_key(1, index) by {
        assert(index == 0 || index == 1);
        assert(1 == 0 || 1 == 1 || 1 == 2);
        assert(leaf::encoded_key(page, kw, vw, index) =~= fixture_key(1, index));
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_value(page, kw, vw, index) == seq![if index == 0 { first } else { second }] by {
        assert(index == 0 || index == 1);
        assert(leaf::encoded_value(page, kw, vw, index) =~= seq![if index == 0 { first } else { second }]);
    }
    decoded_four(fixture_key(1, 0));
    decoded_four(fixture_key(1, 1));
    assert forall|left: int, right: int| 0 <= left < right < 2 implies meaning(#[trigger] leaf::encoded_key(page, kw, vw, left)) < meaning(#[trigger] leaf::encoded_key(page, kw, vw, right)) by {
        assert(left == 0 && right == 1);
    }
    assert(lookup_client::ordered_leaf(page, kw, vw));
}

pub proof fn fixture_leaf_variable_2(page: Seq<u8>, first: u8, second: u8)
    requires 0 <= 2 < 3, page == leaf_image(2, true, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(true), fixture_vw(true)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(true), fixture_vw(true), index) == fixture_key(2, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(true), fixture_vw(true), index) == seq![if index == 0 { first } else { second }],
{
    let kw = fixture_kw(true);
    let vw = fixture_vw(true);
    leaf::concrete_le16(page, 2);
    if true {
        leaf::concrete_le32(page, 4);
        leaf::concrete_le32(page, 8);
        leaf::concrete_le32(page, 12);
        leaf::concrete_le32(page, 16);
    }
    assert forall|boundary: int| 0 <= boundary <= 2 implies {
        &&& leaf::metadata_end(page, kw, vw) <= #[trigger] leaf::key_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::key_boundary(page, kw, vw, boundary) <= u32::MAX
        &&& leaf::key_boundary(page, kw, vw, 2) <= #[trigger] leaf::value_boundary(page, kw, vw, boundary) <= page.len()
        &&& leaf::value_boundary(page, kw, vw, boundary) <= u32::MAX
    } by { assert(boundary == 0 || boundary == 1 || boundary == 2); }
    assert forall|left: int, right: int| 0 <= left <= right <= 2 implies {
        &&& #[trigger] leaf::key_boundary(page, kw, vw, left) <= #[trigger] leaf::key_boundary(page, kw, vw, right)
        &&& #[trigger] leaf::value_boundary(page, kw, vw, left) <= #[trigger] leaf::value_boundary(page, kw, vw, right)
    } by {
        assert(left == 0 || left == 1 || left == 2);
        assert(right == 0 || right == 1 || right == 2);
    }
    assert(leaf::leaf_bytes_wf(page, kw, vw));
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_key(page, kw, vw, index) == fixture_key(2, index) by {
        assert(index == 0 || index == 1);
        assert(2 == 0 || 2 == 1 || 2 == 2);
        assert(leaf::encoded_key(page, kw, vw, index) =~= fixture_key(2, index));
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] leaf::encoded_value(page, kw, vw, index) == seq![if index == 0 { first } else { second }] by {
        assert(index == 0 || index == 1);
        assert(leaf::encoded_value(page, kw, vw, index) =~= seq![if index == 0 { first } else { second }]);
    }
    decoded_four(fixture_key(2, 0));
    decoded_four(fixture_key(2, 1));
    assert forall|left: int, right: int| 0 <= left < right < 2 implies meaning(#[trigger] leaf::encoded_key(page, kw, vw, left)) < meaning(#[trigger] leaf::encoded_key(page, kw, vw, right)) by {
        assert(left == 0 && right == 1);
    }
    assert(lookup_client::ordered_leaf(page, kw, vw));
}

pub proof fn fixture_leaf(page: Seq<u8>, child: int, variable: bool, first: u8, second: u8)
    requires 0 <= child < 3, page == leaf_image(child, variable, first, second),
    ensures
        lookup_client::ordered_leaf(page, fixture_kw(variable), fixture_vw(variable)),
        leaf::pair_count(page) == 2,
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_key(page, fixture_kw(variable), fixture_vw(variable), index) == fixture_key(child, index),
        forall|index: int| 0 <= index < 2 ==> #[trigger] leaf::encoded_value(page, fixture_kw(variable), fixture_vw(variable), index) == seq![if index == 0 { first } else { second }],
{
    if variable {
        if child == 0 { fixture_leaf_variable_0(page, first, second); }
        else if child == 1 { fixture_leaf_variable_1(page, first, second); }
        else { fixture_leaf_variable_2(page, first, second); }
    } else {
        if child == 0 { fixture_leaf_fixed_0(page, first, second); }
        else if child == 1 { fixture_leaf_fixed_1(page, first, second); }
        else { fixture_leaf_fixed_2(page, first, second); }
    }
}

pub proof fn fixture_interval(bytes: Seq<u8>, kw: Option<usize>, child: int, index: int)
    requires branch::key_count(bytes) == 2,
        meaning(branch::encoded_key(bytes, kw, 0)) == 256,
        meaning(branch::encoded_key(bytes, kw, 1)) == 65536,
        0 <= child < 3, 0 <= index < 2,
    ensures interval(bytes, kw, child, fixture_key(child, index)),
{
    if child == 0 {
        if index == 0 { decoded_four(fixture_key(0, 0)); }
        else { decoded_four(fixture_key(0, 1)); }
    } else if child == 1 {
        if index == 0 { decoded_four(fixture_key(1, 0)); }
        else { decoded_four(fixture_key(1, 1)); }
    } else {
        if index == 0 { decoded_four(fixture_key(2, 0)); }
        else { decoded_four(fixture_key(2, 1)); }
    }
}

pub proof fn branch_count_two(bytes: Seq<u8>)
    requires bytes.len() >= 4, bytes[2] == 2, bytes[3] == 0,
    ensures branch::key_count(bytes) == 2,
{
    leaf_lookup023::concrete_le16(bytes, 2);
}

pub proof fn branch_ordered_two(bytes: Seq<u8>, kw: Option<usize>)
    requires 32 <= bytes.len() <= usize::MAX, bytes[0] == 2,
        branch::key_count(bytes) == 2,
        branch::metadata(2, kw) + 8 <= bytes.len(),
        branch::boundary(bytes, kw, 0) == branch::metadata(2, kw),
        branch::boundary(bytes, kw, 1) == branch::metadata(2, kw) + 4,
        branch::boundary(bytes, kw, 2) == branch::metadata(2, kw) + 8,
        branch::encoded_key(bytes, kw, 0) == seq![0u8, 1, 0, 0],
        branch::encoded_key(bytes, kw, 1) == seq![0u8, 0, 1, 0],
    ensures branch_clients023::ordered_bytes::<u32>(bytes, kw),
        meaning(branch::encoded_key(bytes, kw, 0)) == 256,
        meaning(branch::encoded_key(bytes, kw, 1)) == 65536,
{
    assert forall|boundary: int| 0 <= boundary <= 2 implies branch::metadata(2, kw) <= #[trigger] branch::boundary(bytes, kw, boundary) <= bytes.len() by {
        assert(boundary == 0 || boundary == 1 || boundary == 2);
    }
    assert forall|index: int| 0 <= index < 2 implies #[trigger] branch::boundary(bytes, kw, index) <= branch::boundary(bytes, kw, index + 1) by {
        assert(index == 0 || index == 1);
    }
    decoded_four(branch::encoded_key(bytes, kw, 0));
    decoded_four(branch::encoded_key(bytes, kw, 1));
    meaning_bridge(branch::encoded_key(bytes, kw, 0));
    meaning_bridge(branch::encoded_key(bytes, kw, 1));
    assert forall|index: int| 0 <= index < 2 implies <u32 as leaf_lookup023::Value>::valid(#[trigger] branch::encoded_key(bytes, kw, index)) by {
        assert(index == 0 || index == 1);
    }
    assert forall|left: int, right: int| 0 <= left < right < 2 implies <u32 as leaf_lookup023::Value>::meaning(#[trigger] branch::encoded_key(bytes, kw, left)) < <u32 as leaf_lookup023::Value>::meaning(#[trigger] branch::encoded_key(bytes, kw, right)) by {
        assert(left == 0 && right == 1);
    }
}

pub proof fn fixture_branch_fixed(bytes: Seq<u8>)
    requires bytes == branch_image(false),
    ensures branch_clients023::ordered_bytes::<u32>(bytes, fixture_kw(false)),
        branch::key_count(bytes) == 2,
        branch::encoded_key(bytes, fixture_kw(false), 0) == seq![0u8, 1, 0, 0],
        branch::encoded_key(bytes, fixture_kw(false), 1) == seq![0u8, 0, 1, 0],
        meaning(branch::encoded_key(bytes, fixture_kw(false), 0)) == 256,
        meaning(branch::encoded_key(bytes, fixture_kw(false), 1)) == 65536,

{
    let kw = fixture_kw(false);
    branch_count_two(bytes);
    assert(branch::encoded_key(bytes, kw, 0) =~= seq![0u8, 1, 0, 0]);
    assert(branch::encoded_key(bytes, kw, 1) =~= seq![0u8, 0, 1, 0]);
    branch_ordered_two(bytes, kw);
}

pub proof fn fixture_lawful_fixed(bytes: Seq<u8>, leaves: Seq<&[u8]>,
    first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8)
    requires bytes == branch_image(false), leaves.len() == 3,
        leaves[0]@ == leaf_image(0, false, first, second),
        leaves[1]@ == leaf_image(1, false, third, fourth),
        leaves[2]@ == leaf_image(2, false, fifth, sixth),
    ensures supplied(bytes, leaves, fixture_kw(false), fixture_vw(false)),
        branch::key_count(bytes) == 2,
        branch::encoded_key(bytes, fixture_kw(false), 0) == seq![0u8, 1, 0, 0],
        branch::encoded_key(bytes, fixture_kw(false), 1) == seq![0u8, 0, 1, 0],
        meaning(branch::encoded_key(bytes, fixture_kw(false), 0)) == 256,
        meaning(branch::encoded_key(bytes, fixture_kw(false), 1)) == 65536,
        forall|child: int| 0 <= child < 3 ==> leaf::pair_count(#[trigger] leaves[child]@) == 2,
        forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 ==> #[trigger] leaf::encoded_key(leaves[child]@, fixture_kw(false), fixture_vw(false), index) == fixture_key(child, index),
        forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 ==> #[trigger] leaf::encoded_value(leaves[child]@, fixture_kw(false), fixture_vw(false), index) == seq![if child == 0 { if index == 0 { first } else { second } } else if child == 1 { if index == 0 { third } else { fourth } } else { if index == 0 { fifth } else { sixth } }],
{
    let kw = fixture_kw(false);
    let vw = fixture_vw(false);
    fixture_branch_fixed(bytes);
    fixture_leaf(leaves[0]@, 0, false, first, second);
    fixture_leaf(leaves[1]@, 1, false, third, fourth);
    fixture_leaf(leaves[2]@, 2, false, fifth, sixth);
    assert forall|child: int| 0 <= child < 3 implies lookup_client::ordered_leaf(#[trigger] leaves[child]@, kw, vw) && leaf::pair_count(leaves[child]@) == 2 by {
        assert(child == 0 || child == 1 || child == 2);
    }
    assert forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 implies interval(bytes, kw, child, #[trigger] leaf::encoded_key(leaves[child]@, kw, vw, index)) by {
        assert(child == 0 || child == 1 || child == 2);
        assert(index == 0 || index == 1);
        fixture_interval(bytes, kw, child, index);
    }
}

pub proof fn fixture_branch_variable(bytes: Seq<u8>)
    requires bytes == branch_image(true),
    ensures branch_clients023::ordered_bytes::<u32>(bytes, fixture_kw(true)),
        branch::key_count(bytes) == 2,
        branch::encoded_key(bytes, fixture_kw(true), 0) == seq![0u8, 1, 0, 0],
        branch::encoded_key(bytes, fixture_kw(true), 1) == seq![0u8, 0, 1, 0],
        meaning(branch::encoded_key(bytes, fixture_kw(true), 0)) == 256,
        meaning(branch::encoded_key(bytes, fixture_kw(true), 1)) == 65536,

{
    let kw = fixture_kw(true);
    branch_count_two(bytes);
    leaf_lookup023::concrete_le32(bytes, 80);
    leaf_lookup023::concrete_le32(bytes, 84);
    assert(branch::encoded_key(bytes, kw, 0) =~= seq![0u8, 1, 0, 0]);
    assert(branch::encoded_key(bytes, kw, 1) =~= seq![0u8, 0, 1, 0]);
    branch_ordered_two(bytes, kw);
}

pub proof fn fixture_lawful_variable(bytes: Seq<u8>, leaves: Seq<&[u8]>,
    first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8)
    requires bytes == branch_image(true), leaves.len() == 3,
        leaves[0]@ == leaf_image(0, true, first, second),
        leaves[1]@ == leaf_image(1, true, third, fourth),
        leaves[2]@ == leaf_image(2, true, fifth, sixth),
    ensures supplied(bytes, leaves, fixture_kw(true), fixture_vw(true)),
        branch::key_count(bytes) == 2,
        branch::encoded_key(bytes, fixture_kw(true), 0) == seq![0u8, 1, 0, 0],
        branch::encoded_key(bytes, fixture_kw(true), 1) == seq![0u8, 0, 1, 0],
        meaning(branch::encoded_key(bytes, fixture_kw(true), 0)) == 256,
        meaning(branch::encoded_key(bytes, fixture_kw(true), 1)) == 65536,
        forall|child: int| 0 <= child < 3 ==> leaf::pair_count(#[trigger] leaves[child]@) == 2,
        forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 ==> #[trigger] leaf::encoded_key(leaves[child]@, fixture_kw(true), fixture_vw(true), index) == fixture_key(child, index),
        forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 ==> #[trigger] leaf::encoded_value(leaves[child]@, fixture_kw(true), fixture_vw(true), index) == seq![if child == 0 { if index == 0 { first } else { second } } else if child == 1 { if index == 0 { third } else { fourth } } else { if index == 0 { fifth } else { sixth } }],
{
    let kw = fixture_kw(true);
    let vw = fixture_vw(true);
    fixture_branch_variable(bytes);
    fixture_leaf(leaves[0]@, 0, true, first, second);
    fixture_leaf(leaves[1]@, 1, true, third, fourth);
    fixture_leaf(leaves[2]@, 2, true, fifth, sixth);
    assert forall|child: int| 0 <= child < 3 implies lookup_client::ordered_leaf(#[trigger] leaves[child]@, kw, vw) && leaf::pair_count(leaves[child]@) == 2 by {
        assert(child == 0 || child == 1 || child == 2);
    }
    assert forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 implies interval(bytes, kw, child, #[trigger] leaf::encoded_key(leaves[child]@, kw, vw, index)) by {
        assert(child == 0 || child == 1 || child == 2);
        assert(index == 0 || index == 1);
        fixture_interval(bytes, kw, child, index);
    }
}

pub proof fn fixture_lawful(bytes: Seq<u8>, leaves: Seq<&[u8]>, variable: bool,
    first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8)
    requires bytes == branch_image(variable), leaves.len() == 3,
        leaves[0]@ == leaf_image(0, variable, first, second),
        leaves[1]@ == leaf_image(1, variable, third, fourth),
        leaves[2]@ == leaf_image(2, variable, fifth, sixth),
    ensures supplied(bytes, leaves, fixture_kw(variable), fixture_vw(variable)),
        branch::key_count(bytes) == 2,
        branch::encoded_key(bytes, fixture_kw(variable), 0) == seq![0u8, 1, 0, 0],
        branch::encoded_key(bytes, fixture_kw(variable), 1) == seq![0u8, 0, 1, 0],
        meaning(branch::encoded_key(bytes, fixture_kw(variable), 0)) == 256,
        meaning(branch::encoded_key(bytes, fixture_kw(variable), 1)) == 65536,
        forall|child: int| 0 <= child < 3 ==> leaf::pair_count(#[trigger] leaves[child]@) == 2,
        forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 ==> #[trigger] leaf::encoded_key(leaves[child]@, fixture_kw(variable), fixture_vw(variable), index) == fixture_key(child, index),
        forall|child: int, index: int| 0 <= child < 3 && 0 <= index < 2 ==> #[trigger] leaf::encoded_value(leaves[child]@, fixture_kw(variable), fixture_vw(variable), index) == seq![if child == 0 { if index == 0 { first } else { second } } else if child == 1 { if index == 0 { third } else { fourth } } else { if index == 0 { fifth } else { sixth } }],
{
    if variable { fixture_lawful_variable(bytes, leaves, first, second, third, fourth, fifth, sixth); }
    else { fixture_lawful_fixed(bytes, leaves, first, second, third, fourth, fifth, sixth); }
}

pub proof fn numeric_not_lexical()
    ensures meaning(fixture_key(0, 0)) < meaning(fixture_key(0, 1)),
        fixture_key(0, 0)[0] > fixture_key(0, 1)[0],
        meaning(fixture_key(0, 1)) < meaning(fixture_key(1, 1)),
        fixture_key(0, 1)[0] == fixture_key(1, 1)[0],
        fixture_key(0, 1)[1] > fixture_key(1, 1)[1],
{
    decoded_four(fixture_key(0, 0));
    decoded_four(fixture_key(0, 1));
    decoded_four(fixture_key(1, 1));
}

pub fn fixed_query_0(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [1, 0, 0, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[0]@, kw, vw, 0) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 0);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![first]);

}

pub fn fixed_query_1(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [0, 1, 0, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[0]@, kw, vw, 1) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 0);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![second]);

}

pub fn fixed_query_2(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [1, 1, 0, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[1]@, kw, vw, 0) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 1);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![third]);

}

pub fn fixed_query_3(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [0, 0, 1, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[1]@, kw, vw, 1) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 1);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![fourth]);

}

pub fn fixed_query_4(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [1, 0, 1, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[2]@, kw, vw, 0) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 2);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![fifth]);

}

pub fn fixed_query_5(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [255, 255, 255, 255];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[2]@, kw, vw, 1) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 2);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![sixth]);

}

pub fn fixed_query_6(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [0, 0, 0, 0];
    proof { decoded_four(query@); }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.2.is_none());

}

pub fn fixed_query_7(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 88] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 14] = [1, 0, 2, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 14] = [1, 0, 2, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 14] = [1, 0, 2, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = Some(4usize);
    let vw = Some(1usize);
    proof {
        fixture_lawful(bytes@, leaves@, false, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [2, 0, 0, 0];
    proof { decoded_four(query@); }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.2.is_none());
}

pub fn fixed_witness(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    fixed_query_0(first, second, third, fourth, fifth, sixth);
    fixed_query_1(first, second, third, fourth, fifth, sixth);
    fixed_query_2(first, second, third, fourth, fifth, sixth);
    fixed_query_3(first, second, third, fourth, fifth, sixth);
    fixed_query_4(first, second, third, fourth, fifth, sixth);
    fixed_query_5(first, second, third, fourth, fifth, sixth);
    fixed_query_6(first, second, third, fourth, fifth, sixth);
    fixed_query_7(first, second, third, fourth, fifth, sixth);
}

pub fn variable_query_0(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [1, 0, 0, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[0]@, kw, vw, 0) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 0);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![first]);

}

pub fn variable_query_1(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [0, 1, 0, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[0]@, kw, vw, 1) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 0);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![second]);

}

pub fn variable_query_2(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [1, 1, 0, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[1]@, kw, vw, 0) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 1);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![third]);

}

pub fn variable_query_3(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [0, 0, 1, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[1]@, kw, vw, 1) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 1);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![fourth]);

}

pub fn variable_query_4(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [1, 0, 1, 0];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[2]@, kw, vw, 0) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 2);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![fifth]);

}

pub fn variable_query_5(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [255, 255, 255, 255];
    proof { decoded_four(query@); }
    proof {
        assert(leaf::encoded_key(leaves@[2]@, kw, vw, 1) == query@);
        assert(union_has(leaves@, kw, vw, query@));
    }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.0 == 2);
    assert(result.2.is_some());
    assert(result.2.unwrap().0@ == query@);
    assert(result.2.unwrap().1@ == seq![sixth]);

}

pub fn variable_query_6(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [0, 0, 0, 0];
    proof { decoded_four(query@); }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.2.is_none());

}

pub fn variable_query_7(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    let bytes: [u8; 96] = [2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 96, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0];
    let left: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, first, second];
    let middle: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, third, fourth];
    let right: [u8; 30] = [1, 0, 2, 0, 24, 0, 0, 0, 28, 0, 0, 0, 29, 0, 0, 0, 30, 0, 0, 0, 1, 0, 1, 0, 255, 255, 255, 255, fifth, sixth];
    let leaves: [&[u8]; 3] = [&left, &middle, &right];
    let kw = None;
    let vw = None;
    proof {
        fixture_lawful(bytes@, leaves@, true, first, second, third, fourth, fifth, sixth);
    }
    let query: [u8; 4] = [2, 0, 0, 0];
    proof { decoded_four(query@); }
    let result = route_then_search(&bytes, &leaves, kw, vw, &query);
    assert(result.2.is_none());
}

pub fn variable_witness(first: u8, second: u8, third: u8, fourth: u8, fifth: u8, sixth: u8) {
    variable_query_0(first, second, third, fourth, fifth, sixth);
    variable_query_1(first, second, third, fourth, fifth, sixth);
    variable_query_2(first, second, third, fourth, fifth, sixth);
    variable_query_3(first, second, third, fourth, fifth, sixth);
    variable_query_4(first, second, third, fourth, fifth, sixth);
    variable_query_5(first, second, third, fourth, fifth, sixth);
    variable_query_6(first, second, third, fourth, fifth, sixth);
    variable_query_7(first, second, third, fourth, fifth, sixth);
}

}
