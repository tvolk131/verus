#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{assistance_decision, source_name};
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
    source_name("range", s)
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
    assert_eq!(assistance_decision(r, "range_loop", "::fill")["status"], "applied");
    assert_eq!(r["assistance_explanations"]["preparation_completed"], true);
    assert!(r["functions"].as_array().unwrap().is_empty(), "ordinary proof is not exact synthesis");
}
#[test]
fn inclusive_mutation_preserves_length_frames_and_other_fields() {
    succeeds(FILL);
}
#[test]
fn hidden_pointwise_predicate_needs_no_authored_index_trigger() {
    let source = FILL
        .replace(
            "struct Buffer",
            "spec fn filled(b:Buffer,i:int)->bool {b.values@[i]==1}\nstruct Buffer",
        )
        .replace("start<=i<=end ==>", "start<=i<=end && i<final(b).values@.len() ==>")
        .replace("#[trigger] final(b).values@[i]==1", "filled(*final(b),i)")
        .replace("#[trigger] ", "");
    succeeds(&source);
    let wrong = source.replace("b.values[i as usize]=1;", "b.values[i as usize]=2;");
    let e = verify_one_file(
        &name(&wrong),
        wrong,
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap_err();
    assert!(
        e.errors
            .iter()
            .any(|d| d.message.contains("invariant") || d.message.contains("postcondition")),
        "{:?}",
        e
    );
    assert_eq!(e.json_output.unwrap()["verus-infer"]["interface_written"], false);
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
            "{:?}",
            e
        );
    }
}
#[test]
fn source_invariants_remain_obligations() {
    let s = FILL.replace("for i in start..=end {", "for i in start..=end invariant false, {");
    let e =
        verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals", "--output-json"])
            .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("invariant")), "{:?}", e);
    assert_eq!(
        assistance_decision(
            &e.json_output.as_ref().unwrap()["verus-infer"],
            "range_loop",
            "::fill"
        )["reason"],
        "unsupported_iterator_proof_shape"
    );
}
#[test]
fn signed_ranges_are_not_selected_by_this_profile() {
    let s = FILL.replace("u16", "i16").replace("requires end<", "requires 0<=start, end<");
    let e =
        verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals", "--output-json"])
            .unwrap_err();
    assert_eq!(
        assistance_decision(
            &e.json_output.as_ref().unwrap()["verus-infer"],
            "range_loop",
            "::fill"
        )["reason"],
        "unsupported_range"
    );
    assert!(
        e.json_output.unwrap()["verus-infer"]["range_loop_assistance"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn range_explanations_identify_missing_final_state_and_goal_budget() {
    let ensures_start = FILL.find(" ensures ").unwrap();
    let body_start = FILL.find("{for i").unwrap();
    let no_goals = format!("{}{}", &FILL[..ensures_start], &FILL[body_start..]);
    for (source, reason, status) in [
        (no_goals, "missing_final_state_goal", "skipped"),
        (
            FILL.replace(
                "final(b).stamp==old(b).stamp,",
                &"final(b).stamp==old(b).stamp,".repeat(13),
            ),
            "goal_budget_exhausted",
            "budget_exhausted",
        ),
    ] {
        let diagnostics = verify_one_file(
            &name(&source),
            source,
            &["vstd", "--infer-goals", "--no-cheating", "--output-json"],
        )
        .unwrap_or_else(|error| error);
        let report = &diagnostics.json_output.unwrap()["verus-infer"];
        let decision = assistance_decision(report, "range_loop", "::fill");
        assert_eq!(decision["status"], status, "{report}");
        assert_eq!(decision["reason"], reason, "{report}");
        assert!(report["range_loop_assistance"].as_array().unwrap().is_empty());
    }
}
