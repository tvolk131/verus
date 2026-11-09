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


pub closed spec fn reduced(value: int, rounds: nat) -> int
    decreases rounds,
{
    if rounds == 0 { value }
    else { (reduced(value, (rounds - 1) as nat) + 63) / 64 }
}

pub closed spec fn layer(heights: Seq<U64GroupedBitmap>, query: int) -> (int, Seq<u64>) {
    (heights[query].model_len() as int, heights[query].words())
}

pub closed spec fn expected_layer(pages: int, capacity: int, rounds: int) -> (int, Seq<u64>) {
    (reduced(pages, rounds as nat),
     Seq::new(((reduced(capacity, rounds as nat) + 63) / 64) as nat, |offset: int| u64::MAX))
}

impl U64GroupedBitmap {

    pub fn required_words(elements: u32) -> (result: usize)
        ensures result as int == (elements as int + 63) / 64,
    {
        let words = elements.div_ceil(64);
        words as usize
    }

    pub fn new_full(len: u32, capacity: u32) -> (result: Self)
        ensures
            result.model_len() == len,
            result.words() == Seq::new(((capacity as int + 63) / 64) as nat, |offset: int| u64::MAX),
    {
        let data = vec![u64::MAX; Self::required_words(capacity)];
        Self { len, data }
    }
}

impl BtreeBitmap {
    pub fn new(mut num_pages: u32, mut capacity: u32) -> (result: Self)
        ensures
            result.invariant() <==> constructor_domain(num_pages, capacity),
            result.levels().last().model_len() == num_pages,
            constructor_domain(num_pages, capacity) ==> prefix_full(result.levels().last(), num_pages as int),
            result.levels().last().words() == Seq::new(((capacity as int + 63) / 64) as nat, |offset: int| u64::MAX),
            result.levels().len() <= u32::MAX,
            0 < result.levels().len(),
            reduced(capacity as int, (result.levels().len() - 1) as nat) <= 64,
            forall|query: int| 0 <= query < result.levels().len() - 1 ==>
                #[trigger] reduced(capacity as int, query as nat) > 64,
            forall|query: int| 0 <= query < result.levels().len() ==>
                #[trigger] layer(result.levels(), query) ==
                    expected_layer(num_pages as int, capacity as int, result.levels().len() - 1 - query),
    {
        let ghost entry_pages = num_pages;
        let ghost entry_capacity = capacity;
        let mut heights = vec![];

        // Build from the leaf to root
        loop
            invariant_except_break
                capacity as int + heights@.len() <= entry_capacity as int,
                (capacity == 0) == (entry_capacity == 0),
                capacity as int == reduced(entry_capacity as int, heights@.len()),
                num_pages as int == reduced(entry_pages as int, heights@.len()),
                forall|query: int| 0 <= query < heights@.len() ==>
                    #[trigger] reduced(entry_capacity as int, query as nat) > 64,
                forall|query: int| 0 <= query < heights@.len() ==>
                    #[trigger] layer(heights@, query) ==
                        expected_layer(entry_pages as int, entry_capacity as int, query),
            ensures
                0 < heights@.len() <= u32::MAX,
                reduced(entry_capacity as int, (heights@.len() - 1) as nat) <= 64,
                forall|query: int| 0 <= query < heights@.len() - 1 ==>
                    #[trigger] reduced(entry_capacity as int, query as nat) > 64,
                forall|query: int| 0 <= query < heights@.len() ==>
                    #[trigger] layer(heights@, query) ==
                        expected_layer(entry_pages as int, entry_capacity as int, query),
            decreases capacity,
        {
            let ghost before = heights@;
            heights.push(U64GroupedBitmap::new_full(num_pages, capacity));
            proof {
                assert forall|query: int| 0 <= query < heights@.len() implies
                    #[trigger] layer(heights@, query) ==
                        expected_layer(entry_pages as int, entry_capacity as int, query)
                by {
                    if query < before.len() {
                        assert(layer(before, query) ==
                            expected_layer(entry_pages as int, entry_capacity as int, query));
                    }
                }
            }
            if capacity <= 64 {
                break;
            }
            capacity = capacity.div_ceil(64);
            num_pages = num_pages.div_ceil(64);
        }

        // Reverse so that the root is at index 0
        let ghost before_reverse = heights@;
        heights.reverse();
        proof {
            assert forall|query: int| 0 <= query < heights@.len() implies
                #[trigger] layer(heights@, query) ==
                    expected_layer(entry_pages as int, entry_capacity as int, heights@.len() - 1 - query)
            by {
                assert(layer(before_reverse, heights@.len() - 1 - query) ==
                    expected_layer(entry_pages as int, entry_capacity as int, heights@.len() - 1 - query));
            }
        }

        proof {
            assert(layer(heights@, heights@.len() - 1) == expected_layer(entry_pages as int, entry_capacity as int, 0));
            assert(reduced(entry_capacity as int, 0) == entry_capacity as int);
            assert(reduced(entry_pages as int, 0) == entry_pages as int);
            if constructor_domain(entry_pages, entry_capacity) {
                construction_rep(heights@, entry_pages, entry_capacity);
            }
            constructor_domain_necessary(heights@, entry_pages, entry_capacity);
        }
        Self { heights }
    }
}

