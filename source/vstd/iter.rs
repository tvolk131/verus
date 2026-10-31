//! Checked finite iterator consumers and index-preserving sparse slice traversal.
//! These are executable implementations, with no iterator axioms or termination
//! assumptions beyond the existing Iterator trait interface.
use crate::prelude::*;
#[cfg(verus_keep_ghost)]
use crate::std_specs::iter::IteratorSpec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
verus! {

broadcast use {
    crate::seq::group_seq_axioms,
    crate::seq::group_seq_lemmas,
    crate::slice::group_slice_axioms,
};

pub open spec fn sparse_entries<'a, T: 'a>(s: Seq<Option<T>>, start: nat) -> Seq<(usize, &'a T)>
    decreases s.len() - start,
{
    if start >= s.len() {
        seq![]
    } else {
        let tail = sparse_entries(s, start + 1);
        match s[start as int] {
            None => tail,
            Some(v) => seq![(start as usize, &v)] + tail,
        }
    }
}

pub proof fn sparse_entries_bound<'a, T: 'a>(s: Seq<Option<T>>, start: nat)
    ensures
        sparse_entries::<'a, T>(s, start).len() <= if start < s.len() {
            s.len() - start
        } else {
            0
        },
    decreases s.len() - start,
{
    if start < s.len() {
        sparse_entries_bound::<'a, T>(s, start + 1);
    }
}

pub struct SparseIter<'a, T> {
    items: &'a [Option<T>],
    cursor: usize,
}

#[cfg(verus_keep_ghost)]
impl<'a, T> crate::std_specs::iter::IteratorSpecImpl for SparseIter<'a, T> {
    open spec fn obeys_prophetic_iter_laws(&self) -> bool {
        true
    }

    closed spec fn remaining(&self) -> Seq<Self::Item> {
        sparse_entries(self.items@, self.cursor as nat)
    }

    open spec fn will_return_none(&self) -> bool {
        true
    }

    closed spec fn decrease(&self) -> Option<nat> {
        Some(
            if self.cursor < self.items.len() {
                (self.items.len() - self.cursor) as nat
            } else {
                0nat
            },
        )
    }

    closed spec fn peek(&self, i: int) -> Option<Self::Item> {
        if 0 <= i < sparse_entries::<'a, T>(self.items@, self.cursor as nat).len() {
            Some(sparse_entries::<'a, T>(self.items@, self.cursor as nat)[i])
        } else {
            None
        }
    }
}

impl<'a, T> SparseIter<'a, T> {
    pub fn new(items: &'a [Option<T>]) -> (r: Self)
        ensures
            r.decrease().is_some(),
            r.remaining() == sparse_entries(items@, 0),
            r.remaining().len() <= items.len(),
    {
        proof {
            sparse_entries_bound::<'a, T>(items@, 0);
        }
        Self { items, cursor: 0 }
    }
}

impl<'a, T> Iterator for SparseIter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let items = self.items;
        while self.cursor < items.len()
            invariant
                self.items == old(self).items,
                items == self.items,
                self.cursor >= old(self).cursor,
                sparse_entries::<'a, T>(items@, old(self).cursor as nat) == sparse_entries::<'a, T>(
                    items@,
                    self.cursor as nat,
                ),
            decreases items.len() - self.cursor,
        {
            let index = self.cursor;
            self.cursor += 1;
            match items[index].as_ref() {
                None => {},
                Some(value) => return Some((index, value)),
            }
        }
        None
    }
}

pub fn count<I: Iterator>(iter: I) -> (result: usize)
    requires
        iter.obeys_prophetic_iter_laws(),
        iter.decrease().is_some(),
        iter.remaining().len() <= usize::MAX as nat,
    ensures
        result == iter.remaining().len(),
        iter.will_return_none(),
{
    let mut rest = iter;
    let mut result = 0usize;
    loop
        invariant
            rest.obeys_prophetic_iter_laws(),
            rest.decrease().is_some(),
            rest.will_return_none() == iter.will_return_none(),
            result as int + rest.remaining().len() == iter.remaining().len(),
            iter.remaining().len() <= usize::MAX as nat,
        decreases rest.decrease().unwrap(),
    {
        match rest.next() {
            None => return result,
            Some(_) => {
                result += 1;
            },
        }
    }
}

/// The sparse sequence contains precisely the occupied source positions.
pub proof fn lemma_sparse_count<'a, T: 'a>(s: Seq<Option<T>>, start: nat)
    requires
        start <= s.len(),
    ensures
        sparse_entries::<'a, T>(s, start).len() + s.count_prefix(|v: Option<T>| v.is_some(), start)
            == s.count_prefix(|v: Option<T>| v.is_some(), s.len()),
    decreases s.len() - start,
{
    if start < s.len() {
        lemma_sparse_count::<'a, T>(s, start + 1);
    }
}

/// Collect a finite iterator using its checked next contract.
#[cfg(feature="alloc")]
pub fn collect<I: Iterator>(iter: I) -> (values: Vec<I::Item>)
    requires
        iter.obeys_prophetic_iter_laws(),
        iter.decrease().is_some(),
        iter.remaining().len() <= usize::MAX as nat,
    ensures
        values@ == iter.remaining(),
        iter.will_return_none(),
{
    broadcast use crate::std_specs::vec::group_vec_axioms;

    let mut rest = iter;
    let mut values = Vec::new();
    loop
        invariant
            rest.obeys_prophetic_iter_laws(),
            rest.decrease().is_some(),
            rest.will_return_none() == iter.will_return_none(),
            values@ + rest.remaining() == iter.remaining(),
            iter.remaining().len() <= usize::MAX as nat,
        decreases rest.decrease().unwrap(),
    {
        match rest.next() {
            None => return values,
            Some(value) => {
                values.push(value);
            },
        }
    }
}

