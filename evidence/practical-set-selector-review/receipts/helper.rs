use vstd::prelude::*;
use vstd::set_lib::*;
verus! {
pub proof fn lemma_set_pair_observations<A>(a: Set<A>, b: Set<A>)
    ensures
        a.subset_of(b) ==> a.len() <= b.len(),
        a.union(b).len() + a.intersect(b).len() == a.len() + b.len(),
        (a == b) == (forall|x: A| a.contains(x) == b.contains(x)),
{
    lemma_set_intersect_union_lens(a, b);
    if a.subset_of(b) { lemma_len_subset(a, b); }
    if forall|x: A| a.contains(x) == b.contains(x) { assert(a =~= b); }
}

}