pub open spec fn constructor_domain(pages: u32, capacity: u32) -> bool {
    pages <= MAX_PAGE_INDEX + 1 && pages as int <= 64 * ((capacity as int + 63) / 64)
}

pub proof fn reduced_nonnegative(value: int, rounds: nat)
    requires value >= 0,
    ensures 0 <= reduced(value, rounds) <= value,
    decreases rounds,
{
    if rounds > 0 {
        reduced_nonnegative(value, (rounds - 1) as nat);
    }
}

pub proof fn reduced_coverage(pages: int, capacity: int, rounds: nat)
    requires pages >= 0, capacity >= 0,
        pages <= 64 * ((capacity + 63) / 64),
    ensures
        0 <= reduced(pages, rounds) <= 64 * ((reduced(capacity, rounds) + 63) / 64),
        rounds > 0 ==> reduced(pages, rounds) <= reduced(capacity, rounds),
    decreases rounds,
{
    reduced_nonnegative(pages, rounds);
    reduced_nonnegative(capacity, rounds);
    if rounds == 1 {
        assert(reduced(pages, 0) == pages);
        assert(reduced(capacity, 0) == capacity);
        assert(reduced(pages, rounds) == (pages + 63) / 64);
        assert(reduced(capacity, rounds) == (capacity + 63) / 64);
        assert((pages + 63) / 64 <= (capacity + 63) / 64);
    } else if rounds > 1 {
        reduced_coverage(pages, capacity, (rounds - 1) as nat);
        assert((reduced(pages, (rounds - 1) as nat) + 63) / 64 <=
            (reduced(capacity, (rounds - 1) as nat) + 63) / 64);
    }
    assert(reduced(capacity, rounds) <= 64 * ((reduced(capacity, rounds) + 63) / 64));
}

