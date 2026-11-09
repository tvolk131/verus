use vstd::prelude::*;

verus! {
pub const MAX_PAGE_INDEX: u32 = 0x000F_FFFF;

pub struct U64GroupedBitmap { len: u32, data: Vec<u64> }
pub struct BtreeBitmap { heights: Vec<U64GroupedBitmap> }

pub open spec fn bit(words: Seq<u64>, index: int) -> bool {
    ((words[index / 64] >> ((index % 64) as u64)) & 1u64) == 1u64
}

pub proof fn word_bit(value: u64, offset: u64)
    requires offset < 64,
    ensures ((value >> offset) & 1u64 == 1u64) == (value & (1u64 << offset) != 0),
        ((u64::MAX >> offset) & 1u64) == 1u64,
{
    assert(((value >> offset) & 1u64 == 1u64) == (value & (1u64 << offset) != 0))
        by(bit_vector) requires offset < 64;
    assert(((u64::MAX >> offset) & 1u64) == 1u64) by(bit_vector) requires offset < 64;
}

pub proof fn word_update(value: u64, selected: u64, query: u64)
    requires selected < 64, query < 64,
    ensures
        (((value | (1u64 << selected)) >> query) & 1u64 == 1u64) ==
            (selected == query || ((value >> query) & 1u64 == 1u64)),
        (((value & !(1u64 << selected)) >> query) & 1u64 == 1u64) ==
            (selected != query && ((value >> query) & 1u64 == 1u64)),
{
    assert((((value | (1u64 << selected)) >> query) & 1u64 == 1u64) ==
            (selected == query || ((value >> query) & 1u64 == 1u64)))
        by(bit_vector) requires selected < 64, query < 64;
    assert((((value & !(1u64 << selected)) >> query) & 1u64 == 1u64) ==
            (selected != query && ((value >> query) & 1u64 == 1u64)))
        by(bit_vector) requires selected < 64, query < 64;
}

impl U64GroupedBitmap {
    pub closed spec fn words(&self) -> Seq<u64> { self.data@ }
    pub closed spec fn model_len(&self) -> u32 { self.len }
    pub open spec fn covered(&self) -> bool { self.model_len() as int <= 64 * self.words().len() }
    pub open spec fn padded(&self) -> bool {
        forall|query: int| self.model_len() <= query < ((self.model_len() as int + 63) / 64) * 64
            ==> #[trigger] bit(self.words(), query)
    }
    pub fn data_index_of(bit: u32) -> (result: (usize, usize))
        ensures result.0 == bit as int / 64, result.1 == bit as int % 64, result.1 < 64,
    {
        ((bit as usize) / 64, (bit as usize) % 64)
    }
    pub fn select_mask(bit: usize) -> (result: u64)
        requires bit < 64,
        ensures result == 1u64 << (bit as u64),
    {
        1u64 << (bit as u64)
    }
    pub fn first_unset(&self, start_bit: u32, end_bit: u32) -> (result: Option<u32>)
        requires start_bit % 64 == 0,
            start_bit as int + 64 <= u32::MAX,
            end_bit as int == start_bit as int + 64,
            self.model_len() != 0 ==> start_bit as int / 64 < self.words().len(),
        ensures result.is_some() ==> start_bit <= result.unwrap() < end_bit,
            result.is_none() <==> (self.model_len() == 0 || self.words()[start_bit as int / 64] == u64::MAX),
            result.is_some() ==> !bit(self.words(), result.unwrap() as int),
            result.is_some() ==> forall|query: int| start_bit <= query < result.unwrap()
                ==> #[trigger] bit(self.words(), query),
    {
        assert_eq!(end_bit, (start_bit - start_bit % 64) + 64);
        if self.len == 0 {
            return None;
        }

        let (index, bit) = Self::data_index_of(start_bit);
        proof { assert(bit == 0); assert((1u64 << bit) == 1u64) by(bit_vector) requires bit == 0; }
        let mask = (1 << bit) - 1;
        proof { assert(mask == 0); }
        let group = self.data[index];
        proof { assert(group | 0u64 == group) by(bit_vector); }
        let group = group | mask;
        proof {
            assert(group == self.data@[index as int]);
            let count = group.trailing_ones();
            assert forall|query: int| start_bit <= query < start_bit as int + count
                implies #[trigger] crate::bit(self.words(), query) by {
                assert(query / 64 == index);
                assert(query % 64 == query - start_bit);
            }
            if count < 64 {
                assert((start_bit as int + count) / 64 == index);
                assert((start_bit as int + count) % 64 == count);
            }
        }
        match group.trailing_ones() {
            64 => None,
            x => Some(start_bit + x - u32::try_from(bit).unwrap()),
        }
    }
    pub fn set(&mut self, bit: u32) -> (result: bool)
        requires old(self).covered(), bit < old(self).model_len(),
        ensures final(self).covered(), final(self).model_len() == old(self).model_len(),
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
        requires old(self).covered(), bit < old(self).model_len(),
        ensures final(self).covered(), final(self).model_len() == old(self).model_len(),
            final(self).words() == old(self).words().update(bit as int / 64,
                old(self).words()[bit as int / 64] & !(1u64 << ((bit as int % 64) as u64))),
    {
        assert!(bit < self.len, "{bit} must be less than {}", self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        self.data[index] &= !Self::select_mask(bit_index);
    }
}

impl BtreeBitmap {
    pub closed spec fn levels(&self) -> Seq<U64GroupedBitmap> { self.heights@ }
    pub fn get_height(&self) -> (result: u32)
        requires self.levels().len() <= u32::MAX,
        ensures result == self.levels().len(),
    {
        self.heights.len().try_into().unwrap()
    }
    pub fn get_level(&self, i: u32) -> (result: &U64GroupedBitmap)
        requires self.levels().len() <= u32::MAX, i < self.levels().len(),
        ensures *result == self.levels()[i as int],
    {
        assert!(i < self.get_height());
        &self.heights[i as usize]
    }
    pub fn get_level_mut(&mut self, i: u32) -> (result: &mut U64GroupedBitmap)
        requires old(self).levels().len() <= u32::MAX, i < old(self).levels().len(),
        ensures *result == old(self).levels()[i as int],
            final(self).levels() == old(self).levels().update(i as int, *final(result)),
    {
        assert!(i < self.get_height());
        &mut self.heights[i as usize]
    }
}
pub open spec fn prefix_full(group: U64GroupedBitmap, limit: int) -> bool {
    forall|query: int| 0 <= query < limit ==> #[trigger] bit(group.words(), query)
}

pub open spec fn shape(levels: Seq<U64GroupedBitmap>) -> bool {
    &&& 1 <= levels.len() <= u32::MAX
    &&& levels[0].model_len() <= 64
    &&& levels.last().model_len() <= MAX_PAGE_INDEX + 1
    &&& forall|height: int| #![trigger levels[height]] 0 <= height < levels.len() ==>
        (levels[height].covered() && levels[height].padded())
    &&& forall|height: int| 0 <= height < levels.len() - 1 ==>
        #[trigger] levels[height].model_len() as int == (levels[height + 1].model_len() as int + 63) / 64
}

pub open spec fn rep(levels: Seq<U64GroupedBitmap>) -> bool {
    &&& shape(levels)
    &&& forall|height: int, entry: int|
        0 <= height < levels.len() - 1 && 0 <= entry < levels[height].model_len() ==>
        #[trigger] bit(levels[height].words(), entry) == (levels[height + 1].words()[entry] == u64::MAX)
}

pub proof fn length_bound(levels: Seq<U64GroupedBitmap>, height: int)
    requires rep(levels), 0 <= height < levels.len(),
    ensures levels[height].model_len() <= MAX_PAGE_INDEX + 1,
    decreases levels.len() - height,
{
    if height < levels.len() - 1 {
        length_bound(levels, height + 1);
        assert(levels[height].model_len() as int == (levels[height + 1].model_len() as int + 63) / 64);
        assert((levels[height + 1].model_len() as int + 63) / 64 <= levels[height + 1].model_len());
    }
}

pub proof fn full_word(words: Seq<u64>, index: int)
    requires 0 <= index < 64 * words.len(), words[index / 64] == u64::MAX,
    ensures bit(words, index),
{
    word_bit(u64::MAX, (index % 64) as u64);
}

pub proof fn prefix_down(levels: Seq<U64GroupedBitmap>, height: int, limit: int)
    requires rep(levels), 0 <= height < levels.len() - 1,
        0 <= limit <= levels[height].model_len(), prefix_full(levels[height], limit),
    ensures forall|query: int| 0 <= query < levels[height + 1].model_len() && query < 64 * limit
        ==> #[trigger] bit(levels[height + 1].words(), query),
{
    assert(levels[height + 1].covered());
    assert forall|query: int| 0 <= query < levels[height + 1].model_len() && query < 64 * limit
        implies #[trigger] bit(levels[height + 1].words(), query) by {
        assert(query / 64 < limit);
        assert(bit(levels[height].words(), query / 64));
        assert(levels[height + 1].words()[query / 64] == u64::MAX);
        full_word(levels[height + 1].words(), query);
    }
}

