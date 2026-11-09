use vstd::prelude::*;
use vstd::arithmetic::div_mod::{
    lemma_div_is_ordered,
    lemma_fundamental_div_mod,
    lemma_fundamental_div_mod_converse,
    lemma_mod_bound,
};
use vstd::arithmetic::mul::{lemma_mul_inequality, lemma_mul_is_commutative};
use vstd::arithmetic::power2::is_pow2;
use vstd::layout::valid_layout;

verus! {

pub struct OriginalLayoutValues171 {
    pub len: usize,
    pub element_size: usize,
    pub element_alignment_usize: usize,
    pub array_limit: usize,
    pub payload_size: usize,
    pub payload_alignment_usize: usize,
    pub header_size: usize,
    pub header_alignment_usize: usize,
    pub payload_offset: usize,
    pub extended_alignment_usize: usize,
    pub extend_limit: usize,
    pub combined_size: usize,
    pub padded_size: usize,
}

pub open spec fn round_up171(size: int, alignment: int) -> int
    recommends
        alignment > 0,
{
    ((size + alignment - 1) / alignment) * alignment
}

pub open spec fn original_u8_value_relations171(values: OriginalLayoutValues171) -> bool {
    &&& values.element_size == 1
    &&& values.element_alignment_usize == 1
    &&& values.array_limit as int
        == isize::MAX as int + 1 - values.element_alignment_usize as int
    &&& values.payload_size as int == values.element_size as int * values.len as int
    &&& values.payload_alignment_usize == values.element_alignment_usize
    &&& values.header_size > 0
    &&& values.header_alignment_usize > 0
    &&& values.payload_offset == (values.header_size & !0usize)
    &&& values.extended_alignment_usize == if values.header_alignment_usize
        >= values.payload_alignment_usize {
            values.header_alignment_usize
        } else {
            values.payload_alignment_usize
        }
    &&& values.extend_limit as int
        == isize::MAX as int + 1 - values.extended_alignment_usize as int
    &&& values.combined_size as int
        == values.payload_offset as int + values.payload_size as int
    &&& values.padded_size == (
        ((values.combined_size as int + values.extended_alignment_usize as int - 1) as usize)
        & !((values.extended_alignment_usize as int - 1) as usize)
    )
}

pub open spec fn original_u8_normal_guards171(values: OriginalLayoutValues171) -> bool {
    &&& values.element_size != 0
    &&& values.len as int <= values.array_limit as int / values.element_size as int
    &&& values.combined_size <= values.extend_limit
}

pub open spec fn observed_header156_171(values: OriginalLayoutValues171) -> bool {
    values.header_size == 16 && values.header_alignment_usize == 8
}

pub open spec fn allocation_size171(len: usize) -> int {
    16 + ((len as int + 7) / 8) * 8
}

pub proof fn lemma_extend_threshold171(
    maximum: int, length: int, header_size: int, alignment: int,
)
    requires
        0 <= length,
        0 < header_size,
        0 < alignment,
    ensures
        (header_size + length <= maximum + 1 - alignment)
            <==> (length <= maximum - header_size - (alignment - 1)),
{
    assert(maximum + 1 - alignment - header_size
        == maximum - header_size - (alignment - 1));
}

pub proof fn lemma_round_up171(size: int, alignment: int, ceiling: int)
    requires
        0 <= size <= ceiling,
        0 < alignment,
        ceiling % alignment == 0,
    ensures
        size <= round_up171(size, alignment) <= size + alignment - 1,
        round_up171(size, alignment) <= ceiling,
        round_up171(size, alignment) % alignment == 0,
{
    let numerator = size + alignment - 1;
    let quotient = numerator / alignment;
    let remainder = numerator % alignment;
    lemma_fundamental_div_mod(numerator, alignment);
    lemma_mod_bound(numerator, alignment);
    lemma_mul_is_commutative(alignment, quotient);
    lemma_fundamental_div_mod(ceiling, alignment);
    lemma_mul_is_commutative(alignment, ceiling / alignment);
    lemma_fundamental_div_mod_converse(
        ceiling + alignment - 1, alignment, ceiling / alignment, alignment - 1,
    );
    lemma_div_is_ordered(numerator, ceiling + alignment - 1, alignment);
    lemma_mul_inequality(quotient, ceiling / alignment, alignment);
    assert(numerator == quotient * alignment + remainder);
    lemma_fundamental_div_mod_converse(quotient * alignment, alignment, quotient, 0);
}

pub proof fn lemma_mask8_171(word: usize)
    ensures
        (word & !7usize) as int == (word as int / 8) * 8,
{
    assert((word & !7usize) == word - (word % 8usize)) by (bit_vector);
    lemma_fundamental_div_mod(word as int, 8);
}

pub proof fn lemma_signed_limit8_171()
    ensures
        isize::MAX as int % 8 == 7,
        ((isize::MAX as usize) & !7usize) as int == isize::MAX as int + 1 - 8,
        (isize::MAX as int - 7) % 8 == 0,
        16 <= isize::MAX as int - 7,
        isize::MAX as int + 1 <= usize::MAX as int,
{
    assert((isize::MAX as usize) % 8usize == 7usize) by (bit_vector);
    lemma_mask8_171(isize::MAX as usize);
    lemma_fundamental_div_mod(isize::MAX as int, 8);
    lemma_fundamental_div_mod_converse(
        isize::MAX as int - 7, 8, isize::MAX as int / 8, 0,
    );
}

pub proof fn lemma_original_threshold_iff171(values: OriginalLayoutValues171)
    requires
        original_u8_value_relations171(values),
        observed_header156_171(values),
    ensures
        original_u8_normal_guards171(values)
            <==> (values.len <= isize::MAX as int - 16 - 7),
        values.payload_offset == 16,
        values.payload_size == values.len,
        values.payload_alignment_usize == 1,
        values.extended_alignment_usize == 8,
        values.combined_size as int == 16 + values.len as int,
        values.extend_limit as int == isize::MAX as int - 7,
{
    assert((values.header_size & !0usize) == values.header_size) by (bit_vector);
    lemma_extend_threshold171(isize::MAX as int, values.len as int, 16, 8);
}

pub proof fn derive_original_layout171(len: usize, values: OriginalLayoutValues171)
    requires
        values.len == len,
        original_u8_value_relations171(values),
        original_u8_normal_guards171(values),
        observed_header156_171(values),
    ensures
        len <= isize::MAX as int - 16 - 7,
        len as int + 7 <= usize::MAX as int,
        values.payload_offset == 16,
        values.payload_size == len,
        values.extended_alignment_usize == 8,
        values.combined_size as int == 16 + len as int,
        0 < values.combined_size as int <= isize::MAX as int - 7,
        values.combined_size as int + 7 <= isize::MAX as int,
        values.padded_size as int == round_up171(values.combined_size as int, 8),
        values.padded_size as int == allocation_size171(len),
        16 + len as int <= values.padded_size as int <= isize::MAX as int - 7,
        0 < values.padded_size,
        values.padded_size as int % 8 == 0,
        valid_layout(values.combined_size, 8),
        valid_layout(values.padded_size, 8),
        0 <= values.padded_size as int - values.combined_size as int <= 7,
        len == 0 ==> values.combined_size == 16 && values.padded_size == 16,
        len == 0 ==> values.padded_size as int - values.payload_offset as int == 0,
{
    lemma_signed_limit8_171();
    lemma_original_threshold_iff171(values);
    lemma_round_up171(values.combined_size as int, 8, isize::MAX as int - 7);
    let word = (values.combined_size as int + 7) as usize;
    assert(word as int == values.combined_size as int + 7);
    lemma_mask8_171(word);
    lemma_fundamental_div_mod(len as int + 7, 8);
    lemma_mod_bound(len as int + 7, 8);
    lemma_fundamental_div_mod_converse(
        values.combined_size as int + 7,
        8,
        (len as int + 7) / 8 + 2,
        (len as int + 7) % 8,
    );
    assert(round_up171(values.combined_size as int, 8) == allocation_size171(len));
    reveal(is_pow2);
    assert(is_pow2(8)) by (compute);
    assert(valid_layout(values.combined_size, 8));
    assert(valid_layout(values.padded_size, 8));
    if len == 0 {
        assert(allocation_size171(len) == 16);
    }
}

pub proof fn derive_original_cell171(
    len: usize, index: usize, values: OriginalLayoutValues171,
)
    requires
        values.len == len,
        original_u8_value_relations171(values),
        original_u8_normal_guards171(values),
        observed_header156_171(values),
        index < len,
    ensures
        16 <= values.payload_offset as int + index as int
            < values.payload_offset as int + len as int,
        values.payload_offset as int + index as int + 1 <= values.padded_size as int,
        values.payload_offset as int + index as int < isize::MAX as int,
        index < isize::MAX as int,
        values.payload_offset as int + index as int <= usize::MAX as int,
        values.payload_offset as int + index as int + 1 <= usize::MAX as int,
{
    derive_original_layout171(len, values);
}

pub proof fn derive_original_address171(
    base_address: usize, len: usize, index: usize, values: OriginalLayoutValues171,
)
    requires
        values.len == len,
        original_u8_value_relations171(values),
        original_u8_normal_guards171(values),
        observed_header156_171(values),
        index < len,
        base_address as int + values.padded_size as int <= usize::MAX as int + 1,
    ensures
        base_address as int + values.payload_offset as int <= usize::MAX as int,
        base_address as int + values.payload_offset as int + index as int
            <= usize::MAX as int,
        base_address as int + values.payload_offset as int + index as int
            < base_address as int + values.padded_size as int,
        base_address as int + values.payload_offset as int + index as int + 1
            <= base_address as int + values.padded_size as int,
{
    derive_original_cell171(len, index, values);
}

}