pub proof fn construction_rep(levels: Seq<U64GroupedBitmap>, pages: u32, capacity: u32)
    requires constructor_domain(pages, capacity),
        1 <= levels.len() <= u32::MAX,
        reduced(capacity as int, (levels.len() - 1) as nat) <= 64,
        forall|height: int| 0 <= height < levels.len() ==>
            #[trigger] layer(levels, height) ==
                expected_layer(pages as int, capacity as int, levels.len() - 1 - height),
    ensures rep(levels), levels.last().model_len() == pages,
        prefix_full(levels.last(), pages as int),
{
    let depth = (levels.len() - 1) as nat;
    assert(layer(levels, 0) == expected_layer(pages as int, capacity as int, depth as int));
    assert(layer(levels, levels.len() - 1) == expected_layer(pages as int, capacity as int, 0));
    reduced_coverage(pages as int, capacity as int, depth);
    if depth == 0 {
        assert(capacity <= 64);
        assert((capacity as int + 63) / 64 <= 1);
    }
    assert(levels[0].model_len() <= 64);
    assert(levels.last().model_len() == pages);

    assert forall|height: int| #![trigger levels[height]] 0 <= height < levels.len()
        implies levels[height].covered() && levels[height].padded() by {
        let rounds = (levels.len() - 1 - height) as nat;
        assert(layer(levels, height) == expected_layer(pages as int, capacity as int, rounds as int));
        reduced_coverage(pages as int, capacity as int, rounds);
        assert(levels[height].covered());
        assert forall|query: int| levels[height].model_len() <= query <
            ((levels[height].model_len() as int + 63) / 64) * 64 implies
            #[trigger] bit(levels[height].words(), query) by {
            assert(0 <= query < 64 * levels[height].words().len());
            assert(levels[height].words()[query / 64] == u64::MAX);
            full_word(levels[height].words(), query);
        }
    }

    assert forall|height: int| 0 <= height < levels.len() - 1 implies
        #[trigger] levels[height].model_len() as int ==
            (levels[height + 1].model_len() as int + 63) / 64 by {
        let child_rounds = (levels.len() - 2 - height) as nat;
        assert(layer(levels, height) == expected_layer(pages as int, capacity as int, child_rounds as int + 1));
        assert(layer(levels, height + 1) == expected_layer(pages as int, capacity as int, child_rounds as int));
        assert(reduced(pages as int, child_rounds + 1) ==
            (reduced(pages as int, child_rounds) + 63) / 64);
    }
    assert(shape(levels));
    assert forall|height: int, entry: int| 0 <= height < levels.len() - 1 &&
        0 <= entry < levels[height].model_len() implies
        #[trigger] bit(levels[height].words(), entry) ==
            (levels[height + 1].words()[entry] == u64::MAX) by {
        assert(layer(levels, height) == expected_layer(pages as int, capacity as int,
            levels.len() - 1 - height));
        assert(layer(levels, height + 1) == expected_layer(pages as int, capacity as int,
            levels.len() - 2 - height));
        assert(entry < levels[height + 1].words().len());
        assert(levels[height + 1].words()[entry] == u64::MAX);
        assert(levels[height].words()[entry / 64] == u64::MAX);
        full_word(levels[height].words(), entry);
    }
    assert forall|query: int| 0 <= query < pages implies
        #[trigger] bit(levels.last().words(), query) by {
        assert(levels.last().words()[query / 64] == u64::MAX);
        full_word(levels.last().words(), query);
    }
}

pub proof fn constructor_domain_necessary(levels: Seq<U64GroupedBitmap>, pages: u32, capacity: u32)
    requires levels.len() > 0, levels.last().model_len() == pages,
        levels.last().words().len() == (capacity as int + 63) / 64,
    ensures rep(levels) ==> constructor_domain(pages, capacity),
{
    if rep(levels) {
        assert(levels.last().covered());
    }
}

pub fn constructed_full_lifecycle(pages: u32, capacity: u32) -> (result: BtreeBitmap)
    requires constructor_domain(pages, capacity),
    ensures result.invariant(), result.levels().last().model_len() == pages,
        prefix_full(result.levels().last(), pages as int),
{
    let mut tree = BtreeBitmap::new(pages, capacity);
    let ghost initial = tree;
    let first = tree.alloc();
    assert(first.is_none());
    let second = tree.alloc();
    assert(second.is_none());
    assert(tree == initial);
    tree
}

pub open spec fn edit_effect(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int, value: bool) -> bool {
    &&& after.len() == before.len()
    &&& forall|height: int| 0 <= height < before.len() ==>
        #[trigger] after[height].model_len() == before[height].model_len()
    &&& after.last().words() == edited(before.last().words(), entry, value)
    &&& forall|height: int| 0 <= height < before.len() - 1 ==>
        #[trigger] after[height].words() == edited(before[height].words(), path(entry, before.len() as int, height),
            after[height + 1].words()[path(entry, before.len() as int, height)] == u64::MAX)
}

