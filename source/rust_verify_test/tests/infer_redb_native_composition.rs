#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::assert_exact_report;
use common::*;

const NATIVE_BLOCK_COMPOSITION: &str =
    include_str!("../../../evidence/redb-native-composition-cycle1/candidate.rs");

#[test]
fn full_frozen_native_block_composition_has_fresh_exact_helpers() {
    let diagnostics = verify_one_file(
        "redb_frozen_blocks",
        NATIVE_BLOCK_COMPOSITION.to_string(),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json", "--compile"],
    )
    .unwrap();
    let report = &diagnostics.json_output.as_ref().unwrap()["verus-infer"];
    assert_exact_report(report, 5);
    assert_eq!(report["crate_success"], true);
    assert_eq!(report["verification_scope"]["complete_selected_scope_verified"], true);
    assert_eq!(report["imported_inference_dependencies"], serde_json::json!([]));
}

#[test]
fn wrong_clean_property_is_semantically_rejected_after_exact_helpers() {
    let source = NATIVE_BLOCK_COMPOSITION.replace(
        "clean == (!byte_selection_moves(frame, data0@, data1@, bad0, bad1) && !layout_stale)",
        "clean == (!byte_selection_moves(frame, data0@, data1@, bad0, bad1) && layout_stale)",
    );
    assert_ne!(source, NATIVE_BLOCK_COMPOSITION);
    let diagnostics = verify_one_file(
        "redb_wrong_clean",
        source,
        &["vstd", "--no-cheating", "--infer-goals", "--output-json", "--compile"],
    )
    .unwrap_err();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 5);
    assert!(diagnostics.errors.iter().any(|error| error.message.contains("postcondition")));
    assert!(!diagnostics.errors.iter().any(|error| error.message.contains("precondition")));
}

#[test]
fn corrupted_short_tail_is_a_safety_failure_not_a_semantic_negative() {
    let source = format!(
        r#"{NATIVE_BLOCK_COMPOSITION}
verus! {{ fn corrupt_short(data: &[u8]) {{
    requires(data.len() >= 112 && data.len() < 128);
    let _ = TransactionHeader::from_id_tail(data, 0, None, None, true);
}} }}
"#
    );
    let diagnostics = verify_one_file(
        "redb_corrupt_short",
        source,
        &["vstd", "--no-cheating", "--infer-goals", "--output-json", "--compile"],
    )
    .unwrap_err();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], 5);
    assert!(diagnostics.errors.iter().any(|error| error.message.contains("precondition")));
}
