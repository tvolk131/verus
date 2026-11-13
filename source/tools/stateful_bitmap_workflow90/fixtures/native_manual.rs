use vstd::prelude::*;

verus! {

pub open spec fn live(data: Seq<u64>, index: int) -> bool {
    data[index / 64] & (1u64 << ((index % 64) as u64)) != 0
}

pub open spec fn word_count(before: nat, length: u32) -> int {
    if before >= (length as int + 63) / 64 { before as int }
    else { (length as int + 63) / 64 }
}

pub open spec fn resized(before: Seq<u64>, old_len: u32, after: Seq<u64>, new_len: u32, full: bool) -> bool {
    &&& after.len() == word_count(before.len(), new_len)
    &&& new_len as int <= 64 * after.len()
    &&& forall|query: int| 0 <= query < 64 * after.len() ==>
        #[trigger] live(after, query) ==
            (if old_len <= query < new_len { full }
             else if query < 64 * before.len() { live(before, query) } else { full })
    &&& forall|word: int| before.len() <= word < after.len() ==>
        #[trigger] after[word] == if full { u64::MAX } else { 0u64 }
    &&& (new_len <= old_len ==> after == before)
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
    ) by(bit_vector) requires lo < hi, hi <= 64, offset < 64;
}

pub proof fn default_word(full: bool, offset: u64, mask: u64)
    requires offset < 64,
    ensures
        ((if full { u64::MAX } else { 0u64 }) & (1u64 << offset) != 0) == full,
        (if full { u64::MAX | mask } else { 0u64 & !mask }) ==
            (if full { u64::MAX } else { 0u64 }),
{
    assert(
        ((if full { u64::MAX } else { 0u64 }) & (1u64 << offset) != 0) == full
    ) by(bit_vector) requires offset < 64;
    assert(
        (if full { u64::MAX | mask } else { 0u64 & !mask }) ==
            (if full { u64::MAX } else { 0u64 })
    ) by(bit_vector);
}

pub proof fn access_word_update(value: u64, selected: u64, query: u64)
    requires selected < 64, query < 64,
    ensures
        ((value | (1u64 << selected)) & (1u64 << query) != 0) ==
            (selected == query || value & (1u64 << query) != 0),
        ((value & !(1u64 << selected)) & (1u64 << query) != 0) ==
            (selected != query && value & (1u64 << query) != 0),
{
    assert(
        ((value | (1u64 << selected)) & (1u64 << query) != 0) ==
            (selected == query || value & (1u64 << query) != 0)
    ) by(bit_vector) requires selected < 64, query < 64;
    assert(
        ((value & !(1u64 << selected)) & (1u64 << query) != 0) ==
            (selected != query && value & (1u64 << query) != 0)
    ) by(bit_vector) requires selected < 64, query < 64;
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

pub(crate) struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}

impl U64GroupedBitmap {
    pub closed spec fn model_len(&self) -> u32 { self.len }
    pub closed spec fn words(&self) -> Seq<u64> { self.data@ }
    pub closed spec fn well_formed(&self) -> bool {
        self.len as int <= 64 * self.data@.len()
    }

    fn required_words(elements: u32) -> (result: usize)
        ensures result == (elements as int + 63) / 64,
    {
        let words = elements.div_ceil(64);
        words as usize
    }

    fn data_index_of(bit: u32) -> (result: (usize, usize))
        ensures result.0 == bit as int / 64, result.1 == bit as int % 64,
            result.1 < 64,
    {
        ((bit as usize) / 64, (bit as usize) % 64)
    }

    fn select_mask(bit: usize) -> (result: u64)
        requires bit < 64,
        ensures result == 1u64 << (bit as u64),
    {
        1u64 << (bit as u64)
    }

    pub fn len(&self) -> (result: u32)
        ensures result == self.model_len(),
    {
        self.len
    }

