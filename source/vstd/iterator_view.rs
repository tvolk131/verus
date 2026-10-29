//! Checked proof combinators for finite native iterator factories.
//! These bodies use existing native library specifications; they add no axioms.
//! The chosen collection/output view is supplied by the interface author.
use crate::prelude::*;
#[cfg(verus_keep_ghost)]
use crate::std_specs::iter::{IteratorSpec, map_fun, map_iter, map_post, map_postcondition};
#[cfg(verus_keep_ghost)]
use crate::std_specs::iter_adapters::*;
verus! {

proof fn native_map_view<I: Iterator, B, F: FnMut(I::Item) -> B>(
    result: core::iter::Map<I, F>,
    input: Seq<I::Item>,
    output: Seq<B>,
)
    requires
        finite_native_view(map_iter(result), input),
        map_post(map_iter(result), map_fun(result), result),
        output.len() == input.len(),
        forall|i: int, value: B|
            0 <= i < input.len() && #[trigger] map_fun(result).ensures((input[i],), value) ==> value
                == output[i],
    ensures
        finite_native_view(result, output),
{
    broadcast use {
        crate::seq::group_seq_axioms,
        crate::seq::group_seq_lemmas,
        crate::std_specs::iter::group_iter_axioms,
        group_native_adapter_axioms,
    };

    let inner = map_iter(result);
    map_postcondition(inner, map_fun(result), result);
    assert forall|i: int| 0 <= i < result.remaining().len() implies #[trigger] result.remaining()[i]
        == output[i] by {
        assert(inner.remaining()[i] == input[i]);
    }
    assert(result.remaining().is_prefix_of(output));
    assert(result.will_return_none() ==> result.remaining() =~= output);
    assert forall|count: usize| #[trigger] native_count_post(result, count) implies count
        == output.len() by {
        assert(native_count_post(inner, count));
    }
}

proof fn native_sparse_view<'a, T: 'a, B, F: FnMut((usize, &'a Option<T>)) -> Option<B>>(
    result: core::iter::FilterMap<core::iter::Enumerate<core::slice::Iter<'a, Option<T>>>, F>,
    source: Seq<Option<T>>,
    label: spec_fn(int) -> B,
)
    requires
        enumerate_index(filter_map_iter(result)) == 0,
        enumerate_iter(filter_map_iter(result)).remaining() == source.as_ref(),
        source.len() <= usize::MAX as nat,
        filter_map_post::<_, B, _>(filter_map_iter(result), filter_map_fun(result), result),
        forall|i: int|
            0 <= i < source.len() ==> filter_map_fun(result).requires(((i as usize, &source[i]),)),
        forall|i: int, value: Option<B>|
            0 <= i < source.len() && #[trigger] filter_map_fun(result).ensures(
                ((i as usize, &source[i]),),
                value,
            ) ==> value == match source[i] {
                None => None,
                Some(_) => Some(label(i)),
            },
    ensures
        finite_native_view(result, crate::seq_update::sparse_entries(source, label)),
{
    broadcast use {
        crate::seq::group_seq_axioms,
        crate::seq::group_seq_lemmas,
        crate::std_specs::iter::group_iter_axioms,
        group_native_adapter_axioms,
    };

    let inner = filter_map_iter(result);
    filter_map_postcondition::<_, B, _>(inner, filter_map_fun(result), result);
    let full = Seq::new(
        source.len(),
        |i: int|
            match source[i] {
                None => None,
                Some(_) => Some(label(i)),
            },
    );
    crate::seq_update::lemma_sparse_entries_eq(source, label, full);
    assert(inner.remaining().len() == source.len());
    assert forall|i: int|
        0 <= i < filter_map_outputs(result).len() implies #[trigger] filter_map_outputs(result)[i]
        == full[i] by {
        assert(inner.remaining()[i] == (i as usize, &source[i]));
    }
    assert(filter_map_outputs(result).is_prefix_of(full));
    flatten_options_prefix(filter_map_outputs(result), full);
    assert(result.will_return_none() ==> filter_map_outputs(result) =~= full);
    assert(native_count_supported(result));
    assert(result.obeys_prophetic_iter_laws());
    assert(result.decrease().is_some());
    assert(result.decrease().unwrap() <= usize::MAX as nat);
    full.lemma_filter_len(|value: Option<B>| value.is_some());
    assert(crate::seq_update::sparse_entries(source, label).len() <= usize::MAX as nat);
    assert(result.remaining().is_prefix_of(crate::seq_update::sparse_entries(source, label)));
    assert(result.will_return_none() ==> result.remaining() == crate::seq_update::sparse_entries(
        source,
        label,
    ));
    assert forall|count: usize| #[trigger] native_count_post(result, count) implies count
        == crate::seq_update::sparse_entries(source, label).len() by {
        assert(result.will_return_none());
    }

}

pub broadcast proof fn lemma_sparse_entry_origin<A, E>(
    source: Seq<Option<A>>,
    label: spec_fn(int) -> E,
    k: int,
)
    requires
        0 <= k < crate::seq_update::sparse_entries(source, label).len(),
    ensures
        ({
            let entry = #[trigger] crate::seq_update::sparse_entries(source, label)[k];
            exists|i: int| 0 <= i < source.len() && source[i].is_some() && entry == label(i)
        }),
{
    broadcast use {crate::seq::group_seq_axioms, crate::seq::group_seq_lemmas};

    let full = Seq::new(
        source.len(),
        |i: int|
            match source[i] {
                None => None,
                Some(_) => Some(label(i)),
            },
    );
    crate::seq_update::lemma_sparse_entries_eq(source, label, full);
    flatten_options_witness(full, k);
}

pub proof fn prove_sparse_factory<'a, T: 'a, B, F: FnMut((usize, &'a Option<T>)) -> Option<B>>(
    input: core::iter::Enumerate<core::slice::Iter<'a, Option<T>>>,
    result: core::iter::FilterMap<core::iter::Enumerate<core::slice::Iter<'a, Option<T>>>, F>,
    output: Seq<B>,
)
    requires
        enumerate_index(input) == 0,
        exists|f: F, source: Seq<Option<T>>, label: spec_fn(int) -> B|
            filter_map_post::<_, B, _>(input, f, result) && output
                == crate::seq_update::sparse_entries(source, label) && enumerate_iter(
                input,
            ).remaining() == source.as_ref() && source.len() <= usize::MAX as nat && (forall|i: int|

                0 <= i < source.len() ==> f.requires(((i as usize, &source[i]),))) && (forall|
                i: int,
                value: Option<B>,
            |
                0 <= i < source.len() && #[trigger] f.ensures(((i as usize, &source[i]),), value)
                    ==> value == match source[i] {
                    None => None,
                    Some(_) => Some(label(i)),
                }),
    ensures
        finite_native_view(result, output),
{
    broadcast use {
        crate::seq::group_seq_axioms,
        crate::seq::group_seq_lemmas,
        crate::std_specs::iter::group_iter_axioms,
        group_native_adapter_axioms,
    };

    let (f, source, label) = choose|f: F, source: Seq<Option<T>>, label: spec_fn(int) -> B|
        filter_map_post::<_, B, _>(input, f, result) && output == crate::seq_update::sparse_entries(
            source,
            label,
        ) && enumerate_iter(input).remaining() == source.as_ref() && source.len()
            <= usize::MAX as nat && (forall|i: int|
            0 <= i < source.len() ==> f.requires(((i as usize, &source[i]),))) && (forall|
            i: int,
            value: Option<B>,
        |
            0 <= i < source.len() && #[trigger] f.ensures(((i as usize, &source[i]),), value)
                ==> value == match source[i] {
                None => None,
                Some(_) => Some(label(i)),
            });
    filter_map_postcondition::<_, B, _>(input, f, result);
    native_sparse_view(result, source, label);
}

/// Connect a supplied output sequence to a native map using the callback's
/// checked relation. The input observation remains an explicit verified interface.
pub proof fn prove_map_factory<I: Iterator, B, F: FnMut(I::Item) -> B>(
    input: I,
    result: core::iter::Map<I, F>,
    output: Seq<B>,
)
    requires
        exists|f: F, full: Seq<I::Item>|
            map_post(input, f, result) && finite_native_view(input, full) && output.len()
                == full.len() && (forall|i: int|
                0 <= i < full.len() ==> f.requires((#[trigger] full[i],))) && (forall|
                i: int,
                value: B,
            |
                0 <= i < full.len() && #[trigger] f.ensures((full[i],), value) ==> value
                    == output[i]),
    ensures
        finite_native_view(result, output),
{
    broadcast use {
        crate::seq::group_seq_axioms,
        crate::seq::group_seq_lemmas,
        crate::std_specs::iter::group_iter_axioms,
        group_native_adapter_axioms,
    };

    let (f, full) = choose|f: F, full: Seq<I::Item>|
        map_post(input, f, result) && finite_native_view(input, full) && output.len() == full.len()
            && (forall|i: int| 0 <= i < full.len() ==> f.requires((#[trigger] full[i],))) && (
        forall|i: int, value: B|
            0 <= i < full.len() && #[trigger] f.ensures((full[i],), value) ==> value == output[i]);
    map_postcondition(input, f, result);
    native_map_view(result, full, output);
}

} // verus!
