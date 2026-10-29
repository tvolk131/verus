#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const FILL: &str = r#"use vstd::prelude::*;verus!{
struct Buffer {values:Vec<u8>,stamp:u8}
fn fill(b:&mut Buffer,start:u16,end:u16)
 requires end<old(b).values@.len(),
 ensures final(b).values@.len()==old(b).values@.len(),final(b).stamp==old(b).stamp,
  forall|i:int|start<=i<=end ==> #[trigger] final(b).values@[i]==1,
  forall|i:int|0<=i<old(b).values@.len() && (i<start || i>end) ==>
    #[trigger] final(b).values@[i]==old(b).values@[i],
{for i in start..=end {b.values[i as usize]=1;}}
}"#;
fn name(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("range_{:x}", h.finish())
}
fn succeeds(s: &str) {
    let d = verify_one_file(
        &name(s),
        s.into(),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    let r = &d.json_output.unwrap()["verus-infer"];
    assert_eq!(r["range_loop_assistance"].as_array().unwrap().len(), 1, "{r}");
    assert!(r["functions"].as_array().unwrap().is_empty(), "ordinary proof is not exact synthesis");
}
#[test]
fn inclusive_mutation_preserves_length_frames_and_other_fields() {
    succeeds(FILL);
}
#[test]
fn exclusive_usize_range_including_empty_ranges() {
    succeeds(
        &FILL
            .replace("u16", "usize")
            .replace("start..=end", "start..end")
            .replace("start<=i<=end", "start<=i<end")
            .replace("i>end", "i>=end"),
    );
}
#[test]
fn inclusive_range_supports_maximum_and_singleton_endpoints() {
    succeeds(&FILL.replace("u16", "u8").replace(
        "\n}",
        r#"
fn last(b:&mut Buffer)
 requires old(b).values@.len()==256,
 ensures final(b).values@[255]==1,final(b).values@.len()==256,
{fill(b,255,255);}
}"#,
    ));
}
#[test]
fn no_assistance_needs_manual_induction() {
    assert!(
        verify_one_file("range_no_assistance", FILL.into(), &["vstd", "--no-cheating"]).is_err()
    );
}
#[test]
fn changed_values_slots_frames_range_and_skipped_steps_are_rejected() {
    for s in [
        FILL.replace("=1;", "=2;"),
        FILL.replace("i as usize", "0usize"),
        FILL.replace("=1;", "=1;b.stamp=0;"),
        FILL.replace("start..=end", "start..end"),
        FILL.replace("b.values[i as usize]=1;", "if i!=start {b.values[i as usize]=1;}"),
        FILL.replace("b.values[i as usize]=1;", "break;"),
        FILL.replace("requires end<old(b).values@.len(),", "requires true,"),
    ] {
        let e = verify_one_file(
            &name(&s),
            s,
            &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
        )
        .unwrap_err();
        let r = &e.json_output.as_ref().unwrap()["verus-infer"];
        assert!(!r["crate_success"].as_bool().unwrap());
        assert_eq!(r["interface_written"], false);
        assert!(
            e.errors.iter().any(|d| d.message.contains("invariant")
                || d.message.contains("postcondition")
                || d.message.contains("precondition")),
            "{:?}",e
        );
    }
}
#[test]
fn source_invariants_remain_obligations() {
    let s = FILL.replace("for i in start..=end {", "for i in start..=end invariant false, {");
    let e =
        verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals", "--output-json"])
            .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("invariant")), "{:?}",e);
}
#[test]
fn signed_ranges_are_not_selected_by_this_profile() {
    let s = FILL.replace("u16", "i16").replace("requires end<", "requires 0<=start, end<");
    let e =
        verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals", "--output-json"])
            .unwrap_err();
    assert!(
        e.json_output.unwrap()["verus-infer"]["range_loop_assistance"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