pub proof fn full_to_leaf(levels: Seq<U64GroupedBitmap>, height: int)
    requires rep(levels), 0 <= height < levels.len(),
        prefix_full(levels[height], levels[height].model_len() as int),
    ensures prefix_full(levels.last(), levels.last().model_len() as int),
    decreases levels.len() - height,
{
    if height < levels.len() - 1 {
        prefix_down(levels, height, levels[height].model_len() as int);
        assert(levels[height + 1].model_len() <= 64 * levels[height].model_len());
        assert(prefix_full(levels[height + 1], levels[height + 1].model_len() as int));
        full_to_leaf(levels, height + 1);
    }
}

pub proof fn selected_valid(group: U64GroupedBitmap, start: int, selected: int)
    requires group.covered(), group.padded(), group.model_len() > 0,
        start % 64 == 0, 0 <= start < group.model_len(),
        start <= selected < start + 64, !bit(group.words(), selected),
    ensures selected < group.model_len(),
{
    assert(selected < ((group.model_len() as int + 63) / 64) * 64);
    if selected >= group.model_len() {
        assert(bit(group.words(), selected));
    }
}

pub proof fn child_ready(levels: Seq<U64GroupedBitmap>, height: int, entry: int)
    requires rep(levels), 0 <= height < levels.len() - 1,
        0 <= entry < levels[height].model_len(),
        !bit(levels[height].words(), entry), prefix_full(levels[height], entry),
    ensures levels[height + 1].model_len() > 0,
        entry * 64 < levels[height + 1].model_len(),
        entry < levels[height + 1].words().len(),
        levels[height + 1].words()[entry] != u64::MAX,
        entry * 64 + 64 <= u32::MAX,
        forall|query: int| 0 <= query < entry * 64
            ==> #[trigger] bit(levels[height + 1].words(), query),
{
    length_bound(levels, height + 1);
    assert(levels[height + 1].covered());
    assert(levels[height].model_len() as int == (levels[height + 1].model_len() as int + 63) / 64);
    assert(entry * 64 < levels[height + 1].model_len());
    prefix_down(levels, height, entry);
}

