use vstd::prelude::*;
verus! {
// Value interpretation of the enumerated observation equations only.
// These are not native reference values, native iterators, or executable code.
pub ghost struct RefObservation {
    pub identity: int,
    pub current: u8,
    pub eventual: u8,
}
pub ghost struct IterObservation {
    pub refs: Seq<RefObservation>,
    pub resolved: bool,
}
pub ghost struct ZipObservation {
    pub first: IterObservation,
    pub second: Seq<u8>,
    pub resolved: bool,
}

pub open spec fn pad_xor(before: Seq<u8>, key: Seq<u8>, after: Seq<u8>) -> bool {
    &&& before.len() == 128
    &&& after.len() == 128
    &&& forall|i: int| 0 <= i < 128 ==> #[trigger] after[i] ==
        if i < key.len() { before[i] ^ key[i] } else { before[i] }
}

pub open spec fn observed_refs(before: Seq<u8>, after: Seq<u8>) -> Seq<RefObservation> {
    Seq::new(128, |i: int| RefObservation {
        identity: i, current: before[i], eventual: after[i],
    })
}
pub open spec fn constructor_equations(
    before: Seq<u8>, after: Seq<u8>, refs: Seq<RefObservation>,
) -> bool {
    &&& before.len() == 128 && after.len() == 128 && refs.len() == 128
    &&& forall|i: int| 0 <= i < 128 ==> #[trigger] refs[i].identity == i
        && refs[i].current == before[i] && refs[i].eventual == after[i]
}
pub open spec fn zip_remaining(z: ZipObservation) -> Seq<(RefObservation, u8)> {
    z.first.refs.zip_truncate(z.second)
}
pub open spec fn successful_next_equations(a: ZipObservation, b: ZipObservation) -> bool {
    &&& zip_remaining(a).len() > 0
    &&& zip_remaining(b) =~= zip_remaining(a).drop_first()
    &&& zip_remaining(b).len() + 1 == zip_remaining(a).len()
    // All states have obeys_prophetic_iter_laws=true, will_return_none=true,
    // decrease=Some(remaining.len()), peek=None, and exact size hint.
}
pub open spec fn none_next_equations(a: ZipObservation, b: ZipObservation) -> bool {
    &&& zip_remaining(a).len() == 0
    &&& zip_remaining(b) =~= zip_remaining(a)
}
pub open spec fn prefix_equations(before: Seq<u8>, key: Seq<u8>, after: Seq<u8>) -> bool {
    forall|i: int| 0 <= i < 128 && i < key.len() ==>
        #[trigger] after[i] == before[i] ^ key[i]
}
// Explicit interpretations of the two proposed missing laws. These definitions
// are NOT asserted for native types, nor imported as axioms anywhere.
pub open spec fn proposed_resolution_equations(z: ZipObservation) -> bool {
    &&& z.resolved ==> z.first.resolved
    &&& z.first.resolved ==> forall|i: int| 0 <= i < z.first.refs.len() ==>
        #[trigger] z.first.refs[i].eventual == z.first.refs[i].current
}

pub open spec fn bad_future(before: Seq<u8>, key: Seq<u8>) -> Seq<u8> {
    Seq::new(128, |i: int|
        if i < key.len() { before[i] ^ key[i] }
        else if i == key.len() { before[i] ^ 1u8 }
        else { before[i] })
}
pub open spec fn running_state(refs: Seq<RefObservation>, key: Seq<u8>, n: int) -> ZipObservation {
    ZipObservation {
        first: IterObservation { refs: refs.subrange(n, 128), resolved: false },
        second: key.subrange(n, key.len() as int), resolved: false,
    }
}
pub open spec fn terminal_state() -> ZipObservation {
    ZipObservation {
        first: IterObservation { refs: Seq::empty(), resolved: true },
        second: Seq::empty(), resolved: true,
    }
}

pub proof fn toggle_changes(x: u8)
    ensures x ^ 1u8 != x,
{
    assert(x ^ 1u8 != x) by (bit_vector);
}

pub proof fn checked_constructor_and_prefix(before: Seq<u8>, key: Seq<u8>)
    requires before.len() == 128, key.len() < 128,
    ensures
        constructor_equations(before, bad_future(before, key), observed_refs(before, bad_future(before, key))),
        prefix_equations(before, key, bad_future(before, key)),
        !pad_xor(before, key, bad_future(before, key)),
{
    let after = bad_future(before, key);
    let refs = observed_refs(before, after);
    assert forall|i: int| 0 <= i < 128 implies #[trigger] refs[i].identity == i
        && refs[i].current == before[i] && refs[i].eventual == after[i] by {}
    assert forall|i: int| 0 <= i < 128 && i < key.len() implies
        #[trigger] after[i] == before[i] ^ key[i] by {}
    toggle_changes(before[key.len() as int]);
    assert(after[key.len() as int] != before[key.len() as int]);
}

// An arbitrary number of successful generic Zip steps, including arbitrary
// initial bytes and masks. Indices identify the exact initial/eventual referents.
pub proof fn checked_successful_steps(before: Seq<u8>, key: Seq<u8>, n: int)
    requires before.len() == 128, key.len() < 128, 0 <= n < key.len(),
    ensures
        successful_next_equations(
            running_state(observed_refs(before, bad_future(before, key)), key, n),
            running_state(observed_refs(before, bad_future(before, key)), key, n + 1)),
        zip_remaining(running_state(observed_refs(before, bad_future(before, key)), key, n))[0].0.identity == n,
        zip_remaining(running_state(observed_refs(before, bad_future(before, key)), key, n))[0].0.eventual
            == before[n] ^ key[n],
        proposed_resolution_equations(running_state(observed_refs(before, bad_future(before, key)), key, n)),
{
    let refs = observed_refs(before, bad_future(before, key));
    let a = running_state(refs, key, n);
    let b = running_state(refs, key, n + 1);
    assert forall|i: int| 0 <= i < zip_remaining(b).len() implies
        #[trigger] zip_remaining(b)[i] == zip_remaining(a).drop_first()[i] by {
        assert(refs.subrange(n + 1, 128)[i] == refs[n + 1 + i]);
        assert(refs.subrange(n, 128)[i + 1] == refs[n + 1 + i]);
        assert(key.subrange(n + 1, key.len() as int)[i] == key[n + 1 + i]);
        assert(key.subrange(n, key.len() as int)[i + 1] == key[n + 1 + i]);
    }
}

