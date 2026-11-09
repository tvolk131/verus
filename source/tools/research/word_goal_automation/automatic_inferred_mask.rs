use vstd::prelude::*;
verus! {
pub(crate) struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}

pub use vstd::word::member;
pub open spec fn allocated(word: u64, len: u32) -> Seq<bool> {
    vstd::word::view(word, len as nat)
}
pub open spec fn count(view: Seq<bool>) -> nat {
    view.count_prefix(|b: bool| b, view.len())
}
impl U64GroupedBitmap {
    spec fn wf(&self) -> bool {
        self.data@.len() == 1 && 1 <= self.len <= 64 &&
        forall|j: int| self.len <= j < 64 ==> member(self.data@[0], j)
    }
    spec fn view(&self) -> Seq<bool> {
        allocated(self.data@[0], self.len)
    }
    fn data_index_of(bit: u32) -> (r: (usize, usize))
        ensures r.0 == (bit as usize) / 64, r.1 == (bit as usize) % 64,
    {
        ((bit as usize) / 64, (bit as usize) % 64)
    }
    #[verifier::infer]
    fn select_mask(bit: usize) -> (r: u64)
    {
        1u64 << (bit as u64)
    }
    fn get(&self, bit: u32) -> (r: bool)
        requires self.wf(), bit < self.len,
        ensures r == self.view()[bit as int],
    {
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let group = self.data[index];
        group & U64GroupedBitmap::select_mask(bit_index) != 0
    }
    fn set(&mut self, bit: u32) -> (r: bool)
        requires old(self).wf(), bit < old(self).len,
        ensures
            final(self).wf(), final(self).len == old(self).len,
            final(self).data@.len() == old(self).data@.len(),
            final(self).view() == old(self).view().update(bit as int, true),
            forall|j: int| 0 <= j < final(self).len && j != bit ==>
                final(self).view()[j] == old(self).view()[j],
            count(final(self).view()) == count(old(self).view()) +
                if old(self).view()[bit as int] { 0nat } else { 1nat },
            r == (forall|j: int| 0 <= j < final(self).len ==> final(self).view()[j]),
    {
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let mut group = self.data[index];
        group |= Self::select_mask(bit_index);
        self.data[index] = group;

        group == u64::MAX
    }
}
}
fn main() {}
