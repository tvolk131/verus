#![feature(rustc_private)]

#[macro_use]
mod common;
use common::infer::{assert_exact_report, verify_exact};
use common::*;

#[test]
fn readonly_early_returns_compose_with_native_byte_roundtrips() {
    let source = r#"
use vstd::prelude::*;
verus! {
pub struct Reading { pub value: u64 }
impl Reading {
    #[verifier::infer]
    pub fn select(&self, stop: bool) -> u64 {
        if stop { return 0; }
        self.value
    }
}

#[verifier::infer]
pub fn roundtrip(value: u64) -> u64 {
    u64::from_le_bytes(value.to_le_bytes())
}
fn check(reading: &Reading, stop: bool) {
    let selected = reading.select(stop);
    proof {
        vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
        assert(vstd::bytes::spec_u64_from_le_bytes(
            vstd::bytes::spec_u64_to_le_bytes(selected),
        ) == selected);
    }
    let restored = roundtrip(selected);
    assert(restored == selected);
    assert(restored == if stop { 0 } else { reading.value });
}
}
"#;
    verify_exact("record_byte_composition", source.to_owned(), 2);
    let false_goal =
        source.replace("assert(restored == selected);", "assert(restored != selected);");
    let errors = verify_one_file(
        "record_byte_composition_false_goal",
        false_goal,
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert_exact_report(&errors.json_output.as_ref().unwrap()["verus-infer"], 2);
    assert!(errors.errors.iter().any(|error| error.message.contains("assertion failed")));
}

#[test]
fn native_option_callbacks_compose_with_checked_integer_and_byte_models() {
    let source = r#"
use vstd::prelude::*;
verus! {
pub struct Request { pub blocked: u32, pub marker: u32 }
impl Request {
    #[verifier::infer]
    pub fn classify(&self, value: Option<u64>) -> u32 {
        if self.blocked != 0 { return 0; }
        value.map_or(0, |number| u32::try_from(number).unwrap())
    }
}
#[verifier::infer]
pub fn encode(number: u32) -> [u8; 8] {
    u64::from(number).to_le_bytes()
}
fn caller(request: &Request, value: Option<u64>)
    requires request.blocked != 0 || match value {
        None => true,
        Some(number) => number <= u32::MAX,
    },
{
    let expected = if request.blocked != 0 { 0u32 } else {
        match value { None => 0u32, Some(number) => number as u32 }
    };
    let classified = request.classify(value);
    assert(classified == expected);
    let bytes = encode(classified);
    assert(bytes@ == vstd::bytes::spec_u64_to_le_bytes(expected as u64));
}
}
"#;
    verify_exact("native_transfer_composition", source.to_owned(), 2);
    let false_goal =
        source.replace("assert(classified == expected);", "assert(classified != expected);");
    let errors = verify_one_file(
        "native_transfer_composition_false_goal",
        false_goal,
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert_exact_report(&errors.json_output.as_ref().unwrap()["verus-infer"], 2);
    assert!(errors.errors.iter().any(|error| error.message.contains("assertion failed")));
}
