//! Checked observations of mutable iterator referents.
//!
//! `current_values` observes the values at reference creation, whereas
//! `final_values` observes the values at the ends of the individual borrows.
//! Neither is a snapshot of the heap at an arbitrary later program point.
//! In particular, no untouched-suffix effect is inferred for an iterator that
//! has been dropped before exhaustion. These laws only transport checked facts.
use crate::vstd::prelude::*;
#[cfg(verus_keep_ghost)]
use crate::vstd::std_specs::iter::{IteratorSpec, VerusForLoopWrapper};

verus! {

pub open spec fn current_values<'a, T>(refs: Seq<&'a mut T>) -> Seq<T> {
    Seq::new(refs.len(), |i: int| *refs[i])
}

#[verifier::prophetic]
pub open spec fn final_values<'a, T>(refs: Seq<&'a mut T>) -> Seq<T> {
    Seq::new(refs.len(), |i: int| *final(refs[i]))
}

#[verifier::prophetic]
pub open spec fn written_prefix<'a, T>(
    refs: Seq<&'a mut T>,
    count: int,
    property: spec_fn(int, T) -> bool,
) -> bool {
    &&& 0 <= count <= refs.len()
    &&& forall|i: int| 0 <= i < count ==> property(i, *final(refs[i]))
}

pub broadcast proof fn lemma_current_index<'a, T>(refs: Seq<&'a mut T>, i: int)
    requires 0 <= i < refs.len(),
    ensures #[trigger] current_values(refs)[i] == *refs[i],
{
}

pub broadcast proof fn lemma_final_index<'a, T>(refs: Seq<&'a mut T>, i: int)
    requires 0 <= i < refs.len(),
    ensures #[trigger] final_values(refs)[i] == *final(refs[i]),
{
}

/// Convert the two pointwise relations supplied by slice::iter_mut to views.
pub proof fn lemma_observations<'a, T>(
    refs: Seq<&'a mut T>, initial: Seq<T>, eventual: Seq<T>,
)
    requires
        refs.len() == initial.len() == eventual.len(),
        forall|i: int| 0 <= i < refs.len() ==> *refs[i] == initial[i],
        forall|i: int| 0 <= i < refs.len() ==> *final(refs[i]) == eventual[i],
    ensures
        current_values(refs) == initial,
        final_values(refs) == eventual,
{
    assert(current_values(refs) =~= initial);
    assert(final_values(refs) =~= eventual);
}

/// A current write establishes its slot only after that reference's final
/// value is known. Equality with the current referent is a required premise.
pub proof fn lemma_write<'a, T>(
    refs: Seq<&'a mut T>, count: int, current: &'a mut T,
    property: spec_fn(int, T) -> bool,
)
    requires
        written_prefix(refs, count, property),
        count < refs.len(),
        current == refs[count],
        property(count, *final(current)),
    ensures written_prefix(refs, count + 1, property),
{
    assert forall|i: int| 0 <= i < count + 1 implies
        property(i, *final(refs[i])) by {
        if i < count {} else { assert(i == count); }
    }
}

/// Facts about already consumed references survive transport through an
/// unchanged prefix of reference identities; no claim about heap mutation.
pub proof fn lemma_prefix_frame<'a, T>(
    before: Seq<&'a mut T>, after: Seq<&'a mut T>, count: int,
    property: spec_fn(int, T) -> bool,
)
    requires
        written_prefix(before, count, property),
        count <= after.len(),
        forall|i: int| 0 <= i < count ==> before[i] == after[i],
    ensures written_prefix(after, count, property),
{
    assert forall|i: int| 0 <= i < count implies
        property(i, *final(after[i])) by {
        assert(before[i] == after[i]);
    }
}

pub proof fn lemma_prefix_observation<'a, T>(
    refs: Seq<&'a mut T>, eventual: Seq<T>, count: int,
    property: spec_fn(int, T) -> bool,
)
    requires
        written_prefix(refs, count, property),
        final_values(refs) == eventual,
    ensures
        0 <= count <= eventual.len(),
        forall|i: int| 0 <= i < count ==> property(i, eventual[i]),
{
    assert forall|i: int| 0 <= i < count implies property(i, eventual[i]) by {
        lemma_final_index(refs, i);
    }
}

/// The wrapper's history is exactly the consumed prefix, including the
/// just-yielded reference after next. This is not an extra iterator contract.
pub proof fn lemma_history<'a, T: 'a, I: Iterator<Item = &'a mut T>>(
    it: VerusForLoopWrapper<I>,
)
    requires it.wf(), it.iter.obeys_prophetic_iter_laws(),
    ensures
        final_values(it.history()) == final_values(it.seq()).subrange(0, it.index()),
        current_values(it.history()) == current_values(it.seq()).subrange(0, it.index()),
{
    assert(final_values(it.history()) =~= final_values(it.seq()).subrange(0, it.index()));
    assert(current_values(it.history()) =~= current_values(it.seq()).subrange(0, it.index()));
}

/// Observe a subrange of referents without changing its index origin.
pub proof fn lemma_final_subrange<'a, T>(refs: Seq<&'a mut T>, start: int, end: int)
    requires 0 <= start <= end <= refs.len(),
    ensures final_values(refs.subrange(start, end)) == final_values(refs).subrange(start, end),
{
    assert(final_values(refs.subrange(start, end)) =~= final_values(refs).subrange(start, end));
}

pub broadcast group group_observation {
    lemma_current_index,
    lemma_final_index,
}

} // verus!
