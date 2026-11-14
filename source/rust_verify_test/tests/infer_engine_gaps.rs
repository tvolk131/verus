#![feature(rustc_private)]
//! Gaps found by annotating real code: rejections that hid every other
//! result, shapes that were rejected for no semantic reason, and rejection
//! messages that did not say what to change.

#[macro_use]
mod common;
use common::*;

// A plain `for` loop is desugared in place, with no attribute macro on an
// expression, so a crate without `#![feature(proc_macro_hygiene)]` reaches
// inference and gets ordinary rejections instead of E0658. (This harness always
// enables the feature, so the test pins the rejection, not the feature gate.)
test_verify_one_file_with_options! {
    #[test] for_loop_is_an_infer_rejection_not_a_compile_error ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_infer]
fn sum(a: u32) -> u32 { let mut s = 0u32; for i in 0..a { s = s.wrapping_add(i); } s }
"#).to_string() => Err(e) => {
        assert!(!e.errors.is_empty());
        assert!(e.errors.iter().all(|d| d.message.contains("Verus Infer unsupported")), "{:?}", e.errors);
    }
}

// A `for` loop in hand-verified attribute-style code still verifies.
test_verify_one_file_with_options! {
    #[test] plain_for_loop_verifies_without_an_attribute ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_spec(ensures true)]
fn count(n: u32) {
    for _i in 0..n {}
}
"#).to_string() => Ok(())
}

// A method call on a value bound by a `match` arm composes like any call.
test_verify_one_file_with_options! {
    #[test] method_call_on_match_bound_record ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_verify] #[derive(Clone, Copy)]
pub struct Region { pages: u32, page_size: u32 }
impl Region {
    #[verus_infer]
    fn len(&self) -> u64 { self.pages as u64 + self.page_size as u64 }
}
#[verus_verify]
pub struct Layout { full: Region, count: u32, trailing: Option<Region> }
impl Layout {
    #[verus_infer]
    fn len(&self) -> u64 {
        let last = match self.trailing { Some(region) => region.len(), None => 0 };
        self.full.len() + self.count as u64 + last
    }
}
verus! {
fn client() {
    let region = Region { pages: 3, page_size: 4 };
    let layout = Layout { full: region, count: 2, trailing: Some(region) };
    let n = layout.len();
    assert(n == 16);
}
}
"#).to_string() => Ok(())
}

const GETTERS: &str = r#"
use vstd::prelude::*;
pub mod stats {
    use vstd::prelude::*;
    #[verus_verify]
    pub struct CacheStats { evictions: u64, used: u64 }
    impl CacheStats {
        #[verus_infer]
        pub fn new(evictions: u64) -> CacheStats { CacheStats { evictions, used: 0 } }
        #[verus_infer]
        pub fn evictions(&self) -> u64 { self.evictions }
        #[verus_infer]
        pub fn total(&self) -> u64 { self.evictions + self.used }
    }
    verus! {
    // Inside the module the views are transparent.
    fn inside() {
        let stats = CacheStats::new(7);
        let n = stats.evictions();
        assert(n == 7);
    }
    }
}
"#;

// A `pub` function over private fields cannot state its contract in public
// terms; it is installed through its (module-private) named views.
test_verify_one_file_with_options! {
    #[test] public_getters_over_private_fields ["vstd", "--no-cheating"]
        => GETTERS.to_string() => Ok(())
}

// Outside the module the contract is abstract: callable when the precondition
// is trivially true, and nothing leaks about private fields.
test_verify_one_file_with_options! {
    #[test] private_state_stays_abstract_outside_the_module ["vstd", "--no-cheating"]
        => GETTERS.to_string() + r#"
verus! {
fn outside(stats: &stats::CacheStats) {
    let n = stats.evictions();
    assert(n == 7);
}
}
"# => Err(e) => {
        assert_eq!(e.errors.len(), 1, "{:?}", e.errors);
        assert!(e.errors[0].message.contains("assertion failed"), "{:?}", e.errors);
    }
}

