#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::assert_exact_report;
use common::*;

const BASE: &str = r#"
use vstd::prelude::*;
use core::mem::size_of;
#[verus_verify]
struct Layout;
#[verus_verify]
impl Layout {
    const fn word_bytes() -> usize { size_of::<u64>() }
}
#[verus_verify]
const SECOND_WORD: usize = Layout::word_bytes();
#[verus_verify]
const HEADER_BYTES: usize = SECOND_WORD + size_of::<u64>();
#[verus_infer]
fn read_word(data: &[u8]) -> u64 {
    u64::from_le_bytes(data[..size_of::<u64>()].try_into().unwrap())
}
verus! {
struct Header { first: u64, second: u64 }
pub open spec fn little_endian(data: Seq<u8>, offset: int) -> u64 {
    (data[offset] as u64) | ((data[offset + 1] as u64) << 8)
    | ((data[offset + 2] as u64) << 16) | ((data[offset + 3] as u64) << 24)
    | ((data[offset + 4] as u64) << 32) | ((data[offset + 5] as u64) << 40)
    | ((data[offset + 6] as u64) << 48) | ((data[offset + 7] as u64) << 56)
}
}
"#;

#[test]
fn constant_dag_and_slice_conversion_compose_without_byte_proof_steps() {
    let diagnostics = verify_one_file(
        "composed_byte_header",
        format!(
            r#"{BASE}
verus! {{
fn decode_header(data: &[u8]) -> (result: Header)
    requires data.len() >= HEADER_BYTES,
    ensures result.first == little_endian(data@, 0),
        result.second == little_endian(data@, SECOND_WORD as int),
{{
    Header {{ first: read_word(data), second: read_word(&data[SECOND_WORD..]) }}
}}
fn longer_input() -> (result: u64)
    ensures result == 128,
{{
    read_word(&[128u8, 0, 0, 0, 0, 0, 0, 0, 255])
}}
}}
"#
        ),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 1);
}

#[test]
fn wrong_endian_goal_is_a_semantic_rejection_not_a_frontend_decline() {
    let error = verify_one_file(
        "composed_byte_wrong_endian",
        format!(
            r#"{BASE}
verus! {{ fn wrong_endian() {{
    let result = read_word(&[128u8, 0, 0, 0, 0, 0, 0, 0]);
    assert(result == 0x8000_0000_0000_0000u64);
}} }}
"#
        ),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(error.errors.iter().any(|diagnostic| diagnostic.message.contains("assertion failed")));
    let report = &error.json_output.as_ref().unwrap()["verus-infer"];
    assert_exact_report(report, 1);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
}

#[test]
fn short_slice_remains_a_safety_rejection_after_exact_inference() {
    let error = verify_one_file(
        "composed_byte_short_input",
        format!(
            r#"{BASE}
verus! {{ fn short_input(data: &[u8]) -> u64
    requires data.len() < 8,
{{ read_word(data) }} }}
"#
        ),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(error.errors.iter().any(|diagnostic| diagnostic.message.contains("precondition")));
    let report = &error.json_output.as_ref().unwrap()["verus-infer"];
    assert_exact_report(report, 1);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
}

#[test]
fn constant_values_follow_the_original_helper_body_not_its_name() {
    let source = BASE.replace(
        "const fn word_bytes() -> usize { size_of::<u64>() }",
        "const fn word_bytes() -> usize { size_of::<u8>() }",
    );
    let error = verify_one_file(
        "composed_byte_changed_layout",
        format!("{source} verus! {{ fn wrong_layout() {{ assert(SECOND_WORD == 8); }} }}"),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(error.errors.iter().any(|diagnostic| diagnostic.message.contains("assertion failed")));
    let report = &error.json_output.as_ref().unwrap()["verus-infer"];
    assert_exact_report(report, 1);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
}
