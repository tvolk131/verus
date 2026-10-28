//! Checked observations of folds that update indexed sequence slots.
use crate::prelude::*;

verus! {

broadcast use crate::seq::group_seq_axioms;

pub open spec fn scatter<A, B>(
    entries: Seq<A>,
    initial: Seq<B>,
    index: spec_fn(A) -> int,
    value: spec_fn(A) -> B,
) -> Seq<B> {
    entries.fold_left(initial, |acc: Seq<B>, entry: A| acc.update(index(entry), value(entry)))
}

/// Different pointwise-equal step functions yield the same complete fold.
pub broadcast proof fn lemma_fold_left_congruence<A, B>(
    entries: Seq<A>,
    initial: B,
    f: spec_fn(B, A) -> B,
    g: spec_fn(B, A) -> B,
)
    requires
        forall|acc: B, entry: A| #[trigger] f(acc, entry) == g(acc, entry),
    ensures
        #[trigger] entries.fold_left(initial, f) == #[trigger] entries.fold_left(initial, g),
    decreases entries.len(),
{
    reveal_with_fuel(Seq::fold_left, 2);
    if entries.len() > 0 {
        lemma_fold_left_congruence(entries.drop_last(), initial, f, g);
    }
}

/// An explicit collection observation, reusable by checked sparse iterator factories.
pub open spec fn sparse_entries<A, E>(source: Seq<Option<A>>, label: spec_fn(int) -> E) -> Seq<E> {
    crate::std_specs::iter_adapters::flatten_options(
        Seq::new(
            source.len(),
            |i: int|
                match source[i] {
                    None => None,
                    Some(v) => Some(label(i)),
                },
        ),
    )
}

/// Relate an explicit indexed representation to the canonical sparse view.
pub proof fn lemma_sparse_entries_eq<A, E>(
    source: Seq<Option<A>>,
    label: spec_fn(int) -> E,
    full: Seq<Option<E>>,
)
    requires
        full.len() == source.len(),
        forall|i: int|
            0 <= i < source.len() ==> #[trigger] full[i] == match source[i] {
                None => None,
                Some(_) => Some(label(i)),
            },
    ensures
        sparse_entries(source, label) == crate::std_specs::iter_adapters::flatten_options(full),
{
    assert(Seq::new(
        source.len(),
        |i: int|
            match source[i] {
                None => None,
                Some(v) => Some(label(i)),
            },
    ) =~= full);
}