// The generic native next contract permits the hidden first iterator's
// *observation* to change to empty on terminal None. The real specialized Rust
// implementation does not do this. Precisely that correspondence is missing.
pub proof fn checked_terminal_step(before: Seq<u8>, key: Seq<u8>)
    requires before.len() == 128, key.len() < 128,
    ensures
        none_next_equations(
            running_state(observed_refs(before, bad_future(before, key)), key, key.len() as int),
            terminal_state()),
        terminal_state().resolved,
        proposed_resolution_equations(terminal_state()),
        !pad_xor(before, key, bad_future(before, key)),
{
    checked_constructor_and_prefix(before, key);
    assert(zip_remaining(terminal_state()).len() == 0);
}

// The ordinary for-loop snapshot/history equations are satisfied too: each
// history entry is the corresponding initial pair and the final index is the
// original zip length. No law observes the unreturned suffix through history.
pub proof fn checked_history(before: Seq<u8>, key: Seq<u8>, n: int)
    requires before.len() == 128, key.len() < 128, 0 <= n <= key.len(),
    ensures
        zip_remaining(running_state(observed_refs(before, bad_future(before, key)), key, 0)).len() == key.len(),
        forall|i: int| 0 <= i < n ==>
            #[trigger] zip_remaining(running_state(observed_refs(before, bad_future(before, key)), key, 0))
                .subrange(0, n)[i].0.eventual == before[i] ^ key[i],
{
}
} // verus!