impl BtreeBitmap {
    pub open spec fn invariant(&self) -> bool { rep(self.levels()) }

    pub fn find_first_unset(&self) -> (result: Option<u32>)
        requires self.invariant(),
        ensures result.is_some() ==> result.unwrap() < self.levels().last().model_len(),
            result.is_some() ==> !bit(self.levels().last().words(), result.unwrap() as int),
            result.is_some() ==> prefix_full(self.levels().last(), result.unwrap() as int),
            result.is_none() <==> prefix_full(self.levels().last(), self.levels().last().model_len() as int),
    {
        proof { assert(self.levels()[0].covered()); }
        if let Some(mut entry) = self.get_level(0).first_unset(0, 64) {
            let mut height = 0;
            proof {
                selected_valid(self.levels()[0], 0, entry as int);
                assert(prefix_full(self.levels()[0], entry as int));
            }

            while height < self.get_height() - 1
                invariant
                    self.invariant(), 0 <= height < self.levels().len(),
                    entry < self.levels()[height as int].model_len(),
                    !bit(self.levels()[height as int].words(), entry as int),
                    prefix_full(self.levels()[height as int], entry as int),
                decreases self.levels().len() - height,
            {
                proof { child_ready(self.levels(), height as int, entry as int); }
                height += 1;
                entry *= 64;
                let ghost start = entry;
                entry = self
                    .get_level(height)
                    .first_unset(entry, entry + 64)
                    .unwrap();
                proof {
                    selected_valid(self.levels()[height as int], start as int, entry as int);
                    assert forall|query: int| 0 <= query < entry
                        implies #[trigger] bit(self.levels()[height as int].words(), query) by {
                        if query < start { } else { }
                    }
                }
            }

            Some(entry)
        } else {
            proof {
                assert forall|query: int| 0 <= query < self.levels()[0].model_len()
                    implies #[trigger] bit(self.levels()[0].words(), query) by {
                    full_word(self.levels()[0].words(), query);
                }
                full_to_leaf(self.levels(), 0);
            }
            None
        }
    }
}

