# Preregistered typed fallback interface

Base 38b09d18; primary Rust 58cf8f9c35b355199727627d07ae34a4e2846d67.
Exact primary files and body fragments are digest-bound in SOURCE_FREEZE.json.
The selected feature is stack-cache=true; expensive-consistency-checks=false.
The exact fallback for-loop and None are extracted into a separately named
method. The original cache-hit prelude is EXCLUDED and UNPROVED. This is not
whole find_granting_tagged, full enabled-cache lookup, or actual native Miri.
The original full method/helper are frozen as diagnostic baselines, not replaced.

Let B = old(self).borrows@, q: BorTag, a: AccessKind, g: usize.
BorTag is the genuine private NonZero<u64> wrapper, Item the private u64 wrapper;
Stack retains Vec<Item>, Option<BorTag>, Range<usize>, and BOTH 32-slot cache arrays.
Define wf_item(i) := (i.word() & (u64::MAX >> 3)) != 0.
Define match(i,q,a) := masked_word(i) == q.value() && decoded_permission(i)
grants a, with genuine equality and actual Permission::grants checked locally.
The query is arbitrary genuine BorTag (including high-bit and MAX), and access
is arbitrary Read OR Write; B is arbitrary finite length, including empty.
No uniqueness, nonempty, successful-hit, cache-hit, or low-query restriction.

Original structural cache relation C(c,B) := for every 0 <= j < 32,
c.idx[j] < B.len() && c.items[j] == B[c.idx[j]].
Use the ORIGINAL threshold invariant S(c,B) := B.len() > 32 ==> C(c,B).
This permits stale short-stack slots, exactly as in the previous cache component
and actual cache read guard. The optional expensive checker uses >=32 and is
excluded; its stronger threshold must not silently change the theorem.

Desired ordinary conditional INTERNAL fallback theorem:
requires (forall i in B: wf_item(i)) && S(old(self).cache,B).
ensures borrows, unknown_bottom, unique_range unchanged; S(final(cache),B);
Some(g) ==> g < B.len() && match(B[g],q,a)
            && forall j: g < j < B.len() ==> !match(B[j],q,a);
None ==> forall j: 0 <= j < B.len() ==> !match(B[j],q,a).
Write success must entail decoded Unique or SharedReadWrite.
Only cache may change. Preconditions are representation/structural predicates,
not assumptions of the conclusion, iterator lawfulness, future defined execution,
or any application restriction. Global establishment/preservation remains OPEN.

Immediate typed probe: for v: &Vec<u64>, e = v.iter().enumerate(),
r = e.rev(): prove r.obeys_prophetic_iter_laws(), its finite decrease and the
returned inner identity rev_iter(r) == e using only current checked contracts.
Separately test the literal readonly for (_,_) in v.iter().enumerate().rev().
No indexed replacement; no unchecked IteratorSpec bridge or trusted lawfulness.
Any failed ordinary probe is a support obligation, not a semantic control.
