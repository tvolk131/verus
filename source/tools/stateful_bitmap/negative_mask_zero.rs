use vstd::prelude::*;
verus! {
pub open spec fn live(data: Seq<u64>, index: int) -> bool {
    data[index / 64] & (1u64 << ((index % 64) as u64)) != 0
}
pub(crate) struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}
impl U64GroupedBitmap {
    pub closed spec fn model_len(&self) -> u32 {
        self.len
    }
    pub closed spec fn words(&self) -> Seq<u64> {
        self.data@
    }
    pub closed spec fn well_formed(&self) -> bool {
        self.len as int <= 64 * self.data@.len()
    }
    fn data_index_of(bit: u32) -> (result: (usize, usize))
ensures result.0 == bit as int / 64, result.1 == bit as int % 64,
        result.1 < 64,
{
        ((bit as usize) / 64, (bit as usize) % 64)
    }

    fn select_mask(bit: usize) -> (result: u64)
requires bit < 64,
    ensures result == 0u64,
{
        1u64 << (bit as u64)
    }

    pub fn len(&self) -> (result: u32)
ensures result == self.model_len(),
{
        self.len
    }

    pub fn get(&self, bit: u32) -> (result: bool)
requires self.well_formed(), bit < self.model_len(),
    ensures result == live(self.words(), bit as int),
{
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let group = self.data[index];
        group & U64GroupedBitmap::select_mask(bit_index) != 0
    }

    pub fn set(&mut self, bit: u32) -> (result: bool)
requires old(self).well_formed(), bit < old(self).model_len(),
    ensures final(self).well_formed(), final(self).model_len() == old(self).model_len(),
        final(self).words() == old(self).words().update(bit as int / 64,
            old(self).words()[bit as int / 64] | (1u64 << ((bit as int % 64) as u64))),
        result == (final(self).words()[bit as int / 64] == u64::MAX),
{
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let mut group = self.data[index];
        group |= Self::select_mask(bit_index);
        self.data[index] = group;

        group == u64::MAX
    }

    pub fn clear(&mut self, bit: u32)
requires old(self).well_formed(), bit < old(self).model_len(),
    ensures final(self).well_formed(), final(self).model_len() == old(self).model_len(),
        final(self).words() == old(self).words().update(bit as int / 64,
            old(self).words()[bit as int / 64] & !(1u64 << ((bit as int % 64) as u64))),
{
        assert!(bit < self.len, "{bit} must be less than {}", self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        self.data[index] &= !Self::select_mask(bit_index);
    }
}
fn bits_in_range(lo: u32, hi: u32) -> (result: u64)
requires lo < hi, hi <= 64,
    ensures result == (u64::MAX << lo) & (u64::MAX >> ((64 - hi) as u32)),
{
    debug_assert!(lo < hi && hi <= 64);
    // Both shifts are well-defined: `lo < 64` and `64 - hi < 64`.
    let bits_at_or_above_lo = u64::MAX << lo;
    let bits_below_hi = u64::MAX >> (64 - hi);
    bits_at_or_above_lo & bits_below_hi
}
}