pub open spec fn edited(words: Seq<u64>, entry: int, value: bool) -> Seq<u64> {
    words.update(entry / 64, if value {
        words[entry / 64] | (1u64 << ((entry % 64) as u64))
    } else {
        words[entry / 64] & !(1u64 << ((entry % 64) as u64))
    })
}

pub open spec fn group_delta(before: U64GroupedBitmap, after: U64GroupedBitmap, entry: int, value: bool) -> bool {
    after.model_len() == before.model_len() && after.words() == edited(before.words(), entry, value)
}

pub proof fn group_edit(before: U64GroupedBitmap, after: U64GroupedBitmap, entry: int, value: bool)
    requires before.covered(), before.padded(), 0 <= entry < before.model_len(),
        group_delta(before, after, entry, value),
    ensures after.covered(), after.padded(), after.words().len() == before.words().len(),
        bit(after.words(), entry) == value,
        !value ==> after.words()[entry / 64] != u64::MAX,
        forall|query: int| 0 <= query < 64 * before.words().len() ==>
            #[trigger] bit(after.words(), query) == (if query == entry { value } else { bit(before.words(), query) }),
{
    assert(0 <= entry / 64 < before.words().len());
    assert forall|query: int| 0 <= query < 64 * before.words().len()
        implies #[trigger] bit(after.words(), query) == (if query == entry { value } else { bit(before.words(), query) }) by {
        if query / 64 == entry / 64 {
            word_update(before.words()[entry / 64], (entry % 64) as u64, (query % 64) as u64);
            assert((query % 64 == entry % 64) == (query == entry));
        }
    }
    assert forall|query: int| after.model_len() <= query < ((after.model_len() as int + 63) / 64) * 64
        implies #[trigger] bit(after.words(), query) by {
        assert(query != entry);
        assert(bit(before.words(), query));
    }
    if !value && after.words()[entry / 64] == u64::MAX {
        full_word(after.words(), entry);
    }
}

pub open spec fn ancestor(entry: int, steps: nat) -> int
    decreases steps,
{
    if steps == 0 { entry } else { ancestor(entry, (steps - 1) as nat) / 64 }
}

pub open spec fn path(entry: int, height_count: int, height: int) -> int {
    ancestor(entry, (height_count - 1 - height) as nat)
}

pub proof fn path_valid(levels: Seq<U64GroupedBitmap>, entry: int, height: int)
    requires shape(levels), 0 <= entry < levels.last().model_len(), 0 <= height < levels.len(),
    ensures 0 <= path(entry, levels.len() as int, height) < levels[height].model_len(),
        height < levels.len() - 1 ==> path(entry, levels.len() as int, height) == path(entry, levels.len() as int, height + 1) / 64,
        height == levels.len() - 1 ==> path(entry, levels.len() as int, height) == entry,
    decreases levels.len() - height,
{
    if height < levels.len() - 1 {
        path_valid(levels, entry, height + 1);
        assert(levels[height].model_len() as int == (levels[height + 1].model_len() as int + 63) / 64);
    }
}

