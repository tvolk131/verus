use vstd::prelude::*;

verus! {
pub open spec fn live(data: Seq<u64>, index: int) -> bool {
    data[index / 64] & (1u64 << ((index % 64) as u64)) != 0
}

#[verifier::infer]
pub fn bits_in_range(lo: u32, hi: u32) -> u64 {
    debug_assert!(lo < hi && hi <= 64);
    let bits_at_or_above_lo = u64::MAX << lo;
    let bits_below_hi = u64::MAX >> (64 - hi);
    bits_at_or_above_lo & bits_below_hi
}

pub proof fn mask_transition(value: u64, lo: u32, hi: u32, offset: u64, full: bool)
    requires lo < hi, hi <= 64, offset < 64,
    ensures
        ((if full {
            value | ((u64::MAX << lo) & (u64::MAX >> ((64 - hi) as u32)))
        } else {
            value & !((u64::MAX << lo) & (u64::MAX >> ((64 - hi) as u32)))
        }) & (1u64 << offset) != 0) ==
        (if lo as u64 <= offset && offset < hi as u64 { full }
         else { value & (1u64 << offset) != 0 }),
{
    assert(
        ((if full {
            value | ((u64::MAX << lo) & (u64::MAX >> ((64 - hi) as u32)))
        } else {
            value & !((u64::MAX << lo) & (u64::MAX >> ((64 - hi) as u32)))
        }) & (1u64 << offset) != 0) ==
        (if lo as u64 <= offset && offset < hi as u64 { full }
         else { value & (1u64 << offset) != 0 })
    ) by(bit_vector)
        requires lo < hi, hi <= 64, offset < 64;
}

pub fn native_step(data: &mut Vec<u64>, Ghost(entry): Ghost<Seq<u64>>,
    old_len: u32, new_len: u32, word: u32, full: bool)
    requires
        old_len < new_len,
        old_len as int <= 64 * entry.len(),
        old(data)@.len() as int == if entry.len() >= (new_len as int + 63) / 64 {
            entry.len() as int
        } else { (new_len as int + 63) / 64 },
        old_len / 64 <= word <= (new_len - 1) / 64,
        forall|query: int| 0 <= query < 64 * old(data)@.len() ==>
            #[trigger] live(old(data)@, query) ==
                (if old_len <= query < new_len && query < 64 * word as int { full }
                 else if query < 64 * entry.len() { live(entry, query) } else { full }),
    ensures
        final(data)@.len() == old(data)@.len(),
        forall|query: int| 0 <= query < 64 * final(data)@.len() ==>
            #[trigger] live(final(data)@, query) ==
                (if old_len <= query < new_len && query < 64 * (word as int + 1) { full }
                 else if query < 64 * entry.len() { live(entry, query) } else { full }),
{
    let ghost before = data@;
    let word_first_bit = word * 64;
    let lo = old_len.saturating_sub(word_first_bit);
    let hi = (new_len - word_first_bit).min(64);
    let mask = bits_in_range(lo, hi);
    let slot = &mut data[(word as usize) + 1];
    if full {
        *slot |= mask;
    } else {
        *slot &= !mask;
    }
    assert forall|query: int| 0 <= query < 64 * data@.len() implies
        live(data@, query) ==
            (if old_len <= query < new_len && query < 64 * (word as int + 1) { full }
             else if query < 64 * entry.len() { live(entry, query) } else { full })
    by {
        assert(live(before, query) ==
            (if old_len <= query < new_len && query < 64 * word as int { full }
             else if query < 64 * entry.len() { live(entry, query) } else { full }));
        if query / 64 == word {
            mask_transition(before[word as int], lo, hi, (query % 64) as u64, full);
        }
    }
}
}
