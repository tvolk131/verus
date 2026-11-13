pub open spec fn is_prefix(prefix: Seq<u8>, key: Seq<u8>) -> bool {
    prefix.len() <= key.len()
        && forall|index: int| 0 <= index < prefix.len() ==> prefix[index] == key[index]
}

pub open spec fn incomparable(left: Seq<u8>, right: Seq<u8>) -> bool {
    !is_prefix(left, right) && !is_prefix(right, left)
}

pub open spec fn lookup(state: Map<Seq<u8>, Seq<u8>>, key: Seq<u8>) -> Option<Seq<u8>> {
    if state.dom().contains(key) { Some(state[key]) } else { None }
}

pub open spec fn backend_insert(
    before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>,
    target: Seq<u8>, value: Seq<u8>,
) -> bool {
    after == before.insert(target, value)
}

pub open spec fn outside_frame(
    before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>, prefix: Seq<u8>,
) -> bool {
    forall|other: Seq<u8>| !is_prefix(prefix, other) ==> lookup(before, other) == lookup(after, other)
}

pub open spec fn inside_frame(
    before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>, prefix: Seq<u8>,
) -> bool {
    forall|other: Seq<u8>| is_prefix(prefix, other) ==> lookup(before, other) == lookup(after, other)
}

pub proof fn prepared_insert_frame(
    before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>,
    prefix: Seq<u8>, key: Seq<u8>, value: Seq<u8>,
)
    requires backend_insert(before, after, prefix + key, value),
    ensures outside_frame(before, after, prefix),
        lookup(after, prefix + key) == Some(value),
        forall|other: Seq<u8>| other != prefix + key ==> lookup(before, other) == lookup(after, other),
{
    assert(is_prefix(prefix, prefix + key));
    assert forall|other: Seq<u8>| other != prefix + key implies lookup(before, other) == lookup(after, other) by {
        assert(after.dom().contains(other) == before.dom().contains(other));
    }
}

pub proof fn incomparable_target(prefix: Seq<u8>, peer: Seq<u8>, key: Seq<u8>)
    requires incomparable(prefix, peer),
    ensures !is_prefix(peer, prefix + key),
{
    if is_prefix(peer, prefix + key) {
        if prefix.len() <= peer.len() {
            assert forall|index: int| 0 <= index < prefix.len() implies prefix[index] == peer[index] by {
                assert((prefix + key)[index] == prefix[index]);
            }
            assert(is_prefix(prefix, peer));
        } else {
            assert forall|index: int| 0 <= index < peer.len() implies peer[index] == prefix[index] by {
                assert((prefix + key)[index] == prefix[index]);
            }
            assert(is_prefix(peer, prefix));
        }
    }
}

pub proof fn peer_insert_frame(
    before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>,
    prefix: Seq<u8>, peer: Seq<u8>, key: Seq<u8>, value: Seq<u8>,
)
    requires incomparable(prefix, peer), backend_insert(before, after, prefix + key, value),
    ensures inside_frame(before, after, peer),
{
    incomparable_target(prefix, peer, key);
    prepared_insert_frame(before, after, prefix, key, value);
}

pub fn prepare_insert(prefix: &Vec<u8>, key: &[u8], value: &[u8]) -> (target: Vec<u8>)
    ensures target@ == prefix@ + key@,
        forall|before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>|
            #[trigger] backend_insert(before, after, target@, value@) ==> outside_frame(before, after, prefix@),
{
    let target = key_contents(prefix, key);
    proof {
        assert forall|before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>|
            #[trigger] backend_insert(before, after, target@, value@) implies outside_frame(before, after, prefix@) by {
            prepared_insert_frame(before, after, prefix@, key@, value@);
        }
    }
    target
}

pub proof fn overlapping_namespace_witness()
    ensures !incomparable(seq![1u8], seq![1u8, 2u8]),
        seq![1u8] + seq![2u8] == seq![1u8, 2u8] + Seq::<u8>::empty(),
        !inside_frame(Map::<Seq<u8>, Seq<u8>>::empty(),
            Map::<Seq<u8>, Seq<u8>>::empty().insert(seq![1u8, 2u8], seq![9u8]), seq![1u8, 2u8]),
{
    assert(seq![1u8] + seq![2u8] =~= seq![1u8, 2u8]);
    assert(is_prefix(seq![1u8, 2u8], seq![1u8, 2u8]));
}

pub proof fn unframed_backend_witness()
    ensures !outside_frame(Map::<Seq<u8>, Seq<u8>>::empty(),
        Map::<Seq<u8>, Seq<u8>>::empty().insert(seq![7u8], seq![9u8]), seq![1u8]),
{
    assert((seq![1u8])[0] == 1u8);
    assert((seq![7u8])[0] == 7u8);
    assert(!is_prefix(seq![1u8], seq![7u8]));
}