pub open spec fn hierarchy_except(levels: Seq<U64GroupedBitmap>, entry: int, cutoff: int) -> bool {
    forall|height: int, query: int|
        0 <= height < levels.len() - 1 && 0 <= query < levels[height].model_len()
            && (height > cutoff || query != path(entry, levels.len() as int, height)) ==>
        #[trigger] bit(levels[height].words(), query) == (levels[height + 1].words()[query] == u64::MAX)
}

pub open spec fn repair_frame(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int, cutoff: int) -> bool {
    &&& after.len() == before.len()
    &&& forall|height: int| 0 <= height < before.len() ==>
        #[trigger] after[height].model_len() == before[height].model_len()
    &&& forall|height: int| 0 <= height < before.len() ==>
        #[trigger] after[height].words() ==
            if height == before.len() - 1 || height <= cutoff { before[height].words() }
            else { edited(before[height].words(), path(entry, before.len() as int, height),
                after[height + 1].words()[path(entry, before.len() as int, height)] == u64::MAX) }
}

pub proof fn shape_edit(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, height: int, entry: int, value: bool)
    requires shape(before), 0 <= height < before.len(), 0 <= entry < before[height].model_len(),
        after == before.update(height, after[height]), group_delta(before[height], after[height], entry, value),
    ensures shape(after),
{
    assert(before[height].covered() && before[height].padded());
    group_edit(before[height], after[height], entry, value);
    assert forall|other: int| 0 <= other < after.len()
        implies #[trigger] after[other].covered() && after[other].padded() by {
        if other != height { assert(after[other] == before[other]); }
    }
    assert forall|other: int| 0 <= other < after.len() - 1
        implies #[trigger] after[other].model_len() as int == (after[other + 1].model_len() as int + 63) / 64 by {
        assert(after[other].model_len() == before[other].model_len());
        assert(after[other + 1].model_len() == before[other + 1].model_len());
    }
}

pub proof fn repair_step(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, leaf: int, height: int, value: bool)
    requires shape(before), 0 <= leaf < before.last().model_len(), 0 <= height < before.len() - 1,
        hierarchy_except(before, leaf, height),
        after == before.update(height, after[height]),
        group_delta(before[height], after[height], path(leaf, before.len() as int, height), value),
        value == (before[height + 1].words()[path(leaf, before.len() as int, height)] == u64::MAX),
    ensures shape(after), hierarchy_except(after, leaf, height - 1),
{
    path_valid(before, leaf, height);
    let selected = path(leaf, before.len() as int, height);
    shape_edit(before, after, height, selected, value);
    assert(before[height].covered() && before[height].padded());
    group_edit(before[height], after[height], selected, value);
    if height > 0 { path_valid(before, leaf, height - 1); }
    assert forall|other: int, query: int|
        0 <= other < after.len() - 1 && 0 <= query < after[other].model_len()
            && (other > height - 1 || query != path(leaf, after.len() as int, other))
        implies #[trigger] bit(after[other].words(), query) == (after[other + 1].words()[query] == u64::MAX) by {
        if other == height {
            if query == selected {
                assert(bit(after[other].words(), query) == value);
            } else {
                assert(bit(before[other].words(), query) == (before[other + 1].words()[query] == u64::MAX));
                assert(bit(after[other].words(), query) == bit(before[other].words(), query));
            }
        } else if other + 1 == height {
            assert(query != selected / 64);
            assert(after[height].words()[query] == before[height].words()[query]);
            assert(bit(before[other].words(), query) == (before[other + 1].words()[query] == u64::MAX));
        } else {
            assert(after[other] == before[other]);
            assert(after[other + 1] == before[other + 1]);
            assert(bit(before[other].words(), query) == (before[other + 1].words()[query] == u64::MAX));
        }
    }
}

