#![feature(rustc_private)]
//! Large inferred preconditions of crate-private functions are installed as a
//! call to their named view (`f_requires`), so lemmas and broadcast lemmas about
//! the view match a caller's obligation.

#[macro_use]
mod common;
use common::*;

/// `area`'s inferred precondition is over the 2000-character threshold.
const AREA: &str = r#"
use vstd::prelude::*;
#[verus_infer]
VIS fn area(a: u64, b: u64, c: u64, d: u64) -> u64 {
    let s = a * b + c * d;
    let t = s + a * c + b * d;
    let u = t + a * d + b * c;
    let v = u + a * a + b * b;
    v + c * c + d * d
}
"#;

const BROADCAST: &str = r#"
verus! {
pub(crate) broadcast proof fn area_domain(a: u64, b: u64, c: u64, d: u64)
    requires a <= 1000, b <= 1000, c <= 1000, d <= 1000,
    ensures #[trigger] area_requires(a, b, c, d),
{
    assert(area_requires(a, b, c, d)) by(nonlinear_arith)
        requires a <= 1000, b <= 1000, c <= 1000, d <= 1000;
}
pub(crate) fn client(a: u64, b: u64, c: u64, d: u64) -> u64
    requires a <= 1000, b <= 1000, c <= 1000, d <= 1000,
{
    broadcast use area_domain;
    area(a, b, c, d)
}
}
"#;

const PLAIN_CLIENT: &str = r#"
verus! {
pub(crate) fn client(a: u64, b: u64, c: u64, d: u64) -> u64
    requires a <= 1000, b <= 1000, c <= 1000, d <= 1000,
{
    area(a, b, c, d)
}
}
"#;

fn source(vis: &str, client: &str) -> String {
    AREA.replace("VIS", vis) + client
}

fn precondition_failure(e: TestErr) {
    assert_eq!(e.errors.len(), 1, "{:?}", e.errors);
    assert!(e.errors[0].message.contains("precondition not satisfied"), "{:?}", e.errors);
}

// A broadcast lemma triggered on the view now fires at the call.
test_verify_one_file_with_options! {
    #[test] broadcast_lemma_on_view_discharges_private_call ["vstd", "--no-cheating"]
        => source("pub(crate)", BROADCAST) => Ok(())
}

// Public contracts stay inline for cross-crate importers, so the same broadcast
// lemma does not fire there (callers can still call the lemma explicitly).
test_verify_one_file_with_options! {
    #[test] public_precondition_stays_inline ["vstd", "--no-cheating"]
        => source("pub", &BROADCAST.replace("pub(crate) fn client", "pub fn client")
            .replace("pub(crate) broadcast proof fn", "pub broadcast proof fn"))
        => Err(e) => precondition_failure(e)
}

// Without assistance the large precondition is left to the caller.
test_verify_one_file_with_options! {
    #[test] plain_call_needs_a_proof ["vstd", "--no-cheating"]
        => source("pub(crate)", PLAIN_CLIENT) => Err(e) => precondition_failure(e)
}
