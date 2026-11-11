use vstd::prelude::*;
verus! {
spec fn selected(s: Seq<Set<int>>, i: int) -> Set<int>
    recommends 0 <= i < s.len(),
{ s[i] }

// The original guard discharges all uses' recommendation, even for empty s.
proof fn guarded(s: Seq<Set<int>>, i: int, b: Set<int>)
    ensures 0 <= i < s.len() ==> selected(s,i).union(b).contains(7) == (selected(s,i).contains(7) || b.contains(7)),
{}

// Positive-domain control: the same prefix argument is legal at entry.
proof fn entry_guard(s: Seq<Set<int>>, i: int, b: Set<int>)
    requires 0 <= i < s.len(),
    ensures selected(s,i).union(b).contains(7) == (selected(s,i).contains(7) || b.contains(7)),
{}

// Spec indexing is total logically but has an index recommendation.
proof fn guarded_index(s: Seq<Set<int>>, i: int, b: Set<int>)
    ensures 0 <= i < s.len() ==> s[i].union(b).contains(9) == (s[i].contains(9) || b.contains(9)),
{}

proof fn nested<A>(a: Set<Seq<Option<A>>>, b: Set<Seq<Option<A>>>)
    requires a.subset_of(b),
    ensures a.len() <= b.len(),
{ vstd::set_lib::lemma_len_subset(a,b); }
proof fn references(a: &Set<int>, b: &Set<int>)
    ensures (*a).union(*b).contains(0) == (a.contains(0) || b.contains(0)),
{}

// Too many terms/pairs must leave a healthy existing proof alone.
proof fn over_budget(a:Set<int>,b:Set<int>,c:Set<int>,d:Set<int>,e:Set<int>,f:Set<int>,g:Set<int>,h:Set<int>,i:Set<int>)
    ensures a==a,b==b,c==c,d==d,e==e,f==f,g==g,h==h,i==i,
{}
// Bound-local Set observations cannot escape their scope.
proof fn local_scope()
    ensures forall|s:Set<int>| #[trigger] s.len() >= 0,
{}
}