    pub fn resize(&mut self, new_len: u32, full: bool)
        requires old(self).well_formed(),
        ensures final(self).well_formed(), final(self).model_len() == new_len,
            resized(old(self).words(), old(self).model_len(), final(self).words(), new_len, full),
    {
        let ghost entry = self.data@;
        let ghost entry_len = self.len;
        if self.data.len() < Self::required_words(new_len) {
            let default_value = if full { u64::MAX } else { 0 };
            self.data
                .resize(Self::required_words(new_len), default_value);
        }
        let old_len = self.len;
        self.len = new_len;
        proof {
            assert(self.data@.len() == word_count(entry.len(), new_len));
            assert(new_len as int <= 64 * self.data@.len());
            assert forall|query: int| 0 <= query < 64 * self.data@.len() implies
                #[trigger] live(self.data@, query) ==
                    (if query < 64 * entry.len() { live(entry, query) } else { full })
            by {
                if query / 64 >= entry.len() {
                    default_word(full, (query % 64) as u64, 0);
                }
            }
        }
        if old_len >= new_len {
            return;
        }
        // Apply `full` to bits [old_len, new_len) one word at a time. For each
        // word, build a mask of just the bits in that range that fall within
        // the word, then OR it in (full=true) or AND its complement (full=false).
        let start_word = old_len / 64;
        let end_word = (new_len - 1) / 64;
        for word in cursor: start_word..=end_word
            invariant
                old_len == entry_len,
                old_len < new_len,
                old_len as int <= 64 * entry.len(),
                start_word == old_len / 64,
                end_word == (new_len - 1) / 64,
                self.len == new_len,
                self.data@.len() == word_count(entry.len(), new_len),
                new_len as int <= 64 * self.data@.len(),
                0 <= cursor.index() <= end_word as int - start_word as int + 1,
                forall|fresh: int| entry.len() <= fresh < self.data@.len() ==>
                    #[trigger] self.data@[fresh] == if full { u64::MAX } else { 0u64 },
                forall|query: int| 0 <= query < 64 * self.data@.len() ==>
                    #[trigger] live(self.data@, query) ==
                        (if old_len <= query < new_len &&
                            query < 64 * (start_word as int + cursor.index()) { full }
                         else if query < 64 * entry.len() { live(entry, query) } else { full }),
        {
            let ghost before = self.data@;
            proof {
                assert(word as int == start_word as int + cursor.index());
            }
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
            proof {
                assert(self.data@ == before.update(word as int,
                    if full { before[word as int] | mask } else { before[word as int] & !mask }));
                assert forall|fresh: int| entry.len() <= fresh < self.data@.len() implies
                    #[trigger] self.data@[fresh] == if full { u64::MAX } else { 0u64 }
                by {
                    if fresh == word {
                        default_word(full, 0, mask);
                    }
                }
                assert forall|query: int| 0 <= query < 64 * self.data@.len() implies
                    #[trigger] live(self.data@, query) ==
                        (if old_len <= query < new_len && query < 64 * (word as int + 1) { full }
                         else if query < 64 * entry.len() { live(entry, query) } else { full })
                by {
                    assert(live(before, query) ==
                        (if old_len <= query < new_len && query < 64 * word as int { full }
                         else if query < 64 * entry.len() { live(entry, query) } else { full }));
                    if query / 64 == word {
                        assert((lo as int <= query % 64 < hi as int) == (old_len <= query < new_len));
                        mask_transition(before[word as int], lo, hi, (query % 64) as u64, full);
                    } else {
                        assert(live(self.data@, query) == live(before, query));
                        assert((query < 64 * (word as int + 1)) == (query < 64 * word as int));
                    }
                }
            }
        }
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

pub fn resize_read(data: Vec<u64>, old_len: u32, new_len: u32, full: bool, query: u32)
    -> (result: (Vec<u64>, bool, u32))
    requires old_len as int <= 64 * data@.len(), query < new_len,
    ensures resized(data@, old_len, result.0@, new_len, full), result.2 == new_len,
        result.1 == if old_len <= query { full } else { live(data@, query as int) },
{
    let mut bitmap = U64GroupedBitmap { len: old_len, data };
    bitmap.resize(new_len, full);
    let observed = bitmap.get(query);
    let length = bitmap.len();
    (bitmap.data, observed, length)
}

pub fn set_get_clear(data: Vec<u64>, length: u32, bit: u32) -> (result: (Vec<u64>, bool, bool))
    requires length as int <= 64 * data@.len(), bit < length,
    ensures result.1, !result.2, result.0@.len() == data@.len(),
        forall|query: int| 0 <= query < 64 * result.0@.len() ==>
            #[trigger] live(result.0@, query) == (query != bit && live(data@, query)),
{
    let ghost entry = data@;
    let mut bitmap = U64GroupedBitmap { len: length, data };
    bitmap.set(bit);
    let after_set = bitmap.get(bit);
    let ghost middle = bitmap.words();
    proof {
        access_word_update(entry[bit as int / 64], (bit % 64) as u64, (bit % 64) as u64);
        assert(after_set);
    }
    bitmap.clear(bit);
    let after_clear = bitmap.get(bit);
    proof {
        access_word_update(middle[bit as int / 64], (bit % 64) as u64, (bit % 64) as u64);
        assert(!after_clear);
        assert forall|query: int| 0 <= query < 64 * bitmap.words().len() implies
            #[trigger] live(bitmap.words(), query) == (query != bit && live(entry, query))
        by {
            if query / 64 == bit as int / 64 {
                access_word_update(entry[bit as int / 64], (bit % 64) as u64, (query % 64) as u64);
                access_word_update(middle[bit as int / 64], (bit % 64) as u64, (query % 64) as u64);
            }
        }
    }
    (bitmap.data, after_set, after_clear)
}

pub fn shrink_regrow(data: Vec<u64>, old_len: u32, small: u32, new_len: u32, full: bool, query: u32)
    -> (result: (Vec<u64>, bool, u32))
    requires old_len as int <= 64 * data@.len(), small <= old_len, small <= new_len, query < new_len,
    ensures resized(data@, small, result.0@, new_len, full), result.2 == new_len,
        result.1 == if small <= query { full } else { live(data@, query as int) },
{
    let mut bitmap = U64GroupedBitmap { len: old_len, data };
    bitmap.resize(small, !full);
    bitmap.resize(new_len, full);
    let observed = bitmap.get(query);
    let length = bitmap.len();
    (bitmap.data, observed, length)
}

pub fn resize_update(data: Vec<u64>, old_len: u32, new_len: u32, full: bool, bit: u32)
    -> (result: (Vec<u64>, bool, bool, u32))
    requires old_len as int <= 64 * data@.len(), bit < new_len,
    ensures result.1, !result.2, result.3 == new_len,
        result.0@.len() == word_count(data@.len(), new_len),
        forall|query: int| 0 <= query < 64 * result.0@.len() ==>
            #[trigger] live(result.0@, query) ==
                (query != bit && (if old_len <= query < new_len { full }
                 else if query < 64 * data@.len() { live(data@, query) } else { full })),
{
    let ghost original = data@;
    let mut bitmap = U64GroupedBitmap { len: old_len, data };
    bitmap.resize(new_len, full);
    let ghost entry = bitmap.words();
    bitmap.set(bit);
    let after_set = bitmap.get(bit);
    let ghost middle = bitmap.words();
    proof {
        access_word_update(entry[bit as int / 64], (bit % 64) as u64, (bit % 64) as u64);
        assert(after_set);
    }
    bitmap.clear(bit);
    let after_clear = bitmap.get(bit);
    let length = bitmap.len();
    proof {
        access_word_update(middle[bit as int / 64], (bit % 64) as u64, (bit % 64) as u64);
        assert(!after_clear);
        assert forall|query: int| 0 <= query < 64 * bitmap.words().len() implies
            #[trigger] live(bitmap.words(), query) ==
                (query != bit && (if old_len <= query < new_len { full }
                 else if query < 64 * original.len() { live(original, query) } else { full }))
        by {
            assert(live(entry, query) ==
                (if old_len <= query < new_len { full }
                 else if query < 64 * original.len() { live(original, query) } else { full }));
            if query / 64 == bit as int / 64 {
                access_word_update(entry[bit as int / 64], (bit % 64) as u64, (query % 64) as u64);
                access_word_update(middle[bit as int / 64], (bit % 64) as u64, (query % 64) as u64);
            }
        }
    }
    (bitmap.data, after_set, after_clear, length)
}

}
