use vstd::prelude::*;
use f2_ordinary::*;
verus! {
// Observation wrapper for the two inspected signature-count admission guards.
// Explicit finite-set/count assumptions, no NodeMap or cryptographic model.
pub fn admission_observation(n: usize, ca: usize, cb: usize,
    Ghost(u): Ghost<Set<usize>>, Ghost(a): Ghost<Set<usize>>,
    Ghost(b): Ghost<Set<usize>>, Ghost(bad): Ghost<Set<usize>>)
    requires n > 0, u.len() == n, a.subset_of(u), b.subset_of(u), bad.subset_of(u),
        a.len() == ca, b.len() == cb, bad.len() <= fault(n as int),
{
    let peers = NumPeers::from(n);
    let q = peers.threshold();
    if ca < q { return; }
    if cb != q { return; }
    proof {
        quorum_intersection(n as int, u, a, b);
        quorum_has_honest(n as int, u, a, bad);
    }
    assert(a.intersect(b).len() > fault(n as int));
    assert(exists|p: usize| a.contains(p) && !bad.contains(p));
}

pub fn boundary_observations() {
    let zero = NumPeers::from(0);
    let n = zero.total();
    assert(n == 0);
    policy_client(1);
    policy_client(usize::MAX);
}
}
