//! Native adapter models. These declarations extend the trusted Rust-library
//! interface; they do not claim to verify the precompiled core implementation.
//! Count support is closed under known native adapters starting at slice::Iter.
//! Iterator::next laws alone do not constrain user overrides of fold/count.
use super::iter::{IteratorSpec, filter_fun, filter_iter, map_fun, map_iter};
use crate::prelude::*;
use core::iter::{Enumerate, Filter, FilterMap, Map};

verus! {

pub uninterp spec fn native_count_supported<I>(iter: I) -> bool;

// Count may skip callbacks (Map and Enumerate delegate to the inner count).
// Keep its returned-value relation separate from successful exhaustion.
pub uninterp spec fn native_count_post<I>(iter: I, result: usize) -> bool;

pub broadcast axiom fn slice_count_supported<'a, T>(iter: core::slice::Iter<'a, T>)
    ensures
        #[trigger] native_count_supported(iter),
;

pub broadcast axiom fn slice_metric_bound<'a, T>(iter: core::slice::Iter<'a, T>)
    ensures
        #[trigger] IteratorSpec::decrease(&iter) is Some,
        IteratorSpec::decrease(&iter).unwrap() <= usize::MAX as nat,
        IteratorSpec::remaining(&iter).len() <= usize::MAX as nat,
;

pub broadcast axiom fn map_count_supported<I: Iterator, F>(iter: Map<I, F>)
    requires
        native_count_supported(map_iter(iter)),
    ensures
        #[trigger] native_count_supported(iter),
;

pub broadcast axiom fn filter_count_supported<I: Iterator, F>(
    iter: Filter<I, F>,
)
// Native Filter::count evaluates its predicate for every inner item,
// including FnMut predicates; unlike Map::count it cannot skip callbacks.

    requires
        native_count_supported(filter_iter(iter)),
    ensures
        #[trigger] native_count_supported(iter),
;

#[verifier::external_body]
#[verifier::external_type_specification]
#[verifier::reject_recursive_types(I)]
pub struct ExEnumerate<I>(Enumerate<I>);

pub uninterp spec fn enumerate_iter<I>(iter: Enumerate<I>) -> I;

pub uninterp spec fn enumerate_index<I>(iter: Enumerate<I>) -> usize;

pub uninterp spec fn enumerate_post<I>(input: I, result: Enumerate<I>) -> bool;

pub broadcast axiom fn enumerate_postcondition<I: Iterator>(input: I, result: Enumerate<I>)
    requires
        native_count_supported(input),
        input.obeys_prophetic_iter_laws(),
        input.decrease().is_some(),
        input.decrease().unwrap() <= usize::MAX as nat,
        #[trigger] enumerate_post(input, result),
    ensures
        enumerate_iter(result) == input,
        enumerate_index(result) == 0,
        IteratorSpec::obeys_prophetic_iter_laws(&result),
;

impl<I: Iterator> super::iter::IteratorSpecImpl for Enumerate<I> {
    open spec fn obeys_prophetic_iter_laws(&self) -> bool {
        enumerate_iter(*self).obeys_prophetic_iter_laws() && native_count_supported(
            enumerate_iter(*self),
        ) && enumerate_iter(*self).decrease().is_some() && enumerate_index(*self) as int
            + enumerate_iter(*self).decrease().unwrap() <= usize::MAX as nat
    }

    #[verifier::prophetic]
    open spec fn remaining(&self) -> Seq<Self::Item> {
        let inner = enumerate_iter(*self).remaining();
        Seq::new(inner.len(), |i: int| ((enumerate_index(*self) as int + i) as usize, inner[i]))
    }

    #[verifier::prophetic]
    open spec fn will_return_none(&self) -> bool {
        enumerate_iter(*self).will_return_none()
    }

    open spec fn decrease(&self) -> Option<nat> {
        enumerate_iter(*self).decrease()
    }

