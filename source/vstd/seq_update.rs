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

/// Pointwise-compatible predicates give equal positional counts, even when the
/// two sequences have different element types.
pub broadcast proof fn lemma_count_prefix_congruence<A, B>(
    left: Seq<A>, selected: spec_fn(A) -> bool,
    right: Seq<B>, keep: spec_fn(B) -> bool, n: nat,
)
    requires
        n <= left.len(), n <= right.len(),
        forall|i: int| 0 <= i < n ==> selected(#[trigger] left[i]) == keep(right[i]),
    ensures
        #[trigger] left.count_prefix(selected, n) == #[trigger] right.count_prefix(keep, n),
    decreases n,
{
    if n > 0 {
        lemma_count_prefix_congruence(left, selected, right, keep, (n - 1) as nat);
    }
}

// Connect positional accounting to a sparse iterator without requiring the
// original elements, projected slots and iterator items to have the same type.
proof fn sparse_filter_count_prefix<S, A, E, B>(
    original: Seq<S>,
    selected: spec_fn(S) -> bool,
    slots: Seq<Option<A>>,
    label: spec_fn(int) -> E,
    project: spec_fn(E) -> B,
    keep: spec_fn(B) -> bool,
    n: int,
)
    requires
        original.len() == slots.len(),
        0 <= n <= slots.len(),
        forall|i: int| 0 <= i < slots.len() ==>
            selected(#[trigger] original[i]) == (slots[i].is_some() && keep(project(label(i)))),
    ensures
        original.count_prefix(selected, n as nat)
            == sparse_entries(slots.take(n), label).map_values(project).filter(keep).len(),
    decreases n,
{
    use crate::std_specs::iter_adapters::flatten_options;
    if n == 0 {
        broadcast use crate::seq_lib::group_filter_ensures;
        assert(sparse_entries(slots.take(n), label) =~= Seq::empty());
        assert(sparse_entries(slots.take(n), label).map_values(project) =~= Seq::empty());
        reveal(Seq::filter);
    } else {
        sparse_filter_count_prefix(original, selected, slots, label, project, keep, n - 1);
        let previous = Seq::new((n - 1) as nat, |i: int|
            match slots[i] { None => None, Some(_) => Some(label(i)) });
        let last = match slots[n - 1] { None => None, Some(_) => Some(label(n - 1)) };
        assert(Seq::new(slots.take(n).len(), |i: int|
            match slots.take(n)[i] { None => None, Some(_) => Some(label(i)) })
            =~= previous.push(last));
        assert(Seq::new(slots.take(n - 1).len(), |i: int|
            match slots.take(n - 1)[i] { None => None, Some(_) => Some(label(i)) })
            =~= previous);
        lemma_sparse_entries_eq(slots.take(n), label, previous.push(last));
        lemma_sparse_entries_eq(slots.take(n - 1), label, previous);
        flatten_push(previous, last);
        let before = sparse_entries(slots.take(n - 1), label);
        assert(selected(original[n - 1]) == (slots[n - 1].is_some() && keep(project(label(n - 1)))));
        if slots[n - 1].is_some() {
            assert(sparse_entries(slots.take(n), label) == before.push(label(n - 1)));
            before.lemma_push_map_commute(project, label(n - 1));
            before.map_values(project).lemma_filter_push(project(label(n - 1)), keep);
        } else {
            assert(sparse_entries(slots.take(n), label) == before);
        }
    }
}

/// Counting slots agrees with a mapped and filtered sparse observation whenever
/// their per-position predicates agree. This proves only the supplied relation.
pub broadcast proof fn lemma_sparse_map_filter_count<S, A, E, B>(
    original: Seq<S>, selected: spec_fn(S) -> bool,
    slots: Seq<Option<A>>, label: spec_fn(int) -> E,
    project: spec_fn(E) -> B, keep: spec_fn(B) -> bool,
)
    requires
        original.len() == slots.len(),
        forall|i: int| 0 <= i < slots.len() ==>
            selected(#[trigger] original[i]) == (slots[i].is_some() && keep(project(label(i)))),
    ensures
        #[trigger] original.count_prefix(selected, original.len())
            == #[trigger] sparse_entries(slots, label).map_values(project).filter(keep).len(),
{
    sparse_filter_count_prefix(original, selected, slots, label, project, keep, slots.len() as int);
    assert(slots.take(slots.len() as int) =~= slots);
}

pub broadcast proof fn lemma_sparse_filter_count<S, A, E>(
    original: Seq<S>, selected: spec_fn(S) -> bool,
    slots: Seq<Option<A>>, label: spec_fn(int) -> E, keep: spec_fn(E) -> bool,
)
    requires
        original.len() == slots.len(),
        forall|i: int| 0 <= i < slots.len() ==>
            selected(#[trigger] original[i]) == (slots[i].is_some() && keep(label(i))),
    ensures
        #[trigger] original.count_prefix(selected, original.len())
            == #[trigger] sparse_entries(slots, label).filter(keep).len(),
{
    lemma_sparse_map_filter_count(original, selected, slots, label, |e: E| e, keep);
    assert(sparse_entries(slots, label).map_values(|e: E| e) =~= sparse_entries(slots, label));
}

proof fn filter_identity<E>(values: Seq<E>)
    ensures values.filter(|e: E| true) == values,
    decreases values.len(),
{
    reveal(Seq::filter);
    if values.len() > 0 {
        filter_identity(values.drop_last());
        assert(values.drop_last().push(values.last()) =~= values);
    }
}

pub broadcast proof fn lemma_sparse_entries_count<A, E>(
    slots: Seq<Option<A>>, label: spec_fn(int) -> E,
)
    ensures
        #[trigger] sparse_entries(slots, label).len()
            == slots.count_prefix(|slot: Option<A>| slot.is_some(), slots.len()),
{
    lemma_sparse_filter_count(slots, |slot: Option<A>| slot.is_some(), slots, label, |e: E| true);
    filter_identity(sparse_entries(slots, label));
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
