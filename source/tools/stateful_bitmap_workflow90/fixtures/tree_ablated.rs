mod bitmap {
use vstd::prelude::*;

verus! {

pub(crate) struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}

impl U64GroupedBitmap {
    pub closed spec fn model_len(&self) -> u32 { self.len }
    pub closed spec fn words(&self) -> Seq<u64> { self.data@ }

    fn required_words(elements: u32) -> (result: usize)
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

pub(super) struct BtreeBitmap {
    heights: Vec<U64GroupedBitmap>,
}

pub closed spec fn reduced(value: int, rounds: nat) -> int
    decreases rounds,
{
    if rounds == 0 { value }
    else { (reduced(value, (rounds - 1) as nat) + 63) / 64 }
}

pub closed spec fn layer(heights: Seq<U64GroupedBitmap>, query: int) -> (int, Seq<u64>) {
    (heights[query].len as int, heights[query].data@)
}

pub closed spec fn expected_layer(pages: int, capacity: int, rounds: int) -> (int, Seq<u64>) {
    (reduced(pages, rounds as nat),
     Seq::new(((reduced(capacity, rounds as nat) + 63) / 64) as nat, |offset: int| u64::MAX))
}

impl BtreeBitmap {
    pub closed spec fn layers(&self) -> Seq<U64GroupedBitmap> { self.heights@ }

    pub(crate) fn new(mut num_pages: u32, mut capacity: u32) -> (result: Self)
        ensures
            0 < result.layers().len(),
            reduced(capacity as int, (result.layers().len() - 1) as nat) <= 64,
            forall|query: int| 0 <= query < result.layers().len() - 1 ==>
                #[trigger] reduced(capacity as int, query as nat) > 64,
            forall|query: int| 0 <= query < result.layers().len() ==>
                #[trigger] layer(result.layers(), query) ==
                    expected_layer(num_pages as int, capacity as int, result.layers().len() - 1 - query),
    {
        let ghost entry_pages = num_pages;
        let ghost entry_capacity = capacity;
        let mut heights = vec![];

        // Build from the leaf to root
        loop
            invariant_except_break
                capacity as int == reduced(entry_capacity as int, heights@.len()),
                num_pages as int == reduced(entry_pages as int, heights@.len()),
                forall|query: int| 0 <= query < heights@.len() ==>
                    #[trigger] reduced(entry_capacity as int, query as nat) > 64,
                forall|query: int| 0 <= query < heights@.len() ==>
                    #[trigger] layer(heights@, query) ==
                        expected_layer(entry_pages as int, entry_capacity as int, query),
            ensures
                0 < heights@.len(),
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

        Self { heights }
    }
}
}
}