pub proof fn frame_step(original: Seq<U64GroupedBitmap>, before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, leaf: int, height: int, value: bool)
    requires shape(before), 0 <= leaf < before.last().model_len(), 0 <= height < before.len() - 1,
        repair_frame(original, before, leaf, height),
        after == before.update(height, after[height]),
        group_delta(before[height], after[height], path(leaf, before.len() as int, height), value),
        value == (before[height + 1].words()[path(leaf, before.len() as int, height)] == u64::MAX),
    ensures repair_frame(original, after, leaf, height - 1),
{
    assert forall|other: int| 0 <= other < original.len()
        implies #[trigger] after[other].model_len() == original[other].model_len() by {
        if other == height { } else { assert(after[other] == before[other]); }
    }
    assert forall|other: int| 0 <= other < original.len()
        implies #[trigger] after[other].words() ==
            if other == original.len() - 1 || other <= height - 1 { original[other].words() }
            else { edited(original[other].words(), path(leaf, original.len() as int, other),
                after[other + 1].words()[path(leaf, original.len() as int, other)] == u64::MAX) } by {
        if other == height {
            assert(before[height].words() == original[height].words());
        } else if other > height {
            assert(after[other] == before[other]);
            if other < original.len() - 1 { assert(after[other + 1] == before[other + 1]); }
        } else {
            assert(after[other] == before[other]);
        }
    }
}

impl BtreeBitmap {
    pub fn update_to_root(&mut self, i: u32, mut full: bool)
        requires shape(old(self).levels()), i < old(self).levels().last().model_len(),
            hierarchy_except(old(self).levels(), i as int, old(self).levels().len() as int - 2),
            full == (old(self).levels().last().words()[i as int / 64] == u64::MAX),
        ensures final(self).invariant(), repair_frame(old(self).levels(), final(self).levels(), i as int, -1),
    {
        if self.get_height() == 1 {
            return;
        }

        let ghost original = self.levels();
        let mut parent_height = self.get_height() - 2;
        let mut parent_entry = i / 64;
        proof { path_valid(self.levels(), i as int, parent_height as int); }
        loop
            invariant_except_break
                shape(self.levels()), self.levels().len() == original.len(),
                0 <= parent_height < self.levels().len() - 1,
                i < self.levels().last().model_len(),
                parent_entry == path(i as int, self.levels().len() as int, parent_height as int),
                parent_entry < self.levels()[parent_height as int].model_len(),
                full == (self.levels()[parent_height as int + 1].words()[parent_entry as int] == u64::MAX),
                hierarchy_except(self.levels(), i as int, parent_height as int),
                repair_frame(original, self.levels(), i as int, parent_height as int),
            ensures self.invariant(), repair_frame(original, self.levels(), i as int, -1),
            decreases parent_height + 1,
        {
            let ghost before = self.levels();
            let ghost value = full;
            proof {
                assert(shape(before));
                assert(before[parent_height as int].covered() && before[parent_height as int].padded());
            }
            full = if full {
                self.get_level_mut(parent_height).set(parent_entry)
            } else {
                self.get_level_mut(parent_height).clear(parent_entry);
                false
            };
            proof {
                assert(self.levels() == before.update(parent_height as int, self.levels()[parent_height as int]));
                assert(group_delta(before[parent_height as int], self.levels()[parent_height as int], parent_entry as int, value));
                assert(before[parent_height as int].covered() && before[parent_height as int].padded());
                group_edit(before[parent_height as int], self.levels()[parent_height as int], parent_entry as int, value);
                repair_step(before, self.levels(), i as int, parent_height as int, value);
                frame_step(original, before, self.levels(), i as int, parent_height as int, value);
            }

            if parent_height == 0 {
                break;
            }
            proof { path_valid(self.levels(), i as int, parent_height as int - 1); }
            parent_height -= 1;
            parent_entry /= 64;
        }
    }
}

pub open spec fn set_effect(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int) -> bool {
    &&& after.len() == before.len()
    &&& forall|height: int| 0 <= height < before.len() ==>
        #[trigger] after[height].model_len() == before[height].model_len()
    &&& after.last().words() == edited(before.last().words(), entry, true)
    &&& forall|height: int| 0 <= height < before.len() - 1 ==>
        #[trigger] after[height].words() == edited(before[height].words(), path(entry, before.len() as int, height),
            after[height + 1].words()[path(entry, before.len() as int, height)] == u64::MAX)
}

