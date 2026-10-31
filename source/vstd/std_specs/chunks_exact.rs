//! Standard-library boundary for immutable exact chunks and slice/array copy.
//! These contracts model Rust core, not an application's iterator wrappers.
use crate::prelude::*;
use crate::std_specs::iter::*;
use core::slice::ChunksExact;

verus! {

#[verifier::external_type_specification]
#[verifier::external_body]
#[verifier::accept_recursive_types(T)]
pub struct ExChunksExact<'a, T: 'a>(ChunksExact<'a, T>);

pub uninterp spec fn chunks<'a, T: 'a>(i: ChunksExact<'a, T>) -> Seq<&'a [T]>;

pub uninterp spec fn width<'a, T: 'a>(i: ChunksExact<'a, T>) -> usize;

impl<'a, T: 'a> IteratorSpecImpl for ChunksExact<'a, T> {
    open spec fn has_exact_size_hint(&self) -> bool {
        true
    }

    open spec fn obeys_prophetic_iter_laws(&self) -> bool {
        true
    }

    open spec fn remaining(&self) -> Seq<Self::Item> {
        chunks(*self)
    }

    open spec fn will_return_none(&self) -> bool {
        true
    }

    open spec fn decrease(&self) -> Option<nat> {
        Some(chunks(*self).len())
    }

    open spec fn peek(&self, index: int) -> Option<Self::Item> {
        if 0 <= index < chunks(*self).len() {
            Some(chunks(*self)[index])
        } else {
            None
        }
    }
}

impl<'a, T: 'a> ExactSizeIteratorSpecImpl for ChunksExact<'a, T> {
    open spec fn exact_len(&self) -> usize {
        chunks(*self).len() as usize
    }
}

pub assume_specification<'a, T>[ <[T]>::chunks_exact ](s: &'a [T], k: usize) -> (r: ChunksExact<
    'a,
    T,
>)
    requires
        k > 0,
    ensures
        width(r) == k,
        chunks(r).len() <= usize::MAX,
        chunks(r).len() == s@.len() / (k as nat),
        forall|i: int|
            0 <= i < chunks(r).len() ==> #[trigger] chunks(r)[i]@ == s@.subrange(
                i * k,
                (i + 1) * k,
            ),
        forall|i: int| 0 <= i < chunks(r).len() ==> #[trigger] chunks(r)[i]@.len() == k,
;

pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Iterator>::next ](
    s: &mut ChunksExact<'a, T>,
) -> (r: Option<&'a [T]>)
    ensures
        width(*final(s)) == width(*old(s)),
        r == if chunks(*old(s)).len() > 0 {
            Some(chunks(*old(s))[0])
        } else {
            None
        },
        chunks(*final(s)) == if chunks(*old(s)).len() > 0 {
            chunks(*old(s)).drop_first()
        } else {
            Seq::empty()
        },
    no_unwind
;

pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Iterator>::nth ](
    s: &mut ChunksExact<'a, T>,
    n: usize,
) -> (r: Option<<ChunksExact<'a, T> as Iterator>::Item>)
    ensures
        width(*final(s)) == width(*old(s)),
        r == if n < chunks(*old(s)).len() {
            Some(chunks(*old(s))[n as int])
        } else {
            None
        },
        chunks(*final(s)) == if n < chunks(*old(s)).len() {
            chunks(*old(s)).subrange(n as int + 1, chunks(*old(s)).len() as int)
        } else {
            Seq::empty()
        },
    no_unwind
;

pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Iterator>::size_hint ](
    s: &ChunksExact<'a, T>,
) -> (r: (usize, Option<usize>))
    ensures
        r == (chunks(*s).len() as usize, Some(chunks(*s).len() as usize)),
;

pub assume_specification<'a, T>[ <ChunksExact<'a, T> as Clone>::clone ](
    s: &ChunksExact<'a, T>,
) -> (r: ChunksExact<'a, T>)
    ensures
        chunks(r) == chunks(*s),
        width(r) == width(*s),
;

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExTryFromSliceError(core::array::TryFromSliceError);

pub assume_specification<'a, T: Copy, const N: usize>[ <[T; N] as core::convert::TryFrom<
    &'a [T],
>>::try_from ](s: &[T]) -> (r: Result<[T; N], core::array::TryFromSliceError>)
    ensures
        r.is_ok() <==> s@.len() == N,
        r.is_ok() ==> r->Ok_0@ == s@,
;

} // verus!