    open spec fn peek(&self, index: int) -> Option<Self::Item> {
        match enumerate_iter(*self).peek(index) {
            Some(value) => Some(((enumerate_index(*self) as int + index) as usize, value)),
            None => None,
        }
    }
}

pub broadcast axiom fn enumerate_count_supported<I: Iterator>(iter: Enumerate<I>)
    requires
        native_count_supported(enumerate_iter(iter)),
    ensures
        #[trigger] native_count_supported(iter),
;

#[verifier::external_body]
#[verifier::external_type_specification]
#[verifier::reject_recursive_types(I)]
#[verifier::reject_recursive_types(F)]
pub struct ExFilterMap<I, F>(FilterMap<I, F>);

pub uninterp spec fn filter_map_iter<I, F>(iter: FilterMap<I, F>) -> I;

pub uninterp spec fn filter_map_fun<I, F>(iter: FilterMap<I, F>) -> F;

pub uninterp spec fn filter_map_post<I, B, F>(input: I, f: F, result: FilterMap<I, F>) -> bool;

#[verifier::prophetic]
pub uninterp spec fn filter_map_outputs<I: Iterator, B, F: FnMut(I::Item) -> Option<B>>(
    iter: FilterMap<I, F>,
) -> Seq<Option<B>>;

pub open spec fn flatten_options<T>(values: Seq<Option<T>>) -> Seq<T> {
    values.filter(|v: Option<T>| v.is_some()).map_values(|v: Option<T>| v.unwrap())
}

/// Filtering absent slots preserves precisely the present values.
pub broadcast proof fn lemma_flatten_options_contains<T>(values: Seq<Option<T>>, value: T)
    ensures
        #[trigger] flatten_options(values).contains(value) == values.contains(Some(value)),
{
    broadcast use {crate::seq::group_seq_axioms, crate::seq_lib::group_filter_ensures};
    let present = values.filter(|v: Option<T>| v.is_some());
    if flatten_options(values).contains(value) {
        let j = choose|j: int| 0 <= j < flatten_options(values).len()
            && flatten_options(values)[j] == value;
        values.lemma_filter_contains_rev(|v: Option<T>| v.is_some(), present[j]);
        assert(present[j] == Some(value));
    }
    if values.contains(Some(value)) {
        let i = choose|i: int| 0 <= i < values.len() && values[i] == Some(value);
        values.lemma_filter_contains(|v: Option<T>| v.is_some(), i);
        let j = choose|j: int| 0 <= j < present.len() && present[j] == Some(value);
        assert(flatten_options(values)[j] == value);
    }
}

/// Native count's implementation-specific behavior. Filter/FilterMap evaluate
/// callbacks during counting; Map/Enumerate delegate and need not exhaust their
/// own next stream. Custom Iterator overrides are excluded by the closed marker.
pub broadcast axiom fn slice_count_post<'a, T>(iter: core::slice::Iter<'a, T>, count: usize)
    requires
        #[trigger] native_count_post(iter, count),
    ensures
        count == iter.remaining().len(),
        iter.will_return_none(),
;

pub broadcast axiom fn map_count_post<I: Iterator, F>(iter: Map<I, F>, count: usize)
    requires
        #[trigger] native_count_post(iter, count),
    ensures
        native_count_post(map_iter(iter), count),
;

pub broadcast axiom fn enumerate_count_post<I: Iterator>(iter: Enumerate<I>, count: usize)
    requires
        #[trigger] native_count_post(iter, count),
    ensures
        native_count_post(enumerate_iter(iter), count),
;

pub broadcast axiom fn filter_count_post<I: Iterator, F: FnMut(&I::Item) -> bool>(
    iter: Filter<I, F>,
    count: usize,
)
    requires
        #[trigger] native_count_post(iter, count),
    ensures
        count == iter.remaining().len(),
        iter.will_return_none(),
;

pub broadcast axiom fn filter_map_count_post<I: Iterator, F>(
    iter: FilterMap<I, F>,
    count: usize,
) where FilterMap<I, F>: Iterator
    requires
        #[trigger] native_count_post(iter, count),
    ensures
        count == iter.remaining().len(),
        iter.will_return_none(),