pub broadcast proof fn lemma_sparse_entries_empty<A, E>(
    source: Seq<Option<A>>,
    label: spec_fn(int) -> E,
)
    requires
        forall|i: int| 0 <= i < source.len() ==> (#[trigger] source[i]).is_none(),
    ensures
        #[trigger] sparse_entries(source, label).len() == 0,
{
    let full = Seq::new(
        source.len(),
        |i: int|
            match source[i] {
                None => None,
                Some(v) => Some(label(i)),
            },
    );
    full.lemma_all_neg_filter_empty(|v: Option<E>| v.is_some());
}

proof fn flatten_push<E>(values: Seq<Option<E>>, last: Option<E>)
    ensures
        crate::std_specs::iter_adapters::flatten_options(values.push(last)) == match last {
            None => crate::std_specs::iter_adapters::flatten_options(values),
            Some(v) => crate::std_specs::iter_adapters::flatten_options(values).push(v),
        },
{
    use crate::std_specs::iter_adapters::flatten_options;
    values.lemma_filter_push(last, |v: Option<E>| v.is_some());
    if last.is_some() {
        assert(flatten_options(values.push(last)) =~= flatten_options(values).push(last.unwrap()));
    }
}

proof fn scatter_sparse_prefix<A, E, B>(
    source: Seq<Option<A>>,
    initial: Seq<B>,
    label: spec_fn(int) -> E,
    index: spec_fn(E) -> int,
    value: spec_fn(E) -> B,
    n: int,
)
    requires
        initial.len() == source.len(),
        0 <= n <= source.len(),
        forall|i: int|
            0 <= i < source.len() && source[i].is_some() ==> index(#[trigger] label(i)) == i,
    ensures
        scatter(sparse_entries(source.take(n), label), initial, index, value) == Seq::new(
            source.len(),
            |i: int|
                if i < n && source[i].is_some() {
                    value(label(i))
                } else {
                    initial[i]
                },
        ),
    decreases n,
{
    use crate::std_specs::iter_adapters::flatten_options;
    reveal_with_fuel(Seq::fold_left, 2);
    if n == 0 {
        broadcast use crate::seq_lib::group_filter_ensures;

        assert(sparse_entries(source.take(n), label) =~= Seq::empty());
        assert(initial =~= Seq::new(source.len(), |i: int| initial[i]));
    } else {
        scatter_sparse_prefix(source, initial, label, index, value, n - 1);
        let previous = Seq::new(
            (n - 1) as nat,
            |i: int|
                match source[i] {
                    None => None,
                    Some(v) => Some(label(i)),
                },
        );
        let last = match source[n - 1] {
            None => None,
            Some(v) => Some(label(n - 1)),
        };
        assert(Seq::new(
            n as nat,
            |i: int|
                match source[i] {
                    None => None,
                    Some(v) => Some(label(i)),
                },
        ) =~= previous.push(last));
        assert(Seq::new(
            source.take(n).len(),
            |i: int|
                match source.take(n)[i] {
                    None => None,
                    Some(v) => Some(label(i)),
                },
        ) =~= previous.push(last));
        assert(Seq::new(
            source.take(n - 1).len(),
            |i: int|
                match source.take(n - 1)[i] {
                    None => None,
                    Some(v) => Some(label(i)),
                },
        ) =~= previous);
        flatten_push(previous, last);
        let before = scatter(sparse_entries(source.take(n - 1), label), initial, index, value);
        let after = scatter(sparse_entries(source.take(n), label), initial, index, value);
        if source[n - 1].is_some() {
            assert(sparse_entries(source.take(n), label) == sparse_entries(
                source.take(n - 1),
                label,
            ).push(label(n - 1)));
            assert(sparse_entries(source.take(n), label).drop_last() =~= sparse_entries(
                source.take(n - 1),
                label,
            ));
            assert(after == before.update(n - 1, value(label(n - 1))));
        } else {
            assert(after == before);
        }
        assert(after =~= Seq::new(
            source.len(),
            |i: int|
                if i < n && source[i].is_some() {
                    value(label(i))
                } else {
                    initial[i]
                },
        ));
    }
}

pub broadcast proof fn lemma_scatter_sparse<A, E, B>(
    source: Seq<Option<A>>,
    initial: Seq<B>,
    label: spec_fn(int) -> E,
    index: spec_fn(E) -> int,
    value: spec_fn(E) -> B,
)
    requires
        initial.len() == source.len(),
        forall|i: int|
            0 <= i < source.len() && source[i].is_some() ==> index(#[trigger] label(i)) == i,
    ensures
        #[trigger] scatter(sparse_entries(source, label), initial, index, value) == Seq::new(
            source.len(),
            |i: int|
                match source[i] {
                    None => initial[i],
                    Some(v) => value(label(i)),
                },
        ),
{
    scatter_sparse_prefix(source, initial, label, index, value, source.len() as int);
    assert(source.take(source.len() as int) =~= source);
}

/// The last entry writing a slot. Repeated indices have ordinary last-write semantics.
pub open spec fn last_write<A>(entries: Seq<A>, index: spec_fn(A) -> int, slot: int) -> Option<A>
    decreases entries.len(),
{
    if entries.len() == 0 {
        None
    } else if index(entries.last()) == slot {
        Some(entries.last())
    } else {
        last_write(entries.drop_last(), index, slot)
    }
}

pub broadcast proof fn lemma_last_write<A>(entries: Seq<A>, index: spec_fn(A) -> int, slot: int)
    ensures
        match #[trigger] last_write(entries, index, slot) {
            Some(entry) => entries.contains(entry) && index(entry) == slot,
            None => forall|i: int| 0 <= i < entries.len() ==> index(entries[i]) != slot,
        },
    decreases entries.len(),
{
    broadcast use crate::seq::group_seq_axioms;

    reveal_with_fuel(last_write, 2);
    if entries.len() > 0 && index(entries.last()) != slot {
        lemma_last_write(entries.drop_last(), index, slot);
        if last_write(entries, index, slot).is_none() {
            assert forall|i: int| 0 <= i < entries.len() implies index(entries[i]) != slot by {
                if i < entries.len() - 1 {
                    assert(entries.drop_last()[i] == entries[i]);
                }
            }
        }
    }
}

pub broadcast proof fn lemma_scatter_len<A, B>(
    entries: Seq<A>,
    initial: Seq<B>,
    index: spec_fn(A) -> int,
    value: spec_fn(A) -> B,
)
    requires
        forall|i: int| 0 <= i < entries.len() ==> 0 <= index(#[trigger] entries[i]) < initial.len(),
    ensures
        #[trigger] scatter(entries, initial, index, value).len() == initial.len(),
    decreases entries.len(),
{
    broadcast use crate::seq::group_seq_axioms;

    reveal_with_fuel(Seq::fold_left, 2);
    if entries.len() > 0 {
        lemma_scatter_len(entries.drop_last(), initial, index, value);
    }
}

pub broadcast proof fn lemma_scatter_index<A, B>(
    entries: Seq<A>,
    initial: Seq<B>,
    index: spec_fn(A) -> int,
    value: spec_fn(A) -> B,
    slot: int,
)
    requires
        0 <= slot < initial.len(),
        forall|i: int| 0 <= i < entries.len() ==> 0 <= index(#[trigger] entries[i]) < initial.len(),
    ensures
        #[trigger] scatter(entries, initial, index, value)[slot] == match last_write(
            entries,
            index,
            slot,
        ) {
            Some(entry) => value(entry),
            None => initial[slot],
        },
    decreases entries.len(),
{
    broadcast use crate::seq::group_seq_axioms;

    reveal_with_fuel(Seq::fold_left, 2);
    reveal_with_fuel(last_write, 2);
    if entries.len() > 0 {
        lemma_scatter_len(entries.drop_last(), initial, index, value);
        lemma_scatter_index(entries.drop_last(), initial, index, value, slot);
    }
}

} // verus!
