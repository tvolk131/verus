// Reusable bounded representation laws. No application state or summaries.
// This module is research proof assistance, not an engine inference rule.
use vstd::prelude::*;
verus! {
pub proof fn or_singleton_membership(word: u64, index: u64) by(bit_vector)
    requires index < 64,
    ensures
        forall|j: u64| j < 64 ==>
            (((word | (1u64 << index)) & (1u64 << j)) != 0)
                == (j == index || (word & (1u64 << j)) != 0),
{}
pub proof fn full_word_membership(word: u64) by(bit_vector)
    ensures
        (word == u64::MAX) ==
            (forall|j: u64| j < 64 ==> (word & (1u64 << j)) != 0),
{}
}
verus! {
pub open spec fn word_view(word: u64, size: nat) -> Seq<bool> {
    Seq::new(size, |j: int| (word & (1u64 << (j as u64))) != 0)
}
// Bridges finite bit representation to a sequence update and count. Its bound
// is the representation width, independent of any application type or name.
pub proof fn singleton_view_update(word: u64, index: u64, size: nat)
    requires index < size <= 64,
    ensures
        word_view(word | (1u64 << index), size) == word_view(word, size).update(index as int, true),
        word_view(word | (1u64 << index), size).count_prefix(|b: bool| b, size) ==
            word_view(word, size).count_prefix(|b: bool| b, size) +
                if word_view(word, size)[index as int] { 0nat } else { 1nat },
{
    or_singleton_membership(word, index);
    assert forall|j: int| 0 <= j < size implies
        word_view(word | (1u64 << index), size)[j] ==
            word_view(word, size).update(index as int, true)[j] by {
        assert((j as u64) < 64);
    }
    assert(word_view(word | (1u64 << index), size) =~= word_view(word, size).update(index as int, true));
    word_view(word, size).lemma_count_prefix_update(|b: bool| b, index as int, true, size);
}
pub proof fn full_view_with_padding(word: u64, size: nat)
    requires size <= 64,
        forall|j: int| size <= j < 64 ==> #[trigger] (word & (1u64 << (j as u64))) != 0,
    ensures
        (word == u64::MAX) == (forall|j: int| 0 <= j < size ==> word_view(word, size)[j]),
{
    full_word_membership(word);
    if word == u64::MAX {
        assert forall|j: int| 0 <= j < size implies word_view(word, size)[j] by {
            assert((j as u64) < 64);
        }
    } else if forall|j: int| 0 <= j < size ==> word_view(word, size)[j] {
        assert forall|j: u64| j < 64 implies (word & (1u64 << j)) != 0 by {
            if j < size { assert(word_view(word, size)[j as int]); }
            else { assert((word & (1u64 << ((j as int) as u64))) != 0); }
        }
    }
}
}