;

/// A supplied finite observation. A lazy callback may fail to finish; every
/// available output is a prefix, and successful exhaustion establishes equality.
#[verifier::prophetic]
pub open spec fn finite_native_view<I: Iterator>(iter: I, values: Seq<I::Item>) -> bool {
    &&& native_count_supported(iter)
    &&& iter.obeys_prophetic_iter_laws()
    &&& iter.decrease().is_some()
    &&& iter.decrease().unwrap() <= usize::MAX as nat
    &&& values.len() <= usize::MAX as nat
    &&& forall|count: usize| #[trigger] native_count_post(iter, count) ==> count == values.len()
    &&& iter.remaining().is_prefix_of(values)
    &&& (iter.will_return_none() ==> iter.remaining() == values)
}

pub proof fn flatten_options_prefix<T>(prefix: Seq<Option<T>>, full: Seq<Option<T>>)
    requires
        prefix.is_prefix_of(full),
    ensures
        flatten_options(prefix).is_prefix_of(flatten_options(full)),
{
    broadcast use crate::seq::group_seq_lemmas;

    let rest = full.subrange(prefix.len() as int, full.len() as int);
    assert(full =~= prefix + rest);
    let p = |v: Option<T>| v.is_some();
    Seq::filter_distributes_over_add(prefix, rest, p);
    assert(flatten_options(full) =~= flatten_options(prefix) + flatten_options(rest));
}

pub proof fn filter_index_by_value<T>(values: Seq<T>, pred: spec_fn(T) -> bool)
    ensures
        values.filter_index(|i: int| pred(values[i])) == values.filter(pred),
    decreases values.len(),
{
    broadcast use crate::seq::group_seq_lemmas;

    reveal(Seq::filter_index);
    reveal(Seq::filter);
    if values.len() > 0 {
        let rest = values.drop_last();
        filter_index_by_value(rest, pred);
        rest.filter_index_ext(|i: int| pred(values[i]), |i: int| pred(rest[i]));
    }
}

