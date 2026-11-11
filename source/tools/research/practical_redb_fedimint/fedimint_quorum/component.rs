// Frozen F2 source-connected projection; executable method text is pinned.
use vstd::prelude::*;

#[verus_verify]
#[derive(Copy, Clone)]
pub struct NumPeers(usize);

verus! {
impl NumPeers {
    // Closed: public clients observe the count via checked method contracts.
    pub closed spec fn count(self) -> int { self.0 as int }
}
pub open spec fn fault(n: int) -> int { (n - 1) / 3 }
pub open spec fn quorum(n: int) -> int { n - fault(n) }
}

#[verus_verify]
impl NumPeers {
    #[verus_spec(r => ensures r == self.count())]
    pub fn total(self) -> usize {
        self.0
    }

    #[verus_spec(r => requires self.count() > 0,
        ensures r == fault(self.count()), 3 * r < self.count(), self.count() <= 3 * r + 3)]
    pub fn max_evil(self) -> usize {
        (self.total() - 1) / 3
    }

    #[verus_spec(r => requires self.count() > 0,
        ensures r == fault(self.count()) + 1)]
    pub fn one_honest(self) -> usize {
        self.max_evil() + 1
    }

    #[verus_spec(r => requires self.count() > 0,
        ensures r == quorum(self.count()) - 1)]
    pub fn degree(self) -> usize {
        self.threshold() - 1
    }

    #[verus_spec(r => requires self.count() > 0,
        ensures r == quorum(self.count()), 1 <= r <= self.count())]
    pub fn threshold(self) -> usize {
        self.total() - self.max_evil()
    }
}

#[verus_verify]
impl From<usize> for NumPeers {
    #[verus_spec(r => ensures r.count() == value)]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

verus! {
// Semantic BFT policy assumptions are explicit; all set arithmetic is mechanical.
pub proof fn quorum_intersection(n: int, u: Set<usize>, a: Set<usize>, b: Set<usize>)
    requires n > 0, u.len() == n, a.subset_of(u), b.subset_of(u),
        a.len() >= quorum(n), b.len() >= quorum(n),
    ensures a.intersect(b).len() > fault(n),
{
    // MANUAL_INTERSECTION: unsolved baseline slot, never claimed as a proof.
}

pub proof fn quorum_has_honest(n: int, u: Set<usize>, a: Set<usize>, bad: Set<usize>)
    requires n > 0, u.len() == n, a.subset_of(u), bad.subset_of(u),
        a.len() >= quorum(n), bad.len() <= fault(n),
    ensures !a.subset_of(bad), exists|p: usize| a.contains(p) && !bad.contains(p),
{
    // MANUAL_HONEST: unsolved baseline slot, never claimed as a proof.
}

pub proof fn one_honest_margin(n: int, u: Set<usize>, chosen: Set<usize>, bad: Set<usize>)
    requires n > 0, u.len() == n, chosen.subset_of(u), bad.subset_of(u),
        chosen.len() >= fault(n) + 1, bad.len() <= fault(n),
    ensures exists|p: usize| chosen.contains(p) && !bad.contains(p),
{
    // MANUAL_MARGIN: unsolved baseline slot, never claimed as a proof.
}

// Numeric observation client, not an implementation of either cryptographic guard.
pub fn policy_client(n: usize)
    requires n > 0,
{
    let peers = NumPeers::from(n);
    let total = peers.total();
    let f = peers.max_evil();
    let h = peers.one_honest();
    let q = peers.threshold();
    let d = peers.degree();
    assert(total == n);
    assert(f == fault(n as int));
    assert(3 * f < n <= 3 * f + 3);
    assert(1 <= q <= n);
    assert(q == n - f);
    assert(h == f + 1);
    assert(d == q - 1);
}
}