// Little-endian decoding of the 16-, 32- and 64-bit words, from a subslice.
test_verify_one_file_with_options! {
    #[test] little_endian_words_of_each_width ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_infer]
fn word16(data: &[u8]) -> u16 { u16::from_le_bytes(data[1..3].try_into().unwrap()) }
#[verus_infer]
fn word32(data: &[u8]) -> u32 { u32::from_le_bytes(data[0..4].try_into().unwrap()) }
#[verus_infer]
fn word64(data: &[u8]) -> u64 { u64::from_le_bytes(data[3..11].try_into().unwrap()) }
#[verus_infer]
fn bytes32(x: u32) -> [u8; 4] { x.to_le_bytes() }
verus! {
fn client(data: &[u8])
    requires data.len() == 4, data[0] == 1, data[1] == 2, data[2] == 0, data[3] == 0,
{
    let x = word32(data);
    assert(x == 513) by (bit_vector)
        requires x == (1u32 | 2u32 << 8 | 0u32 << 16 | 0u32 << 24);
}
// The precondition is the exact bound.
fn short(data: &[u8])
    requires data.len() == 3,
{
    let _ = word16(data);
}
}
"#).to_string() => Ok(())
}

test_verify_one_file_with_options! {
    #[test] little_endian_word_needs_enough_bytes ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_infer]
fn word32(data: &[u8]) -> u32 { u32::from_le_bytes(data[0..4].try_into().unwrap()) }
verus! {
fn short(data: &[u8])
    requires data.len() == 3,
{
    let _ = word32(data);
}
}
"#).to_string() => Err(e) => {
        assert_eq!(e.errors.len(), 1, "{:?}", e.errors);
        assert!(e.errors[0].message.contains("precondition not satisfied"), "{:?}", e.errors);
    }
}

fn has_help(e: &TestErr, message: &str, help: &str) {
    assert!(
        e.errors.iter().any(|d| d.message.contains(message) && d.rendered.contains(help)),
        "{:?}",
        e.errors
    );
}

// A rejected call names the callee as Rust does and says what to change.
test_verify_one_file_with_options! {
    #[test] rejected_call_names_the_callee_and_the_fix ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
verus! {
pub struct Counter { pub n: u32 }
impl Counter {
    pub fn bump(&self) -> (r: u32) ensures r >= self.n { self.n }
}
}
#[verus_infer]
fn caller(c: &Counter) -> u32 { c.bump() }
"#).to_string() => Err(e) => {
        has_help(&e, "Counter::bump`", "ensures ret == <expression>");
    }
}

// A constructor of a struct with private fields: its postcondition is a view
// call, which the candidate note still prints.
test_verify_one_file_with_options! {
    #[test] constructor_over_private_fields_verifies ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
mod m {
    use vstd::prelude::*;
    #[verus_verify]
    pub struct S { x: u64 }
    impl S {
        #[verus_infer]
        pub fn new(x: u64) -> S { S { x } }
    }
}
fn main() {}
"#).to_string() => Ok(())
}

// A rejected std call cannot take `#[verus_infer]`, so the help does not say to add it.
test_verify_one_file_with_options! {
    #[test] rejected_std_call_gets_std_advice ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_infer]
fn f(a: Option<u32>) -> u32 { a.unwrap_or_default() }
"#).to_string() => Err(e) => {
        has_help(&e, "unwrap_or_default`", "models only some standard-library functions");
        assert!(e.errors.iter().all(|d| !d.rendered.contains("add `#[verus_infer]` to it")), "{:?}", e.errors);
    }
}

// A record rejection names the field and the reason.
test_verify_one_file_with_options! {
    #[test] record_rejection_names_the_field ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_verify]
pub struct Holder { rows: Vec<Vec<u8>>, n: u32 }
impl Holder {
    #[verus_infer]
    fn n(&self) -> u32 { self.n }
}
"#).to_string() => Err(e) => {
        has_help(&e, "field `rows`", "write its contract by hand");
    }
}

// Both kinds of contract on one function: the help says to keep one.
test_verify_one_file_with_options! {
    #[test] mixed_contract_rejection_says_keep_one ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
#[verus_infer]
#[verus_spec(requires a < 10)]
fn f(a: u32) -> u32 { a + 1 }
"#).to_string() => Err(e) => {
        has_help(&e, "combining inference with handwritten contracts", "keep one of the two");
    }
}

// Verus's own "ignored" errors point at the attribute that declares the item.
test_verify_one_file_with_options! {
    #[test] undeclared_type_help_names_the_attribute ["vstd", "--no-cheating"] => (r#"
use vstd::prelude::*;
pub struct Plain { a: u32 }
#[verus_verify]
pub struct Holder { p: Plain, n: u32 }
"#).to_string() => Err(e) => {
        has_help(&e, "cannot use type", "add `#[verus_verify]` to its declaration");
    }
}
