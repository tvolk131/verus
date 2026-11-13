#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};
use common::*;

const PROVIDER: &str = r#"
use vstd::prelude::*;
use core::mem::size_of;
#[verus_verify]
pub struct WordLayout;
#[verus_verify]
impl WordLayout {
    pub const fn bytes() -> usize { size_of::<u64>() }
}
#[verus_verify]
pub const WORD_BYTES: usize = WordLayout::bytes();
#[verus_infer]
pub fn read_word(data: &[u8]) -> u64 {
    u64::from_le_bytes(data[..size_of::<u64>()].try_into().unwrap())
}
#[verus_verify]
#[derive(Copy, Clone)]
pub struct Counter { pub value: u64 }
#[verus_verify]
impl Counter {
    #[verus_infer]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self { value: self.value.checked_add(other.value)? })
    }
    #[verus_infer]
    pub fn within(&self, limit: &Self) -> bool {
        self.value <= limit.value
    }
}
verus! {
pub open spec fn le_word(data: Seq<u8>) -> u64 {
    (data[0] as u64) | ((data[1] as u64) << 8)
    | ((data[2] as u64) << 16) | ((data[3] as u64) << 24)
    | ((data[4] as u64) << 32) | ((data[5] as u64) << 40)
    | ((data[6] as u64) << 48) | ((data[7] as u64) << 56)
}
}
"#;

const GOALS: &str = r#"
verus! {
pub fn decode_add_and_bound(data: &[u8], extra: Counter, limit: &Counter)
    -> (result: Option<Counter>)
    requires data.len() >= WORD_BYTES,
    ensures match result {
        Some(value) => value.value as int == (le_word(data@) as int) + (extra.value as int)
            && value.value <= limit.value,
        None => (le_word(data@) as int) + (extra.value as int) > u64::MAX
            || (le_word(data@) as int) + (extra.value as int) > limit.value,
    },
{
    let decoded = Counter { value: read_word(data) };
    match decoded.checked_add(extra) {
        Some(total) => if total.within(limit) { Some(total) } else { None },
        None => None,
    }
}
pub fn longer_input_and_equal_limit() -> (result: Option<Counter>)
    ensures result.is_some(), result->Some_0.value == 129,
{
    let limit = Counter { value: 129 };
    decode_add_and_bound(&[128u8, 0, 0, 0, 0, 0, 0, 0, 255], Counter { value: 1 }, &limit)
}
}
"#;

const FALSE_OVERFLOW: &str = r#"
verus! {
pub fn overflow_is_not_a_wrapped_sum() {
    let maximum = Counter { value: u64::MAX };
    let decoded = Counter { value: read_word(&[255u8, 255, 255, 255, 255, 255, 255, 255]) };
    let result = decoded.checked_add(Counter { value: 1 });
    assert(result.is_some());
    let within = maximum.within(&maximum);
    assert(within);
}
}
"#;

#[test]
fn byte_decoding_option_payloads_and_borrowed_bounds_compose() {
    let diagnostics = verify_one_file(
        "mixed_mechanical_composition",
        format!("{PROVIDER}{GOALS}"),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 3);
}

#[test]
fn composed_inference_does_not_turn_checked_overflow_into_wrapping() {
    let error = verify_one_file(
        "mixed_mechanical_false_overflow",
        format!("{PROVIDER}{FALSE_OVERFLOW}"),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(error.errors.iter().any(|diagnostic| diagnostic.message.contains("assertion failed")));
    let report = &error.json_output.as_ref().unwrap()["verus-infer"];
    assert_exact_report(report, 3);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
}

#[test]
fn mixed_provider_summaries_compose_in_a_real_certified_consumer() {
    let crates = InferCrates::new();
    let run =
        crates.build_source("provider", PROVIDER, &[], &["--no-cheating", "--output-json"], true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 3);
    let prefix = "use vstd::prelude::*; use provider::{Counter, WORD_BYTES, le_word, read_word};\n";
    let run = crates.build_source(
        "client",
        &format!("{prefix}{GOALS}"),
        &["provider"],
        &["--no-cheating", "--infer-goals", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(crates.vir("client").is_file());
    let report = json_inference(&run);
    assert!(!report["imported_inference_dependencies"].as_array().unwrap().is_empty());
    let run = crates.build_source(
        "client",
        &format!("{prefix}{FALSE_OVERFLOW}"),
        &["provider"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("client").exists());
}