pub proof fn leaf_begin_value(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int, value: bool)
    requires rep(before), 0 <= entry < before.last().model_len(),
        after == before.update(before.len() as int - 1, after.last()),
        group_delta(before.last(), after.last(), entry, value),
    ensures shape(after), hierarchy_except(after, entry, after.len() as int - 2),
        bit(after.last().words(), entry) == value,
        !value ==> after.last().words()[entry / 64] != u64::MAX,
{
    let leaf_height = before.len() as int - 1;
    shape_edit(before, after, leaf_height, entry, value);
    assert(before.last().covered() && before.last().padded());
    group_edit(before.last(), after.last(), entry, value);
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

pub proof fn edit_finish(before: Seq<U64GroupedBitmap>, middle: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int, value: bool)
    requires rep(before), 0 <= entry < before.last().model_len(),
        middle == before.update(before.len() as int - 1, middle.last()),
        group_delta(before.last(), middle.last(), entry, value),
        repair_frame(middle, after, entry, -1),
    ensures edit_effect(before, after, entry, value), bit(after.last().words(), entry) == value,
{
    assert(shape(before));
    assert(before[before.len() as int - 1].covered() && before[before.len() as int - 1].padded());
    assert(before.last().covered() && before.last().padded());
    group_edit(before.last(), middle.last(), entry, value);
    assert forall|height: int| 0 <= height < before.len()
        implies #[trigger] after[height].model_len() == before[height].model_len() by { }
    assert forall|height: int| 0 <= height < before.len() - 1
        implies #[trigger] after[height].words() == edited(before[height].words(), path(entry, before.len() as int, height),
            after[height + 1].words()[path(entry, before.len() as int, height)] == u64::MAX) by {
        assert(middle[height] == before[height]);
    }
}

impl BtreeBitmap {
    pub fn clear(&mut self, i: u32)
        requires old(self).invariant(), i < old(self).levels().last().model_len(),
        ensures final(self).invariant(),
            edit_effect(old(self).levels(), final(self).levels(), i as int, false),
            !bit(final(self).levels().last().words(), i as int),
    {
        let ghost original = self.levels();
        self.get_level_mut(self.get_height() - 1).clear(i);
        let ghost middle = self.levels();
        proof {
            assert(middle == original.update(original.len() as int - 1, middle.last()));
            assert(group_delta(original.last(), middle.last(), i as int, false));
            leaf_begin_value(original, middle, i as int, false);
        }
        self.update_to_root(i, false);
        proof { edit_finish(original, middle, self.levels(), i as int, false); }
    }
}

pub open spec fn physical_frame(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>) -> bool {
    &&& after.len() == before.len()
    &&& forall|height: int| 0 <= height < before.len() ==>
        #[trigger] after[height].model_len() == before[height].model_len()
        && after[height].words().len() == before[height].words().len()
    &&& forall|height: int, word: int| 0 <= height < before.len() &&
        (before[height].model_len() as int + 63) / 64 <= word < before[height].words().len() ==>
        #[trigger] after[height].words()[word] == before[height].words()[word]
    &&& forall|height: int, query: int| 0 <= height < before.len() &&
        before[height].model_len() <= query < 64 * before[height].words().len() ==>
        #[trigger] bit(after[height].words(), query) == bit(before[height].words(), query)
}

