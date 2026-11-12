#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const REDUCED: &str =
    include_str!("../fixtures/infer/word_sequence/redb_tree_reduced.rs");
const CONTROLS: &str =
    include_str!("../fixtures/infer/word_sequence/redb_tree_controls.rs");
fn check(name: &str, source: String, infer: bool, success: bool, extra: &[&str]) {
    let mut options = vec!["vstd", "--no-cheating", "--output-json", "--edition 2024"];
    if infer {
        options.push("--infer-goals")
    }
    options.extend_from_slice(extra);
    let result = verify_one_file(name, source, &options);
    assert_eq!(result.is_ok(), success, "{name}: {result:?}");
    if let Err(e) = result {
        assert!(e.errors.iter().all(|e| e.code.is_none()), "frontend failure: {e:?}");
        assert!(
            e.errors.iter().any(|e| e.message.contains("postcondition not satisfied")
                || e.message.contains("assertion failed")
                || e.message.contains("precondition not satisfied")),
            "not an ordinary proof failure: {e:?}"
        );
        assert!(
            !e.errors.iter().any(|e| e.message.to_lowercase().contains("resource limit")),
            "resource outcome: {e:?}"
        );
    }
}
#[test]
fn full_tree_physical_frame_including_padding_and_repeats() {
    check("word_sequence_full", REDUCED.into(), true, true, &[])
}
#[test]
fn assistance_disabled_only_word_writers_fail() {
    check("word_sequence_off", REDUCED.into(), false, false, &[])
}
#[test]
fn field_and_application_names_are_irrelevant() {
    let source = REDUCED
        .replace("BtreeBitmap", "Hierarchy")
        .replace("U64GroupedBitmap", "WordStorage")
        .replace("physical_bit", "observed_flag")
        .replace("word_change", "edited_storage")
        .replace("data:", "storage:")
        .replace(".data", ".storage");
    check("word_sequence_renamed", source, true, true, &[]);
}
#[test]
fn false_membership_count_and_frame_are_rejected() {
    for cfg in ["negative_leaf", "negative_count", "negative_frame"] {
        let selected = CONTROLS.replace(&format!("#[cfg({cfg})]"), "");
        check(cfg, format!("{REDUCED}\n{selected}"), true, false, &[]);
    }
}
#[test]
fn changed_executable_update_does_not_receive_an_assumed_frame() {
    check(
        "word_sequence_wrong_update",
        REDUCED.replace(
            "group |= Self::select_mask(bit_index);",
            "group &= !Self::select_mask(bit_index);",
        ),
        true,
        false,
        &[],
    );
}
#[test]
fn unrelated_valid_and_false_programs_remain_compatible() {
    check(
        "word_sequence_unrelated",
        "use vstd::prelude::*; verus!{fn f(x:u64)->(r:u64) ensures r==x {x}} fn main(){}".into(),
        true,
        true,
        &[],
    );
    check(
        "word_sequence_unrelated_false",
        "use vstd::prelude::*; verus!{fn f(x:u64)->(r:u64) ensures r!=x {x}} fn main(){}".into(),
        true,
        false,
        &[],
    );
}

#[test]
fn missing_valid_bit_safety_domain_is_not_invented() {
    check(
        "word_sequence_missing_safety",
        REDUCED.replace(
            "requires layer_wf(*old(self)), bit < old(self).len,",
            "requires layer_wf(*old(self)),",
        ),
        true,
        false,
        &[],
    );
}
