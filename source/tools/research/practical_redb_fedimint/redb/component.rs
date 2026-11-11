use vstd::prelude::*;
verus! {
pub open spec fn words(elements: u32) -> nat { (elements as nat + 63) / 64 }
pub open spec fn physical_bit(data: Seq<u64>, j: int) -> bool {
    (data[j / 64] & (1u64 << ((j % 64) as u64))) != 0
}
pub(crate) struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}
fn bits_in_range(lo: u32, hi: u32) -> (result: u64)
    requires lo < hi, hi <= 64,
    ensures forall|j: u32| j < 64 ==> ((result & (1u64 << j)) != 0) == (lo <= j < hi),
{
    debug_assert!(lo < hi && hi <= 64);
    // Both shifts are well-defined: `lo < 64` and `64 - hi < 64`.
    let bits_at_or_above_lo = u64::MAX << lo;
    let bits_below_hi = u64::MAX >> (64 - hi);
    bits_at_or_above_lo & bits_below_hi
}
impl U64GroupedBitmap {
fn required_words(elements: u32) -> (result: usize)
    ensures result == words(elements),
{
        let words = elements.div_ceil(64);
        words as usize
    }
pub closed spec fn logical_len(&self) -> u32 { self.len }
pub closed spec fn storage(&self) -> Seq<u64> { self.data@ }
pub fn resize(&mut self, new_len: u32, full: bool)
    requires words(old(self).logical_len()) <= old(self).storage().len(),
    ensures
        final(self).logical_len() == new_len,
        final(self).storage().len() == if old(self).storage().len() >= words(new_len) {
            old(self).storage().len()
        } else { words(new_len) },
        words(final(self).logical_len()) <= final(self).storage().len(),
        forall|j: int| 0 <= j < old(self).logical_len() && j < new_len ==>
            physical_bit(final(self).storage(), j) == physical_bit(old(self).storage(), j),
        forall|j: int| old(self).logical_len() <= j < new_len ==>
            physical_bit(final(self).storage(), j) == full,
        forall|j: int| 0 <= j < old(self).storage().len() * 64 && !(old(self).logical_len() <= j < new_len) ==>
            physical_bit(final(self).storage(), j) == physical_bit(old(self).storage(), j),
{
        if self.data.len() < Self::required_words(new_len) {
            let default_value = if full { u64::MAX } else { 0 };
            self.data
                .resize(Self::required_words(new_len), default_value);
        }
        let old_len = self.len;
        self.len = new_len;
        if old_len >= new_len {
            return;
        }
        // Apply `full` to bits [old_len, new_len) one word at a time. For each
        // word, build a mask of just the bits in that range that fall within
        // the word, then OR it in (full=true) or AND its complement (full=false).
        let start_word = old_len / 64;
        let end_word = (new_len - 1) / 64;
        for word in start_word..=end_word {
            let word_first_bit = word * 64;
            let lo = old_len.saturating_sub(word_first_bit);
            let hi = (new_len - word_first_bit).min(64);
            let mask = bits_in_range(lo, hi);
            let slot = &mut self.data[word as usize];
            if full {
                *slot |= mask;
            } else {
                *slot &= !mask;
            }
        }
    }
}
}
fn main() {}