/// Preserve empty slots and indices while transforming only occupied values.
#[cfg(feature="alloc")]
pub fn map_sparse<T, U, F: Fn(&T) -> U>(items: &[Option<T>], f: F) -> (values: Vec<Option<U>>)
    requires
        forall|i: int|
            0 <= i < items@.len() && items@[i].is_some() ==> f.requires((&items@[i].unwrap(),)),
    ensures
        values.len() == items.len(),
        forall|i: int|
            #![trigger values@[i]]
            0 <= i < items@.len() ==> match items@[i] {
                None => values@[i].is_none(),
                Some(value) => values@[i].is_some() && f.ensures((&value,), values@[i].unwrap()),
            },
{
    broadcast use crate::std_specs::vec::group_vec_axioms;

    let mut values: Vec<Option<U>> = Vec::new();
    let mut i = 0usize;
    while i < items.len()
        invariant
            i == values@.len(),
            i <= items.len(),
            forall|j: int|
                #![trigger values@[j]]
                0 <= j < i ==> match items@[j] {
                    None => values@[j].is_none(),
                    Some(value) => values@[j].is_some() && f.ensures(
                        (&value,),
                        values@[j].unwrap(),
                    ),
                },
            forall|j: int|
                0 <= j < items@.len() && items@[j].is_some() ==> f.requires((&items@[j].unwrap(),)),
        decreases items.len() - i,
    {
        let value = match items[i].as_ref() {
            None => None,
            Some(value) => Some(f(value)),
        };
        values.push(value);
        i += 1;
    }
    values
}

/// Count a checked predicate over a finite iterator. The supplied mathematical
/// predicate is connected to the executable callback by an ordinary contract.
pub fn count_where<I: Iterator, F: Fn(&I::Item) -> bool>(
    iter: I,
    predicate: F,
    Ghost(matches): Ghost<spec_fn(I::Item) -> bool>,
) -> (result: usize)
    requires
        iter.obeys_prophetic_iter_laws(),
        iter.decrease().is_some(),
        iter.remaining().len() <= usize::MAX as nat,
        forall|i: int|
            0 <= i < iter.remaining().len() ==> predicate.requires((&iter.remaining()[i],)),
        forall|i: int, b: bool|
            0 <= i < iter.remaining().len() && predicate.ensures((&iter.remaining()[i],), b) ==> b
                == matches(iter.remaining()[i]),
    ensures
        result == iter.remaining().count_prefix(matches, iter.remaining().len()),
        iter.will_return_none(),
{
    let mut rest = iter;
    let mut result = 0usize;
    let ghost mut seen: nat = 0;
    loop
        invariant
            rest.obeys_prophetic_iter_laws(),
            rest.decrease().is_some(),
            rest.will_return_none() == iter.will_return_none(),
            seen <= iter.remaining().len(),
            rest.remaining() == iter.remaining()[seen as int..],
            result == iter.remaining().count_prefix(matches, seen),
            result <= seen,
            iter.remaining().len() <= usize::MAX as nat,
            forall|i: int|
                0 <= i < iter.remaining().len() ==> predicate.requires((&iter.remaining()[i],)),
            forall|i: int, b: bool|
                0 <= i < iter.remaining().len() && predicate.ensures((&iter.remaining()[i],), b)
                    ==> b == matches(iter.remaining()[i]),
        decreases rest.decrease().unwrap(),
    {
        match rest.next() {
            None => return result,
            Some(value) => {
                if predicate(&value) {
                    result += 1;
                }
                proof {
                    seen = seen + 1;
                }
            },
        }
    }
}

/// Return the first matching element, with an explicit absence/earliest witness.
pub fn find<I: Iterator, F: Fn(&I::Item) -> bool>(
    iter: I,
    predicate: F,
    Ghost(matches): Ghost<spec_fn(I::Item) -> bool>,
) -> (result: Option<I::Item>)
    requires
        iter.obeys_prophetic_iter_laws(),
        iter.decrease().is_some(),
        forall|i: int|
            0 <= i < iter.remaining().len() ==> predicate.requires((&iter.remaining()[i],)),
        forall|i: int, b: bool|
            0 <= i < iter.remaining().len() && predicate.ensures((&iter.remaining()[i],), b) ==> b
                == matches(iter.remaining()[i]),
    ensures
        result.is_some() == exists|i: int|
            0 <= i < iter.remaining().len() && matches(iter.remaining()[i]),
        result.is_some() ==> exists|i: int|
            0 <= i < iter.remaining().len() && result.unwrap() == iter.remaining()[i] && matches(
                iter.remaining()[i],
            ) && forall|j: int| 0 <= j < i ==> !matches(iter.remaining()[j]),
        result.is_none() ==> iter.will_return_none(),
{
    let mut rest = iter;
    let ghost mut seen: nat = 0;
    loop
        invariant
            rest.obeys_prophetic_iter_laws(),
            rest.decrease().is_some(),
            rest.will_return_none() == iter.will_return_none(),
            seen <= iter.remaining().len(),
            rest.remaining() == iter.remaining()[seen as int..],
            forall|j: int| 0 <= j < seen ==> !matches(iter.remaining()[j]),
            forall|i: int|
                0 <= i < iter.remaining().len() ==> predicate.requires((&iter.remaining()[i],)),
            forall|i: int, b: bool|
                0 <= i < iter.remaining().len() && predicate.ensures((&iter.remaining()[i],), b)
                    ==> b == matches(iter.remaining()[i]),
        decreases rest.decrease().unwrap(),
    {
        match rest.next() {
            None => return None,
            Some(value) => {
                if predicate(&value) {
                    return Some(value);
                }
                proof {
                    seen = seen + 1;
                }
            },
        }
    }
}

} // verus!
