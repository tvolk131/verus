#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::assert_exact_report;
use common::*;

const CONVERSIONS: &str = r#"
use vstd::prelude::*;
#[verus_infer]
fn take_four(data: &[u16]) -> [u16; 4] {
    <[u16; 4]>::try_from(data).unwrap()
}
#[verus_infer]
fn take_empty(data: &[bool]) -> [bool; 0] {
    data.try_into().unwrap()
}
"#;

#[test]
fn canonical_direct_and_forwarded_conversion_preserve_length_and_elements() {
    let diagnostics = verify_one_file(
        "canonical_slice_array",
        format!(
            r#"{CONVERSIONS}
verus! {{
fn values(data: &[u16]) -> (result: [u16; 4])
    requires data.len() == 4,
    ensures result@ == data@,
{{ take_four(data) }}
fn empty(data: &[bool]) -> (result: [bool; 0])
    requires data.len() == 0,
    ensures result@ == data@,
{{ take_empty(data) }}
}}
"#
        ),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json", "--compile"],
    )
    .unwrap();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 2);
}

#[test]
fn too_long_conversion_is_not_mistaken_for_prefix_conversion() {
    let diagnostics = verify_one_file(
        "slice_array_wrong_length",
        format!(
            r#"{CONVERSIONS}
verus! {{ fn too_long(data: &[u16]) -> [u16; 4]
    requires data.len() == 5,
{{ take_four(data) }} }}
"#
        ),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json", "--compile"],
    )
    .unwrap_err();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 2);
    assert!(diagnostics.errors.iter().any(|error| error.message.contains("precondition")));
}

#[test]
fn wrong_element_property_is_rejected_after_all_exact_checks() {
    let diagnostics = verify_one_file(
        "slice_array_wrong_value",
        format!(
            r#"{CONVERSIONS}
verus! {{ fn wrong_value() {{
    let result = take_four(&[1u16, 2, 3, 65535]);
    assert(result[3] == 0);
}} }}
"#
        ),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json", "--compile"],
    )
    .unwrap_err();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 2);
    assert!(diagnostics.errors.iter().any(|error| error.message.contains("assertion failed")));
}