pub proof fn effect_frame(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, entry: int, value: bool)
    requires rep(before), 0 <= entry < before.last().model_len(), edit_effect(before, after, entry, value),
    ensures physical_frame(before, after),
        forall|height: int, word: int| 0 <= height < before.len() &&
            0 <= word < before[height].words().len() && word != path(entry, before.len() as int, height) / 64 ==>
            #[trigger] after[height].words()[word] == before[height].words()[word],
        forall|height: int, query: int| 0 <= height < before.len() &&
            0 <= query < 64 * before[height].words().len() && query != path(entry, before.len() as int, height) ==>
            #[trigger] bit(after[height].words(), query) == bit(before[height].words(), query),
{
    assert forall|height: int| 0 <= height < before.len() implies
        #[trigger] after[height].model_len() == before[height].model_len()
        && after[height].words().len() == before[height].words().len() by {
        path_valid(before, entry, height);
        let selected = path(entry, before.len() as int, height);
        let written = if height == before.len() - 1 { value }
            else { after[height + 1].words()[selected] == u64::MAX };
        assert(group_delta(before[height], after[height], selected, written));
        group_edit(before[height], after[height], selected, written);
    }
    assert forall|height: int, word: int| 0 <= height < before.len() &&
        0 <= word < before[height].words().len() && word != path(entry, before.len() as int, height) / 64 implies
        #[trigger] after[height].words()[word] == before[height].words()[word] by {
        path_valid(before, entry, height);
        let selected = path(entry, before.len() as int, height);
        if height == before.len() - 1 {
            assert(after[height].words() == edited(before[height].words(), entry, value));
        } else {
            assert(after[height].words() == edited(before[height].words(), selected,
                after[height + 1].words()[selected] == u64::MAX));
        }
    }
    assert forall|height: int, query: int| 0 <= height < before.len() &&
        0 <= query < 64 * before[height].words().len() && query != path(entry, before.len() as int, height) implies
        #[trigger] bit(after[height].words(), query) == bit(before[height].words(), query) by {
        path_valid(before, entry, height);
        let selected = path(entry, before.len() as int, height);
        let written = if height == before.len() - 1 { value }
            else { after[height + 1].words()[selected] == u64::MAX };
        assert(group_delta(before[height], after[height], selected, written));
        group_edit(before[height], after[height], selected, written);
    }
    assert forall|height: int, word: int| 0 <= height < before.len() &&
        (before[height].model_len() as int + 63) / 64 <= word < before[height].words().len() implies
        #[trigger] after[height].words()[word] == before[height].words()[word] by {
        path_valid(before, entry, height);
        assert(word != path(entry, before.len() as int, height) / 64);
    }
    assert forall|height: int, query: int| 0 <= height < before.len() &&
        before[height].model_len() <= query < 64 * before[height].words().len() implies
        #[trigger] bit(after[height].words(), query) == bit(before[height].words(), query) by {
        path_valid(before, entry, height);
        assert(query != path(entry, before.len() as int, height));
    }
}

pub proof fn frame_transitive(before: Seq<U64GroupedBitmap>, middle: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>)
    requires physical_frame(before, middle), physical_frame(middle, after),
    ensures physical_frame(before, after),
{
    assert(after.len() == before.len());
    assert forall|height: int| 0 <= height < before.len() implies
        #[trigger] after[height].model_len() == before[height].model_len()
        && after[height].words().len() == before[height].words().len() by {
        assert(middle[height].model_len() == before[height].model_len());
        assert(after[height].model_len() == middle[height].model_len());
        assert(middle[height].words().len() == before[height].words().len());
        assert(after[height].words().len() == middle[height].words().len());
    }
    assert forall|height: int, word: int| 0 <= height < before.len() &&
        (before[height].model_len() as int + 63) / 64 <= word < before[height].words().len() implies
        #[trigger] after[height].words()[word] == before[height].words()[word] by {
        assert(middle[height].model_len() == before[height].model_len());
        assert(middle[height].words().len() == before[height].words().len());
        assert(middle[height].words()[word] == before[height].words()[word]);
        assert(after[height].words()[word] == middle[height].words()[word]);
    }
    assert forall|height: int, query: int| 0 <= height < before.len() &&
        before[height].model_len() <= query < 64 * before[height].words().len() implies
        #[trigger] bit(after[height].words(), query) == bit(before[height].words(), query) by {
        assert(middle[height].model_len() == before[height].model_len());
        assert(middle[height].words().len() == before[height].words().len());
        assert(bit(middle[height].words(), query) == bit(before[height].words(), query));
        assert(bit(after[height].words(), query) == bit(middle[height].words(), query));
    }
}

pub open spec fn leaf_prefix_effect(before: U64GroupedBitmap, after: U64GroupedBitmap, count: int) -> bool {
    after.model_len() == before.model_len() &&
    forall|query: int| 0 <= query < before.model_len() ==>
        #[trigger] bit(after.words(), query) == (query >= count && bit(before.words(), query))
}

pub proof fn prefix_clear_step(original: U64GroupedBitmap, before: U64GroupedBitmap, after: U64GroupedBitmap, index: int)
    requires before.covered(), before.padded(), 0 <= index < before.model_len(),
        leaf_prefix_effect(original, before, index), group_delta(before, after, index, false),
    ensures leaf_prefix_effect(original, after, index + 1),
{
    group_edit(before, after, index, false);
    assert forall|query: int| 0 <= query < original.model_len() implies
        #[trigger] bit(after.words(), query) == (query >= index + 1 && bit(original.words(), query)) by {
        assert(bit(before.words(), query) == (query >= index && bit(original.words(), query)));
    }
}

