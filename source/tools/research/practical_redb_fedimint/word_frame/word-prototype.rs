use vstd::prelude::*;
verus! {
pub open spec fn seq_word_bit(s: Seq<u64>, j: int) -> bool {
    vstd::word::member(s[j / 64], j % 64)
}
pub open spec fn seq_word_edit(s: Seq<u64>, i: int, value: bool) -> Seq<u64> {
    s.update(i / 64, if value {
        s[i / 64] | (1u64 << ((i % 64) as u64))
    } else { s[i / 64] & !(1u64 << ((i % 64) as u64)) })
}
pub proof fn checked_word_bits(x: u64, p: u64, j: u64)
    requires p < 64, j < 64,
    ensures
        vstd::word::member(x | (1u64 << p), j as int) == (j == p || vstd::word::member(x, j as int)),
        vstd::word::member(x & !(1u64 << p), j as int) == (j != p && vstd::word::member(x, j as int)),
        vstd::word::member(u64::MAX, j as int),
{
    assert((p < 64 && j < 64) ==> (((x | (1u64 << p)) & (1u64 << j) != 0) ==
        (j == p || (x & (1u64 << j) != 0)))) by(bit_vector);
    assert(((x & !(1u64 << p)) & (1u64 << j) != 0) ==
        (j != p && (x & (1u64 << j) != 0))) by(bit_vector);
    assert(j < 64 ==> (u64::MAX & (1u64 << j)) != 0) by(bit_vector);
}
pub proof fn checked_seq_bit_update(s: Seq<u64>, i: int, value: bool)
    requires 0 <= i < s.len() * 64,
    ensures
        forall|j: int| #![trigger seq_word_edit(s,i,value)[j / 64]]
            0 <= j < s.len() * 64 ==> seq_word_bit(seq_word_edit(s,i,value),j)
            == (if j == i { value } else { seq_word_bit(s,j) }),
{
    let z = seq_word_edit(s,i,value);
    assert(0 <= i / 64 < s.len());
    assert forall|j: int| 0 <= j < s.len() * 64 implies seq_word_bit(z,j)
        == (if j == i { value } else { seq_word_bit(s,j) }) by {
        if j / 64 == i / 64 {
            checked_word_bits(s[j/64], (i%64) as u64, (j%64) as u64);
            assert((j%64 == i%64) == (j == i));
        }
    }
}
pub proof fn checked_seq_bit_update_at(s: Seq<u64>, i: int, value: bool, j: int)
    requires 0 <= i < s.len()*64, 0 <= j < s.len()*64,
    ensures seq_word_bit(seq_word_edit(s,i,value),j)
        == (if j == i { value } else { seq_word_bit(s,j) }),
{
    checked_seq_bit_update(s,i,value);
    assert(seq_word_bit(seq_word_edit(s,i,value),j)
        == (if j == i { value } else { seq_word_bit(s,j) }));
}

}
