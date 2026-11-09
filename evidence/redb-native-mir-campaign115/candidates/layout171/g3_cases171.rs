mod g3_scalar171;
use g3_scalar171::*;
use vstd::prelude::*;

verus! {

pub open spec fn proposed_values171(length: usize) -> OriginalLayoutValues171 {
    OriginalLayoutValues171 {
        len: length,
        element_size: 1,
        element_alignment_usize: 1,
        array_limit: isize::MAX as usize,
        payload_size: length,
        payload_alignment_usize: 1,
        header_size: 16,
        header_alignment_usize: 8,
        payload_offset: 16,
        extended_alignment_usize: 8,
        extend_limit: (isize::MAX as int - 7) as usize,
        combined_size: (16 + length as int) as usize,
        padded_size: (((16 + length as int + 7) as usize) & !7usize),
    }
}

pub proof fn proposed_accepted171(length: usize)
    requires
        length <= isize::MAX as int - 23,
    ensures
        original_u8_value_relations171(proposed_values171(length)),
        original_u8_normal_guards171(proposed_values171(length)),
        observed_header156_171(proposed_values171(length)),
        proposed_values171(length).padded_size as int == allocation_size171(length),
{
    lemma_signed_limit8_171();
    let values = proposed_values171(length);
    assert((16usize & !0usize) == 16usize) by (bit_vector);
    assert(original_u8_value_relations171(values));
    lemma_original_threshold_iff171(values);
    derive_original_layout171(length, values);
}

pub proof fn proposed_padding_table171()
    ensures
        allocation_size171(0) == 16,
        allocation_size171(1) == 24,
        allocation_size171(7) == 24,
        allocation_size171(8) == 24,
        allocation_size171(9) == 32,
        proposed_values171(0).combined_size == 16,
        proposed_values171(0).padded_size == 16,
{
    proposed_accepted171(0);
    proposed_accepted171(1);
    proposed_accepted171(7);
    proposed_accepted171(8);
    proposed_accepted171(9);
}

pub proof fn proposed_exact_limit171()
    ensures
        original_u8_normal_guards171(
            proposed_values171((isize::MAX as int - 23) as usize)),
        proposed_values171((isize::MAX as int - 23) as usize).combined_size as int
            == isize::MAX as int - 7,
        proposed_values171((isize::MAX as int - 23) as usize).padded_size as int
            == isize::MAX as int - 7,
{
    let length = (isize::MAX as int - 23) as usize;
    proposed_accepted171(length);
    derive_original_layout171(length, proposed_values171(length));
}

pub proof fn proposed_rejects_next_length171()
    ensures
        proposed_values171((isize::MAX as int - 22) as usize).len
            <= proposed_values171((isize::MAX as int - 22) as usize).array_limit,
        !original_u8_normal_guards171(
            proposed_values171((isize::MAX as int - 22) as usize)),
{
    lemma_signed_limit8_171();
}

pub proof fn proposed_rejects_array_overflow171()
    ensures
        !original_u8_normal_guards171(
            proposed_values171((isize::MAX as int + 1) as usize)),
{
    lemma_signed_limit8_171();
}

pub proof fn proposed_rejects_zero_total171(values: OriginalLayoutValues171)
    requires
        original_u8_value_relations171(values),
        original_u8_normal_guards171(values),
        observed_header156_171(values),
    ensures
        values.combined_size != 0,
        values.padded_size != 0,
        values.len == 0 ==> values.padded_size == 16,
{
    derive_original_layout171(values.len, values);
}

pub proof fn proposed_rejects_wrong_padding171(values: OriginalLayoutValues171)
    requires
        values.combined_size == 17,
        values.extended_alignment_usize == 8,
        values.padded_size != 24,
    ensures
        !original_u8_value_relations171(values),
{
    assert((24usize & !7usize) == 24usize) by (bit_vector);
}

pub proof fn proposed_rejects_numeric_projection171(values: OriginalLayoutValues171)
    requires
        observed_header156_171(values),
        values.extended_alignment_usize != 8
            || values.payload_alignment_usize != 1
            || values.payload_offset != 16,
    ensures
        !original_u8_value_relations171(values),
{
    if original_u8_value_relations171(values) {
        lemma_original_threshold_iff171(values);
    }
}

pub proof fn proposed_rejects_header_substitution171(values: OriginalLayoutValues171)
    requires
        values.header_size != 16 || values.header_alignment_usize != 8,
    ensures
        !observed_header156_171(values),
{
}

pub proof fn proposed_rejects_len_substitution171(
    caller_len: usize, values: OriginalLayoutValues171,
)
    requires
        caller_len != values.len,
    ensures
        !(values.len == caller_len
            && original_u8_value_relations171(values)
            && original_u8_normal_guards171(values)),
{
}

pub proof fn proposed_rejects_padding_index171()
    ensures
        16 + 1 < proposed_values171(1).padded_size,
        !(1usize < proposed_values171(1).len),
        !(0usize < proposed_values171(0).len),
{
    proposed_accepted171(1);
    proposed_accepted171(0);
}

pub proof fn proposed_first_and_last_cells171(length: usize)
    requires
        0 < length <= isize::MAX as int - 23,
    ensures
        16 + (length as int - 1) + 1 <= proposed_values171(length).padded_size,
{
    proposed_accepted171(length);
    derive_original_cell171(length, 0, proposed_values171(length));
    derive_original_cell171(length, (length - 1) as usize, proposed_values171(length));
}

}