pub open spec fn clear_trace_prefix(history: Seq<Seq<U64GroupedBitmap>>, count: int) -> bool {
    &&& count >= 0 && history.len() == count + 1
    &&& forall|step: int| 0 <= step < history.len() ==> #[trigger] rep(history[step])
    &&& forall|step: int| 0 <= step < count ==>
        #[trigger] edit_effect(history[step], history[step + 1], step, false)
}

pub fn clear_all(tree: &mut BtreeBitmap, pages: u32) -> (result: Ghost<Seq<Seq<U64GroupedBitmap>>>)
    requires old(tree).invariant(), pages == old(tree).levels().last().model_len(),
    ensures final(tree).invariant(), clear_trace_prefix(result@, pages as int),
        result@[0] == old(tree).levels(), result@.last() == final(tree).levels(),
        physical_frame(old(tree).levels(), final(tree).levels()),
        leaf_prefix_effect(old(tree).levels().last(), final(tree).levels().last(), pages as int),
{
    let ghost original = tree.levels();
    let ghost history = seq![original];
    for index in 0..pages
        invariant
            index <= pages, tree.invariant(), tree.levels().last().model_len() == pages,
            clear_trace_prefix(history, index as int), history[0] == original,
            history.last() == tree.levels(),
            physical_frame(original, tree.levels()),
            leaf_prefix_effect(original.last(), tree.levels().last(), index as int),
        decreases pages - index,
    {
        let ghost before = tree.levels();
        tree.clear(index);
        proof {
            effect_frame(before, tree.levels(), index as int, false);
            frame_transitive(original, before, tree.levels());
            assert(group_delta(before.last(), tree.levels().last(), index as int, false));
            prefix_clear_step(original.last(), before.last(), tree.levels().last(), index as int);
            let previous = history;
            history = history.push(tree.levels());
            assert forall|step: int| 0 <= step < history.len() implies #[trigger] rep(history[step]) by {
                if step < previous.len() { assert(history[step] == previous[step]); }
            }
            assert forall|step: int| 0 <= step < index as int + 1 implies
                #[trigger] edit_effect(history[step], history[step + 1], step, false) by {
                if step < index { assert(history[step] == previous[step]); assert(history[step + 1] == previous[step + 1]); }
                else { assert(history[step] == before); }
            }
        }
    }
    Ghost(history)
}

pub open spec fn allocation_step(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, result: Option<u32>) -> bool {
    &&& rep(after)
    &&& (result.is_none() <==> prefix_full(before.last(), before.last().model_len() as int))
    &&& (result.is_none() ==> after == before)
    &&& (result.is_some() ==> result.unwrap() < before.last().model_len()
        && !bit(before.last().words(), result.unwrap() as int)
        && prefix_full(before.last(), result.unwrap() as int)
        && bit(after.last().words(), result.unwrap() as int)
        && set_effect(before, after, result.unwrap() as int))
}

pub proof fn initialized_first(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, result: Option<u32>)
    requires rep(before), allocation_step(before, after, result),
        forall|query: int| 0 <= query < before.last().model_len() ==> !#[trigger] bit(before.last().words(), query),
    ensures result == (if before.last().model_len() == 0 { None } else { Some(0u32) }),
        after.last().model_len() == before.last().model_len(), physical_frame(before, after),
        forall|query: int| 0 <= query < after.last().model_len() ==>
            #[trigger] bit(after.last().words(), query) == (query == 0),
{
    if before.last().model_len() == 0 {
        assert(prefix_full(before.last(), 0));
    } else {
        assert(!bit(before.last().words(), 0));
        assert(!prefix_full(before.last(), before.last().model_len() as int));
        assert(result.is_some());
        if result.unwrap() > 0 { assert(bit(before.last().words(), 0)); }
        assert(result == Some(0u32));
        assert(edit_effect(before, after, 0, true));
        effect_frame(before, after, 0, true);
        assert(group_delta(before.last(), after.last(), 0, true));
        group_edit(before.last(), after.last(), 0, true);
    }
}