pub proof fn leaf_begin(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int)
    requires rep(before), 0 <= entry < before.last().model_len(),
        after == before.update(before.len() as int - 1, after.last()),
        group_delta(before.last(), after.last(), entry, true),
    ensures shape(after), hierarchy_except(after, entry, after.len() as int - 2),
        bit(after.last().words(), entry),
{
    let leaf_height = before.len() as int - 1;
    shape_edit(before, after, leaf_height, entry, true);
    assert(before.last().covered() && before.last().padded());
    group_edit(before.last(), after.last(), entry, true);
    if before.len() > 1 { path_valid(before, entry, leaf_height - 1); }
    assert forall|height: int, query: int|
        0 <= height < after.len() - 1 && 0 <= query < after[height].model_len()
            && (height > after.len() as int - 2 || query != path(entry, after.len() as int, height))
        implies #[trigger] bit(after[height].words(), query) == (after[height + 1].words()[query] == u64::MAX) by {
        assert(after[height] == before[height]);
        assert(bit(before[height].words(), query) == (before[height + 1].words()[query] == u64::MAX));
        if height + 1 == leaf_height {
            assert(query != entry / 64);
            assert(after.last().words()[query] == before.last().words()[query]);
        } else {
            assert(after[height + 1] == before[height + 1]);
        }
    }
}

pub proof fn set_finish(before: Seq<U64GroupedBitmap>, middle: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int)
    requires rep(before), 0 <= entry < before.last().model_len(),
        middle == before.update(before.len() as int - 1, middle.last()),
        group_delta(before.last(), middle.last(), entry, true),
        repair_frame(middle, after, entry, -1),
    ensures set_effect(before, after, entry), bit(after.last().words(), entry),
{
    assert(shape(before));
    assert(before[before.len() as int - 1].covered() && before[before.len() as int - 1].padded());
    assert(before.last().covered() && before.last().padded());
    group_edit(before.last(), middle.last(), entry, true);
    assert forall|height: int| 0 <= height < before.len()
        implies #[trigger] after[height].model_len() == before[height].model_len() by { }
    assert forall|height: int| 0 <= height < before.len() - 1
        implies #[trigger] after[height].words() == edited(before[height].words(), path(entry, before.len() as int, height),
            after[height + 1].words()[path(entry, before.len() as int, height)] == u64::MAX) by {
        assert(middle[height] == before[height]);
    }
}

impl BtreeBitmap {
    pub fn set(&mut self, i: u32)
        requires old(self).invariant(), i < old(self).levels().last().model_len(),
        ensures final(self).invariant(), set_effect(old(self).levels(), final(self).levels(), i as int),
            bit(final(self).levels().last().words(), i as int),
    {
        let ghost original = self.levels();
        let full = self.get_level_mut(self.get_height() - 1).set(i);
        let ghost middle = self.levels();
        proof {
            assert(middle == original.update(original.len() as int - 1, middle.last()));
            assert(group_delta(original.last(), middle.last(), i as int, true));
            leaf_begin(original, middle, i as int);
        }
        self.update_to_root(i, full);
        proof { set_finish(original, middle, self.levels(), i as int); }
    }

    pub fn alloc(&mut self) -> (result: Option<u32>)
        requires old(self).invariant(),
        ensures final(self).invariant(),
            result.is_none() <==> prefix_full(old(self).levels().last(), old(self).levels().last().model_len() as int),
            result.is_none() ==> *final(self) == *old(self),
            result.is_some() ==> result.unwrap() < old(self).levels().last().model_len(),
            result.is_some() ==> !bit(old(self).levels().last().words(), result.unwrap() as int),
            result.is_some() ==> prefix_full(old(self).levels().last(), result.unwrap() as int),
            result.is_some() ==> bit(final(self).levels().last().words(), result.unwrap() as int),
            result.is_some() ==> set_effect(old(self).levels(), final(self).levels(), result.unwrap() as int),
    {
        let entry = self.find_first_unset()?;
        self.set(entry);
        Some(entry)
    }
}

}
