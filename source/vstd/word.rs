//! Checked bounded u64 membership/sequence bridges. No application state model.
use super::prelude::*;
verus! {

broadcast use super::seq::group_seq_axioms;

pub open spec fn member(word: u64, index: int) -> bool {
    (word & (1u64 << (index as u64))) != 0
}

pub open spec fn view(word: u64, size: nat) -> Seq<bool> {
    Seq::new(size, |j: int| member(word, j))
}

proof fn or_bits(word: u64, index: u64)
    by (bit_vector)
    requires
        index < 64,
    ensures
        forall|j: u64|
            j < 64 ==> (((word | (1u64 << index)) & (1u64 << j)) != 0) == (j == index || (word & (
            1u64 << j)) != 0),
{
}

proof fn full_bits(word: u64)
    by (bit_vector)
    ensures
        (word == u64::MAX) == (forall|j: u64| j < 64 ==> (word & (1u64 << j)) != 0),
{
}

pub broadcast proof fn lemma_or_member(word: u64, index: u64, j: int)
    requires
        index < 64,
        0 <= j < 64,
    ensures
        #[trigger] member(word | (1u64 << index), j) == (j == index || member(word, j)),
{
    or_bits(word, index);
    assert((j as u64) < 64);
}

pub broadcast proof fn lemma_or_view(word: u64, index: u64, size: nat)
    requires
        index < size <= 64,
    ensures
        #[trigger] view(word | (1u64 << index), size) == view(word, size).update(
            index as int,
            true,
        ),
        view(word | (1u64 << index), size).count_prefix(|b: bool| b, size) == view(
            word,
            size,
        ).count_prefix(|b: bool| b, size) + if view(word, size)[index as int] {
            0nat
        } else {
            1nat
        },
{
    or_bits(word, index);
    assert forall|j: int| 0 <= j < size implies view(word | (1u64 << index), size)[j] == view(
        word,
        size,
    ).update(index as int, true)[j] by {
        assert((j as u64) < 64);
    }
    assert(view(word | (1u64 << index), size) =~= view(word, size).update(index as int, true));
    view(word, size).lemma_count_prefix_update(|b: bool| b, index as int, true, size);
}

pub broadcast proof fn lemma_full_view(word: u64, size: nat)
    requires
        size <= 64,
        forall|j: int| size <= j < 64 ==> #[trigger] member(word, j),
    ensures
        #![trigger view(word, size)]
        (word == u64::MAX) == (forall|j: int| 0 <= j < size ==> #[trigger] view(word, size)[j]),
{
    full_bits(word);
    if word == u64::MAX {
        assert forall|j: int| 0 <= j < size implies view(word, size)[j] by {
            assert((j as u64) < 64);
        }
    } else if forall|j: int| 0 <= j < size ==> view(word, size)[j] {
        assert forall|j: u64| j < 64 implies (word & (1u64 << j)) != 0 by {
            if j < size {
                assert(view(word, size)[j as int]);
            } else {
                assert(member(word, j as int));
            }
        }
    }
}

pub open spec fn seq_word_bit(s: Seq<u64>, j: int) -> bool {
    member(s[j / 64], j % 64)
}

pub open spec fn seq_word_edit(s: Seq<u64>, i: int, value: bool) -> Seq<u64> {
    s.update(
        i / 64,
        if value {
            s[i / 64] | (1u64 << ((i % 64) as u64))
        } else {
            s[i / 64] & !(1u64 << ((i % 64) as u64))
        },
    )
}

pub proof fn checked_word_bits(x: u64, p: u64, j: u64)
    requires
        p < 64,
        j < 64,
    ensures
        member(x | (1u64 << p), j as int) == (j == p || member(x, j as int)),
        member(x & !(1u64 << p), j as int) == (j != p && member(x, j as int)),
        member(u64::MAX, j as int),
{
    assert((p < 64 && j < 64) ==> (((x | (1u64 << p)) & (1u64 << j) != 0) == (j == p || (x & (1u64
        << j) != 0)))) by (bit_vector);
    assert(((x & !(1u64 << p)) & (1u64 << j) != 0) == (j != p && (x & (1u64 << j) != 0)))
        by (bit_vector);
    assert(j < 64 ==> (u64::MAX & (1u64 << j)) != 0) by (bit_vector);
}

pub proof fn checked_seq_bit_update(s: Seq<u64>, i: int, value: bool)
    requires
        0 <= i < s.len() * 64,
    ensures
        forall|j: int|
            #![trigger seq_word_edit(s,i,value)[j / 64]]
            0 <= j < s.len() * 64 ==> seq_word_bit(seq_word_edit(s, i, value), j) == (if j == i {
                value
            } else {
                seq_word_bit(s, j)
            }),
{
    let z = seq_word_edit(s, i, value);
    assert(0 <= i / 64 < s.len());
    assert forall|j: int| 0 <= j < s.len() * 64 implies seq_word_bit(z, j) == (if j == i {
        value
    } else {
        seq_word_bit(s, j)
    }) by {
        if j / 64 == i / 64 {
            checked_word_bits(s[j / 64], (i % 64) as u64, (j % 64) as u64);
            assert((j % 64 == i % 64) == (j == i));
        }
    }
}

pub proof fn checked_seq_bit_update_at(s: Seq<u64>, i: int, value: bool, j: int)
    requires
        0 <= i < s.len() * 64,
        0 <= j < s.len() * 64,
    ensures
        seq_word_bit(seq_word_edit(s, i, value), j) == (if j == i {
            value
        } else {
            seq_word_bit(s, j)
        }),
{
    checked_seq_bit_update(s, i, value);
    assert(seq_word_bit(seq_word_edit(s, i, value), j) == (if j == i {
        value
    } else {
        seq_word_bit(s, j)
    }));
}

} // verus!