pub proof fn initialized_second(before: Seq<U64GroupedBitmap>, after: Seq<U64GroupedBitmap>, result: Option<u32>)
    requires rep(before), allocation_step(before, after, result),
        forall|query: int| 0 <= query < before.last().model_len() ==>
            #[trigger] bit(before.last().words(), query) == (query == 0),
    ensures result == (if before.last().model_len() < 2 { None } else { Some(1u32) }),
        after.last().model_len() == before.last().model_len(), physical_frame(before, after),
        forall|query: int| 0 <= query < after.last().model_len() ==>
            #[trigger] bit(after.last().words(), query) == (query < 2),
{
    if before.last().model_len() < 2 {
        assert(prefix_full(before.last(), before.last().model_len() as int));
    } else {
        assert(!bit(before.last().words(), 1));
        assert(!prefix_full(before.last(), before.last().model_len() as int));
        assert(result.is_some());
        if result.unwrap() == 0 { assert(bit(before.last().words(), 0)); }
        if result.unwrap() > 1 { assert(bit(before.last().words(), 1)); }
        assert(result == Some(1u32));
        assert(edit_effect(before, after, 1, true));
        effect_frame(before, after, 1, true);
        assert(group_delta(before.last(), after.last(), 1, true));
        group_edit(before.last(), after.last(), 1, true);
    }
}

pub struct InitializedRun {
    pub tree: BtreeBitmap,
    pub first: Option<u32>,
    pub second: Option<u32>,
    pub clear_trace: Ghost<Seq<Seq<U64GroupedBitmap>>>,
    pub initialized: Ghost<Seq<U64GroupedBitmap>>,
    pub after_first: Ghost<Seq<U64GroupedBitmap>>,
}

pub fn new_initialized_allocations(pages: u32, capacity: u32) -> (result: InitializedRun)
    requires constructor_domain(pages, capacity),
    ensures result.tree.invariant(),
        result.first == (if pages == 0 { None } else { Some(0u32) }),
        result.second == (if pages < 2 { None } else { Some(1u32) }),
        result.tree.levels().last().model_len() == pages,
        clear_trace_prefix(result.clear_trace@, pages as int),
        result.clear_trace@[0].last().model_len() == pages,
        result.clear_trace@[0].last().words() == Seq::new(((capacity as int + 63) / 64) as nat, |offset: int| u64::MAX),
        forall|height: int| 0 <= height < result.clear_trace@[0].len() ==>
            #[trigger] layer(result.clear_trace@[0], height) == expected_layer(pages as int, capacity as int,
                result.clear_trace@[0].len() - 1 - height),
        result.initialized@ == result.clear_trace@.last(),
        leaf_prefix_effect(result.clear_trace@[0].last(), result.initialized@.last(), pages as int),
        allocation_step(result.initialized@, result.after_first@, result.first),
        allocation_step(result.after_first@, result.tree.levels(), result.second),
        physical_frame(result.clear_trace@[0], result.initialized@),
        physical_frame(result.initialized@, result.after_first@),
        physical_frame(result.after_first@, result.tree.levels()),
        physical_frame(result.clear_trace@[0], result.tree.levels()),
        forall|query: int| 0 <= query < pages ==>
            #[trigger] bit(result.tree.levels().last().words(), query) == (query < 2),
{
    let mut tree = BtreeBitmap::new(pages, capacity);
    let ghost constructed = tree.levels();
    let history = clear_all(&mut tree, pages);
    let ghost initialized = tree.levels();
    let first = tree.alloc();
    let ghost after_first = tree.levels();
    proof {
        assert(allocation_step(initialized, after_first, first));
        assert forall|query: int| 0 <= query < initialized.last().model_len() implies
            !#[trigger] bit(initialized.last().words(), query) by { }
        initialized_first(initialized, after_first, first);
    }
    let second = tree.alloc();
    proof {
        assert(allocation_step(after_first, tree.levels(), second));
        initialized_second(after_first, tree.levels(), second);
        frame_transitive(constructed, initialized, after_first);
        frame_transitive(constructed, after_first, tree.levels());
    }
    InitializedRun { tree, first, second, clear_trace: history,
        initialized: Ghost(initialized), after_first: Ghost(after_first) }
}

}