pub broadcast proof fn filter_decisions<T>(
    values: Seq<T>,
    keep: spec_fn(int) -> bool,
    pred: spec_fn(T) -> bool,
)
    requires
        forall|i: int| 0 <= i < values.len() ==> keep(i) == pred(#[trigger] values[i]),
    ensures
        #[trigger] values.filter_index(keep) == #[trigger] values.filter(pred),
{
    values.filter_index_ext(keep, |i: int| pred(values[i]));
    filter_index_by_value(values, pred);
}

pub broadcast proof fn filter_empty_iff<T>(values: Seq<T>, pred: spec_fn(T) -> bool)
    ensures
        (#[trigger] values.filter(pred).len() == 0) == (forall|i: int|
            0 <= i < values.len() ==> !pred(values[i])),
{
    broadcast use crate::seq_lib::group_filter_ensures;

    if forall|i: int| 0 <= i < values.len() ==> !pred(values[i]) {
        values.lemma_all_neg_filter_empty(pred);
    } else {
        let i = choose|i: int| 0 <= i < values.len() && pred(values[i]);
        assert(values.filter(pred).contains(values[i]));
    }
}

pub broadcast axiom fn filter_map_postcondition<I: Iterator, B, F: FnMut(I::Item) -> Option<B>>(
    input: I,
    f: F,
    result: FilterMap<I, F>,
)
    requires
        native_count_supported(input),
        input.obeys_prophetic_iter_laws(),
        input.decrease().is_some(),
        forall|i: int|
            0 <= i < input.remaining().len() ==> f.requires((#[trigger] input.remaining()[i],)),
        #[trigger] filter_map_post::<I, B, F>(input, f, result),
    ensures
        filter_map_iter(result) == input,
        filter_map_fun(result) == f,
        IteratorSpec::obeys_prophetic_iter_laws(&result),
        filter_map_outputs(result).len() <= input.remaining().len(),
        forall|i: int|
            0 <= i < filter_map_outputs(result).len() ==> f.ensures(
                (input.remaining()[i],),
                #[trigger] filter_map_outputs(result)[i],
            ),
        result.remaining() == flatten_options(filter_map_outputs(result)),
        result.remaining().len() <= input.remaining().len(),
        forall|k: int|
            0 <= k < result.remaining().len() ==> exists|i: int|
                0 <= i < input.remaining().len() && f.ensures(
                    (input.remaining()[i],),
                    Some(#[trigger] result.remaining()[k]),
                ),
        result.will_return_none() ==> input.will_return_none() && filter_map_outputs(result).len()
            == input.remaining().len(),
        result.decrease() == input.decrease(),
;

impl<B, I: Iterator, F: FnMut(I::Item) -> Option<B>> super::iter::IteratorSpecImpl for FilterMap<
    I,
    F,
> {
    open spec fn obeys_prophetic_iter_laws(&self) -> bool {
        filter_map_iter(*self).obeys_prophetic_iter_laws() && native_count_supported(
            filter_map_iter(*self),
        )
    }

    #[verifier::prophetic]
    uninterp spec fn remaining(&self) -> Seq<B>;

    #[verifier::prophetic]
    uninterp spec fn will_return_none(&self) -> bool;

    uninterp spec fn decrease(&self) -> Option<nat>;

    open spec fn peek(&self, _index: int) -> Option<B> {
        None
    }
}

pub broadcast axiom fn filter_map_count_supported<I: Iterator, F>(iter: FilterMap<I, F>)
    requires
        native_count_supported(filter_map_iter(iter)),
    ensures
        #[trigger] native_count_supported(iter),
;

/// Every flattened element comes from a present original slot.
pub broadcast proof fn flatten_options_witness<T>(values: Seq<Option<T>>, index: int)
    requires
        0 <= index < flatten_options(values).len(),
    ensures
        ({
            let value = #[trigger] flatten_options(values)[index];
            exists|i: int| 0 <= i < values.len() && #[trigger] values[i] == Some(value)
        }),
{
    broadcast use {crate::seq::group_seq_lemmas, crate::seq_lib::group_filter_ensures};

    let present = values.filter(|v: Option<T>| v.is_some());
    assert(present[index].is_some());
    assert(present.contains(present[index]));
    values.lemma_filter_contains_rev(|v: Option<T>| v.is_some(), present[index]);
    assert(values.contains(present[index]));
    let i = choose|i: int| 0 <= i < values.len() && values[i] == present[index];
    assert(values[i] == Some(flatten_options(values)[index]));
}

/// Checked induction step used by generated total native fold invariants.
pub broadcast proof fn fold_prefix_step<A, B>(
    values: Seq<A>,
    initial: B,
    f: spec_fn(B, A) -> B,
    index: int,
)
    requires
        0 <= index < values.len(),
    ensures
        values.take(index + 1).fold_left(initial, f) == f(
            #[trigger] values.take(index).fold_left(initial, f),
            values[index],
        ),
{
    broadcast use crate::seq::group_seq_lemmas;

    let prefix = values.take(index);
    assert(values.take(index + 1) =~= prefix.push(values[index]));
    assert(prefix.push(values[index]).drop_last() =~= prefix);
    assert(prefix.push(values[index]).last() == values[index]);
    reveal(Seq::fold_left);
}

pub broadcast group group_native_adapter_axioms {
    fold_prefix_step,
    flatten_options_witness,
    slice_count_post,
    map_count_post,
    enumerate_count_post,
    filter_count_post,
    filter_map_count_post,
    slice_count_supported,
    slice_metric_bound,
    map_count_supported,
    filter_count_supported,
    enumerate_postcondition,
    enumerate_count_supported,
    filter_map_postcondition,
    filter_map_count_supported,
    filter_decisions,
    filter_empty_iff,
}

} // verus!
