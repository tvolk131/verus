#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const SPARSE_COUNT: &str = r#"
use vstd::prelude::*;
verus! {
pub proof fn occupied<A>(slots: Seq<Option<A>>)
    ensures slots.count_prefix(|slot: Option<A>| slot.is_some(), slots.len())
        == vstd::seq_update::sparse_entries(slots, |i: int| i).len(),
{}
pub open spec fn rounds(source: Seq<Option<(int, int)>>) -> Seq<Option<int>> {
    source.map_values(|slot: Option<(int, int)>| match slot {
        None => None, Some(pair) => Some(pair.1),
    })
}
pub open spec fn entries(slots: Seq<Option<int>>) -> Seq<(int, int)> {
    vstd::seq_update::sparse_entries(slots, |i: int| (i, slots[i].unwrap()))
}
pub proof fn projected(source: Seq<Option<(int, int)>>, target: int)
    ensures source.count_prefix(|slot: Option<(int, int)>|
            slot.is_some() && slot.unwrap().1 == target, source.len())
        == entries(rounds(source)).map_values(|entry: (int, int)| entry.1)
            .filter(|value: int| value == target).len(),
{}
pub proof fn positional_projection(source: Seq<Option<(int, int)>>, target: int)
    ensures source.count_prefix(|slot: Option<(int, int)>|
            slot.is_some() && slot.unwrap().1 == target, source.len())
        == rounds(source).count_prefix(|slot: Option<int>|
            slot.is_some() && slot.unwrap() == target, rounds(source).len()),
{}
}
"#;

#[test]
fn sparse_projected_filtered_counts_use_checked_demanded_lemmas() {
    let plain = verify_one_file("sparse_count_plain", SPARSE_COUNT.to_owned(), &["vstd"]);
    let errors = plain.expect_err("the inductive connection needs checked library assistance");
    assert!(errors.errors.iter().any(|d| d.message.contains("postcondition")), "{:?}", errors);
    let assisted = verify_one_file(
        "sparse_count_assisted",
        SPARSE_COUNT.to_owned(),
        &["vstd", "--infer-goals", "--no-cheating"],
    );
    assert!(assisted.is_ok(), "{:?}", assisted);
}

#[test]
fn false_sparse_counts_and_unrelated_projection_cannot_borrow_a_bridge() {
    for (name, source) in [
        ("sparse_count_wrong", SPARSE_COUNT.replace("|i: int| i).len()", "|i: int| i).len() + 1")),
        ("sparse_count_wrong_projection", SPARSE_COUNT.replace("Some(pair.1)", "Some(pair.0)")),
        (
            "sparse_count_wrong_size",
            SPARSE_COUNT.replace("entries(rounds(source))", "entries(rounds(source).take(0))"),
        ),
    ] {
        let result = verify_one_file(name, source, &["vstd", "--infer-goals", "--no-cheating"]);
        let errors = result.expect_err("a false positional/iterator equation must fail");
        assert!(errors.errors.iter().any(|d| d.message.contains("postcondition")), "{:?}", errors);
    }
}

const OCCUPIED: &str = r#"
use vstd::prelude::*;
verus! {
#[verifier::infer(sound)] fn replace<T: Copy>(v: &mut Vec<T>, i: usize, value: T) { v[i] = value; }
pub open spec fn used(s: Seq<bool>) -> nat { s.count_prefix(|b: bool| b, s.len()) }
fn occupy(v: &mut Vec<bool>, total: &mut usize, i: usize)
    requires i < old(v)@.len(), !old(v)@[i as int], *old(total) == used(old(v)@),
    ensures *final(total) == used(final(v)@),
        final(v)@ == old(v)@.update(i as int, true),
{
    let n = v.len();
    replace(v, i, true);
    *total += 1;
}
}
"#;

#[test]
fn occupied_slot_preservation_requires_and_uses_checked_assistance() {
    let plain = verify_one_file("occupied_plain", OCCUPIED.to_owned(), &["vstd"]);
    let errors = plain.unwrap_err();
    assert!(
        errors
            .errors
            .iter()
            .any(|d| d.message.contains("arithmetic") || d.message.contains("postcondition")),
        "baseline must fail a proof: {:?}",
        errors
    );
    let assisted =
        verify_one_file("occupied_assisted", OCCUPIED.to_owned(), &["vstd", "--infer-goals"]);
    assert!(assisted.is_ok(), "{:?}", assisted);
}

fn rejected(name: &str, code: String) {
    let errors = verify_one_file(name, code, &["vstd", "--infer-goals"]).unwrap_err();
    assert!(
        errors.errors.iter().any(|d| d.message.contains("arithmetic")
            || d.message.contains("postcondition")
            || d.message.contains("assertion failed")),
        "must be a semantic proof rejection: {:?}",
        errors
    );
}

#[test]
fn wrong_counter_is_rejected() {
    rejected("wrong_counter", OCCUPIED.replace("*total += 1", "*total += 2"));
}

#[test]
fn wrong_slot_is_rejected() {
    rejected("wrong_slot", OCCUPIED.replace("replace(v, i, true)", "replace(v, 0, true)"));
}

#[test]
fn wrong_value_is_rejected() {
    rejected("wrong_value", OCCUPIED.replace("replace(v, i, true)", "replace(v, i, false)"));
}

#[test]
fn missing_accounting_invariant_is_rejected() {
    rejected("missing_accounting", OCCUPIED.replace(", *old(total) == used(old(v)@)", ""));
}

#[test]
fn duplicate_occupancy_is_not_counted_as_a_new_slot() {
    rejected("duplicate", OCCUPIED.replace("!old(v)@[i as int], ", ""));
}

#[test]
fn false_goal_is_not_assumed() {
    rejected("false_goal", OCCUPIED.replace("    *total += 1;", "    *total += 1; assert(false);"));
}

test_verify_one_file_with_options! {
    #[test] updates_outside_a_counted_prefix_leave_its_count_unchanged ["vstd", "--infer-goals"] => verus_code! {
        use vstd::prelude::*;
        proof fn outside<T>(s: Seq<T>, p: spec_fn(T) -> bool, i: int, value: T, n: nat)
            requires n <= i < s.len(),
            ensures s.update(i, value).count_prefix(p, n) == s.count_prefix(p, n),
        { }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] predicate_parameters_and_full_prefix_counts ["vstd", "--infer-goals"] => verus_code! {
        use vstd::prelude::*;
        proof fn capacity(s: Seq<u8>, limit: u8, i: int)
            requires 0 <= i < s.len(), s[i] <= limit,
            ensures s.count_prefix(|x: u8| x > limit, s.len()) < s.len(),
        { }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] all_empty_slots_have_zero_count ["vstd", "--infer-goals"] => verus_code! {
        use vstd::prelude::*;
        proof fn empty(s: Seq<bool>)
            requires forall|i: int| 0 <= i < s.len() ==> !#[trigger] s[i],
            ensures s.count_prefix(|b: bool| b, s.len()) == 0,
        { }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] explicit_count_assertions_also_select_the_strategy ["vstd", "--infer-goals"] => verus_code! {
        use vstd::prelude::*;
        proof fn bound<T>(s: Seq<T>, p: spec_fn(T) -> bool, n: nat)
            requires n <= s.len(),
        { assert(s.count_prefix(p, n) <= n); }
    } => Ok(())
}
