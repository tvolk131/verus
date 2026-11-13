use vstd::prelude::*;
use insert_link80a::*;
verus! {
pub fn imported_preparation(prefix: &Vec<u8>, key: &[u8], value: &[u8]) -> (target: Vec<u8>)
    ensures target@ == prefix@ + key@,
        forall|before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>|
            #[trigger] backend_insert(before, after, target@, value@) ==> outside_frame(before, after, prefix@),
{
    prepare_insert(prefix, key, value)
}

pub proof fn imported_peer_frame(
    before: Map<Seq<u8>, Seq<u8>>, after: Map<Seq<u8>, Seq<u8>>,
    prefix: Seq<u8>, peer: Seq<u8>, key: Seq<u8>, value: Seq<u8>,
)
    requires incomparable(prefix, peer), backend_insert(before, after, prefix + key, value),
    ensures inside_frame(before, after, peer),
{
    peer_insert_frame(before, after, prefix, peer, key, value);
}
}
