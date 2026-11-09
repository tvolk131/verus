use vstd::prelude::*;
use builder_step::{RawLeafBuilder, metadata, part_views, payload_sum};
use lookup_client::{request, ordered_inputs, build_then_lookup};

verus! {

pub fn leaf_split_required(
    num_pairs: usize,
    keys_values_bytes: usize,
    fixed_key_size: Option<usize>,
    fixed_value_size: Option<usize>,
    page_size: usize,
) -> (result: bool)
    requires metadata(num_pairs as int, fixed_key_size, fixed_value_size) + keys_values_bytes <= usize::MAX,
    ensures
        result == (num_pairs > 1 && !(metadata(num_pairs as int, fixed_key_size, fixed_value_size) + keys_values_bytes <= page_size && num_pairs <= 65535)),
        num_pairs > 1 && !result ==> metadata(num_pairs as int, fixed_key_size, fixed_value_size) + keys_values_bytes <= page_size && num_pairs <= 65535,
{
    !leaf_fits_one_page(
        num_pairs,
        keys_values_bytes,
        fixed_key_size,
        fixed_value_size,
        page_size,
    ) && num_pairs > 1
}

pub fn leaf_fits_one_page(
    num_pairs: usize,
    keys_values_bytes: usize,
    fixed_key_size: Option<usize>,
    fixed_value_size: Option<usize>,
    page_size: usize,
) -> (result: bool)
    requires metadata(num_pairs as int, fixed_key_size, fixed_value_size) + keys_values_bytes <= usize::MAX,
    ensures result == (metadata(num_pairs as int, fixed_key_size, fixed_value_size) + keys_values_bytes <= page_size && num_pairs <= 65535),
{
    let required = RawLeafBuilder::required_bytes(
        num_pairs,
        keys_values_bytes,
        fixed_key_size,
        fixed_value_size,
    );
    required <= page_size && u16::try_from(num_pairs).is_ok()
}

pub fn fit_then_lookup<'a>(page: &'a mut [u8], keys: &[&[u8]], values: &[&[u8]],
    key_width: Option<usize>, value_width: Option<usize>, reserved: usize,
    payload: usize, query: &[u8])
    -> (result: (bool, Option<(&'a [u8], &'a [u8])>))
    requires
        keys.len() >= 1,
        values.len() == keys.len(),
        reserved as int == payload_sum(part_views(keys@), keys@.len() as int),
        payload as int == reserved as int + payload_sum(part_views(values@), values@.len() as int),
        metadata(keys@.len() as int, key_width, value_width) + payload <= usize::MAX,
        metadata(keys@.len() as int, key_width, value_width) + payload <= u32::MAX,
        forall|index: int| 0 <= index < keys.len() && key_width.is_some()
            ==> #[trigger] keys@[index]@.len() == key_width.unwrap(),
        forall|index: int| 0 <= index < values.len() && value_width.is_some()
            ==> #[trigger] values@[index]@.len() == value_width.unwrap(),
        ordered_inputs(keys@), query.len() == 4,
    ensures
        result.0 == (metadata(keys@.len() as int, key_width, value_width) + payload <= old(page)@.len() && keys.len() <= 65535),
        !result.0 ==> result.1.is_none(),
        result.0 ==> (result.1.is_some() == exists|index: int| 0 <= index < keys@.len() && #[trigger] keys@[index]@ == query@),
        result.1.is_some() ==> result.1.unwrap().0@ == query@,
        result.1.is_some() ==> exists|index: int| 0 <= index < keys@.len()
            && #[trigger] keys@[index]@ == query@ && values@[index]@ == result.1.unwrap().1@,
{
    let fits = leaf_fits_one_page(keys.len(), payload, key_width, value_width, page.len());
    if !fits {
        (false, None)
    } else {
        proof { assert(request(page@, keys@, values@, key_width, value_width, reserved)); }
        let found = build_then_lookup(page, keys, values, key_width, value_width, reserved, query);
        (true, found)
    }
}

pub fn scalar_boundaries() {
    let equality = leaf_fits_one_page(2, 8, Some(4), Some(0), 12);
    assert(equality);
    let below = leaf_fits_one_page(65535, 0, Some(0), Some(0), 4);
    assert(below);
    let above = leaf_fits_one_page(65536, 0, Some(0), Some(0), 4);
    assert(!above);
    let split = leaf_split_required(1, 5, Some(4), Some(1), 8);
    let fits = leaf_fits_one_page(1, 5, Some(4), Some(1), 8);
    assert(!split && !fits);
    let zero = leaf_fits_one_page(0, 0, None, None, 4);
    let zero_split = leaf_split_required(0, 0, None, None, 4);
    assert(zero && !zero_split);
}

pub fn caller_witness(page: &mut [u8], key_width: Option<usize>, value_width: Option<usize>, present: bool)
    requires
        key_width.is_none() || key_width == Some(4),
        value_width.is_none() || value_width == Some(0),
{
    let key_one = [1u8, 0, 0, 0];
    let key_256 = [0u8, 1, 0, 0];
    let absent = [2u8, 0, 0, 0];
    let empty: [u8; 0] = [];
    let keys: [&[u8]; 2] = [&key_one, &key_256];
    let values: [&[u8]; 2] = [&empty, &empty];
    let query: &[u8] = if present { &key_256 } else { &absent };
    let ghost capacity = page@.len();
    proof {
        assert(part_views(keys@) =~= seq![key_one@, key_256@]);
        assert(part_views(values@) =~= seq![empty@, empty@]);
        reveal_with_fuel(payload_sum, 3);
        vstd::bytes::lemma_u32_from_le_bytes_expansion(key_one@);
        vstd::bytes::lemma_u32_from_le_bytes_expansion(key_256@);
        assert((1u32 | (0u32 << 8) | (0u32 << 16) | (0u32 << 24)) == 1u32) by (bit_vector);
        assert((0u32 | (1u32 << 8) | (0u32 << 16) | (0u32 << 24)) == 256u32) by (bit_vector);
        assert(<u32 as leaf_decoder::Value>::meaning(key_one@) == 1);
        assert(<u32 as leaf_decoder::Value>::meaning(key_256@) == 256);
        assert forall|index: int| 0 <= index < keys@.len()
            implies #[trigger] keys@[index]@.len() == 4 by {
            assert(index == 0 || index == 1);
        }
        assert forall|left: int, right: int| 0 <= left < right < keys@.len()
            implies <u32 as leaf_decoder::Value>::meaning(#[trigger] keys@[left]@)
                < <u32 as leaf_decoder::Value>::meaning(#[trigger] keys@[right]@) by {
            assert(left == 0 && right == 1);
        }
        assert(ordered_inputs(keys@));
    }
    let result = fit_then_lookup(page, &keys, &values, key_width, value_width, 8, 8, query);
    assert(result.0 == (metadata(2, key_width, value_width) + 8 <= capacity));
    assert(result.1.is_some() == (result.0 && present));
    if result.1.is_some() {
        assert(result.1.unwrap().0@ == key_256@);
        assert(result.1.unwrap().1@ == empty@);
    }
    proof {
        if capacity < metadata(2, key_width, value_width) + 8 {
            assert(!result.0 && result.1.is_none());
        }
    }
}

pub fn concrete_caller_cases() {
    let mut fixed_fixed = [0u8; 12];
    caller_witness(&mut fixed_fixed, Some(4), Some(0), true);
    caller_witness(&mut fixed_fixed, Some(4), Some(0), false);
    let mut fixed_variable = [0u8; 20];
    caller_witness(&mut fixed_variable, Some(4), None, true);
    caller_witness(&mut fixed_variable, Some(4), None, false);
    let mut variable_fixed = [0u8; 20];
    caller_witness(&mut variable_fixed, None, Some(0), true);
    caller_witness(&mut variable_fixed, None, Some(0), false);
    let mut variable_variable = [0u8; 28];
    caller_witness(&mut variable_variable, None, None, true);
    caller_witness(&mut variable_variable, None, None, false);
    let mut spare = [0u8; 32];
    caller_witness(&mut spare, Some(4), Some(0), true);
    caller_witness(&mut spare, Some(4), None, false);
    caller_witness(&mut spare, None, Some(0), true);
    caller_witness(&mut spare, None, None, false);
    let mut too_short = [0u8; 3];
    caller_witness(&mut too_short, Some(4), Some(0), true);
    caller_witness(&mut too_short, Some(4), None, false);
    caller_witness(&mut too_short, None, Some(0), true);
    caller_witness(&mut too_short, None, None, false);
}

}
