use vstd::prelude::*;
verus! {
pub proof fn subset_size<A>(a: Set<A>, b: Set<A>)
    requires a.subset_of(b),
    ensures a.len() <= b.len(),
{}
pub proof fn extensional<A>(a: Set<A>, b: Set<A>)
    requires forall|x: A| a.contains(x) == b.contains(x),
    ensures a == b,
{}
pub proof fn inclusion<A>(a: Set<A>, b: Set<A>)
    ensures a.union(b).len() + a.intersect(b).len() == a.len() + b.len(),
{}
// Immutable Spec signatures are essential; even an otherwise harmless mutable
// signature is skipped. This body needs no extra set facts.
proof fn unsupported_mutable(a: &mut Set<int>, b: Set<int>) {}
#[cfg(strict_false)]
proof fn false_cardinality(a: Set<int>, b: Set<int>)
    requires a.subset_of(b),
    ensures a.len() > b.len(),
{}
#[cfg(strict_false)]
proof fn false_equality(a: Set<int>, b: Set<int>)
    requires a.len() == b.len(),
    ensures a == b,
{}
#[cfg(strict_false)]
fn false_caller() { proof { subset_size(set![1int], Set::<int>::empty()); } }
}
