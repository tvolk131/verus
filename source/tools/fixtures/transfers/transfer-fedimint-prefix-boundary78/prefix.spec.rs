pub open spec fn prefix_view(prefix: Seq<u8>, suffix: Seq<u8>) -> Seq<u8> {
    prefix + suffix
}

pub open spec fn is_prefix(prefix: Seq<u8>, whole: Seq<u8>) -> bool {
    prefix.len() <= whole.len()
        && forall|index: int| 0 <= index < prefix.len() ==> prefix[index] == whole[index]
}

pub open spec fn prefix_incomparable(left: Seq<u8>, right: Seq<u8>) -> bool {
    !is_prefix(left, right) && !is_prefix(right, left)
}

pub proof fn concatenation_has_prefix(prefix: Seq<u8>, suffix: Seq<u8>)
    ensures is_prefix(prefix, prefix_view(prefix, suffix)),
{}

pub proof fn suffix_cancellation(prefix: Seq<u8>, left: Seq<u8>, right: Seq<u8>)
    requires prefix_view(prefix, left) == prefix_view(prefix, right),
    ensures left == right,
{
    assert((prefix + left).len() == prefix.len() + left.len());
    assert((prefix + right).len() == prefix.len() + right.len());
    assert(left.len() == right.len());
    assert forall|index: int| 0 <= index < left.len() implies left[index] == right[index] by {
        assert((prefix + left)[prefix.len() + index] == left[index]);
        assert((prefix + right)[prefix.len() + index] == right[index]);
    }
    assert(left =~= right);
}

pub proof fn equal_keys_comparable(left: Seq<u8>, right: Seq<u8>, first: Seq<u8>, second: Seq<u8>)
    requires prefix_view(left, first) == prefix_view(right, second),
    ensures is_prefix(left, right) || is_prefix(right, left),
{
    if left.len() <= right.len() {
        assert forall|index: int| 0 <= index < left.len() implies left[index] == right[index] by {
            assert((left + first)[index] == left[index]);
            assert((right + second)[index] == right[index]);
        }
    } else {
        assert forall|index: int| 0 <= index < right.len() implies right[index] == left[index] by {
            assert((left + first)[index] == left[index]);
            assert((right + second)[index] == right[index]);
        }
    }
}

pub proof fn disjoint_namespaces(left: Seq<u8>, right: Seq<u8>, first: Seq<u8>, second: Seq<u8>)
    requires prefix_incomparable(left, right),
    ensures prefix_view(left, first) != prefix_view(right, second),
{
    if prefix_view(left, first) == prefix_view(right, second) {
        equal_keys_comparable(left, right, first, second);
    }
}

pub proof fn comparable_namespace_overlap()
    ensures !prefix_incomparable(seq![1u8], seq![1u8, 2u8]),
        prefix_view(seq![1u8], seq![2u8]) == prefix_view(seq![1u8, 2u8], Seq::<u8>::empty()),
{
    assert(seq![1u8] + seq![2u8] =~= seq![1u8, 2u8] + Seq::<u8>::empty());
}
