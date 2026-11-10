use vstd::prelude::*;
verus! {
pub const TAG_MASK: u64 = u64::MAX >> 3;
pub const PERM_MASK: u64 = 0x3 << 61;
pub const PROTECTED_MASK: u64 = 0x1 << 63;
pub open spec fn item_word_wf(w: u64) -> bool { w & TAG_MASK != 0 }
pub open spec fn packed(t: u64, p: u64, protected: bool) -> u64 {
    t | (p << 61) | ((if protected { 1u64 } else { 0u64 }) << 63)
}
pub open spec fn updated(w: u64, p: u64) -> u64 {
    (w & !PERM_MASK) | (p << 61)
}
pub proof fn packed_fields(t: u64, p: u64, protected: bool)
    requires 0 < t <= TAG_MASK, p <= 3,
    ensures item_word_wf(packed(t,p,protected)),
        packed(t,p,protected) & TAG_MASK == t,
        (packed(t,p,protected) & PERM_MASK) >> 61 == p,
        (packed(t,p,protected) & PROTECTED_MASK > 0) == protected,
{
    assert(TAG_MASK == 0x1fffffffffffffffu64) by(bit_vector);
    assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
    assert(PROTECTED_MASK == 0x8000000000000000u64) by(bit_vector);
    let flag = if protected { 1u64 } else { 0u64 };
    assert((t | (p << 61u64) | (flag << 63u64)) & 0x1fffffffffffffffu64 == t) by(bit_vector)
        requires t <= 0x1fffffffffffffffu64, p <= 3u64, flag <= 1u64;
    assert(((t | (p << 61u64) | (flag << 63u64)) & 0x6000000000000000u64) >> 61u64 == p) by(bit_vector)
        requires t <= 0x1fffffffffffffffu64, p <= 3u64, flag <= 1u64;
    assert(((t | (p << 61u64) | (flag << 63u64)) & 0x8000000000000000u64 > 0u64) == (flag == 1u64)) by(bit_vector)
        requires t <= 0x1fffffffffffffffu64, p <= 3u64, flag <= 1u64;
}
pub proof fn updated_fields(w: u64, p: u64)
    requires p <= 3,
    ensures updated(w,p) & TAG_MASK == w & TAG_MASK,
        updated(w,p) & PROTECTED_MASK == w & PROTECTED_MASK,
        item_word_wf(updated(w,p)) == item_word_wf(w),
        (updated(w,p) & PERM_MASK) >> 61 == p,
{
    assert(TAG_MASK == 0x1fffffffffffffffu64) by(bit_vector);
    assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
    assert(PROTECTED_MASK == 0x8000000000000000u64) by(bit_vector);
    assert(((w & !0x6000000000000000u64) | (p << 61u64)) & 0x1fffffffffffffffu64 == w & 0x1fffffffffffffffu64) by(bit_vector)
        requires p <= 3u64;
    assert(((w & !0x6000000000000000u64) | (p << 61u64)) & 0x8000000000000000u64 == w & 0x8000000000000000u64) by(bit_vector)
        requires p <= 3u64;
    assert((((w & !0x6000000000000000u64) | (p << 61u64)) & 0x6000000000000000u64) >> 61u64 == p) by(bit_vector)
        requires p <= 3u64;
}
pub proof fn tag_range(t: u64)
    ensures (t & TAG_MASK == t) == (t <= TAG_MASK),
        (t & TAG_MASK) <= TAG_MASK,
{
    assert(TAG_MASK == 0x1fffffffffffffffu64) by(bit_vector);
    assert((t & 0x1fffffffffffffffu64 == t) == (t <= 0x1fffffffffffffffu64)) by(bit_vector);
    assert((t & 0x1fffffffffffffffu64) <= 0x1fffffffffffffffu64) by(bit_vector);
}
pub proof fn packed_fields_raw(t: u64, p: u64, raw: u64)
    requires 0 < t <= TAG_MASK, p <= 3,
    ensures item_word_wf(t | (p << 61) | (raw << 63)),
        (t | (p << 61) | (raw << 63)) & TAG_MASK == t,
        ((t | (p << 61) | (raw << 63)) & PERM_MASK) >> 61 == p,
{
    assert(TAG_MASK == 0x1fffffffffffffffu64) by(bit_vector);
    assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
    assert((t | (p << 61u64) | (raw << 63u64)) & 0x1fffffffffffffffu64 == t) by(bit_vector)
        requires t <= 0x1fffffffffffffffu64, p <= 3u64;
    assert(((t | (p << 61u64) | (raw << 63u64)) & 0x6000000000000000u64) >> 61u64 == p) by(bit_vector)
        requires t <= 0x1fffffffffffffffu64, p <= 3u64;
}
}
