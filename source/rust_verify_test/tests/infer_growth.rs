#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{assistance_decision, source_name};
use common::*;

const SIMPLE: &str = r#"use vstd::prelude::*;verus!{
struct History {values:Vec<u8>,tag:u8}
fn grow(h:&mut History,n:usize)
 ensures final(h).values@.len()==if old(h).values@.len()>n {old(h).values@.len() as int}else{n as int},
 final(h).values@.take(old(h).values@.len() as int)==old(h).values@,
 final(h).tag==old(h).tag,
 forall|i:int|old(h).values@.len()<=i<final(h).values@.len() ==> #[trigger] final(h).values@[i]==0,
{while h.values.len()<n {h.values.push(0);}}
}"#;
const STRUCTURED: &str = r#"use vstd::prelude::*;verus!{
struct Snapshot {items:Vec<u8>,round:u16}
fn advance(s:&Snapshot)->(r:Snapshot)
 requires s.round<65535,
 ensures r.round==s.round+1,r.items@==s.items@,
{Snapshot{items:s.items.clone(),round:s.round+1}}
struct History {values:Vec<Snapshot>,stamp:u8}
impl History {
 closed spec fn valid(&self)->bool {
  0<self.values@.len()<=65535 && forall|i:int|#![trigger self.values@[i]] 0<=i<self.values@.len() ==> self.values@[i].round==i+1
 }
 fn grow(&mut self,round:u16)
 requires old(self).valid(),round<65535,
 ensures final(self).valid(),final(self).stamp==old(self).stamp,
  final(self).values@.len()==if old(self).values@.len()>round as int+1 {old(self).values@.len() as int}else{round as int+1},
  final(self).values@.take(old(self).values@.len() as int)==old(self).values@,
 {
  while self.values.len()<=round as usize {
   let next=advance(self.values.last().unwrap());
   self.values.push(next);
  }
 }
}
}"#;
fn name(s: &str) -> String {
    source_name("growth", s)
}
fn succeeds(s: &str) {
    let d = verify_one_file(
        &name(s),
        s.into(),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    let r = &d.json_output.unwrap()["verus-infer"];
    assert_eq!(r["growth_loop_assistance"].as_array().unwrap().len(), 1, "{r}");
    let decision = assistance_decision(r, "growth_loop", "::grow");
    assert_eq!(decision["status"], "applied");
    assert_eq!(decision["reason"], serde_json::Value::Null);
    assert_eq!(r["assistance_explanations"]["preparation_completed"], true);
    assert!(
        r["functions"].as_array().unwrap().is_empty(),
        "ordinary proofs must not claim exact inference"
    );
}
#[test]
fn append_preserves_prefix_frames_and_describes_new_elements() {
    succeeds(SIMPLE);
}
#[test]
fn nested_collection_elements_compose_checked_transition_contracts() {
    succeeds(STRUCTURED);
}
#[test]
fn no_assistance_requires_manual_loop_proofs() {
    assert!(
        verify_one_file("unassisted_growth", SIMPLE.into(), &["vstd", "--no-cheating"]).is_err()
    );
}
#[test]
fn wrong_values_frames_prefixes_and_progress_are_rejected() {
    for s in [
        SIMPLE.replace("push(0)", "push(1)"),
        SIMPLE.replace("h.values.push(0);", "h.values.push(0);h.tag=0;"),
        SIMPLE.replace("h.values.push(0);", "h.values.push(0);h.values[0]=0;"),
        SIMPLE.replace("h.values.push(0);", "if false {h.values.push(0);}"),
        STRUCTURED.replace(
            "requires old(self).valid(),round<65535,",
            "requires old(self).valid(),round<=65535,",
        ),
        STRUCTURED.replace("round:s.round+1", "round:s.round"),
    ] {
        let e = verify_one_file(
            &name(&s),
            s,
            &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
        )
        .unwrap_err();
        let report = &e.json_output.as_ref().unwrap()["verus-infer"];
        assert_eq!(assistance_decision(report, "growth_loop", "::grow")["status"], "applied");
        assert_eq!(report["crate_success"], false);
        assert_eq!(report["interface_written"], false);
        assert!(
            e.errors.iter().any(|d| d.message.contains("invariant")
                || d.message.contains("postcondition")
                || d.message.contains("precondition")
                || d.message.contains("decreases")),
            "{:?}",
            e
        );
    }
}
#[test]
fn explicit_loop_proofs_are_preserved() {
    let s = SIMPLE.replace(
        "while h.values.len()<n",
        "while h.values.len()<n invariant false, decreases 0int, ",
    );
    let e =
        verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals", "--output-json"])
            .unwrap_err();
    assert_eq!(
        assistance_decision(
            &e.json_output.as_ref().unwrap()["verus-infer"],
            "growth_loop",
            "::grow"
        )["reason"],
        "authored_loop_proof"
    );
    assert!(
        e.json_output.unwrap()["verus-infer"]["growth_loop_assistance"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(e.errors.iter().any(|d| d.message.contains("invariant")));
}

const SELECTOR: &str = r#"use vstd::prelude::*;verus!{
pub struct Slot {pub number:u16}
pub struct History {pub slots:Vec<Slot>,pub tag:u8}
impl History {
 pub open spec fn valid(&self)->bool {
  0<self.slots@.len()<=u16::MAX && forall|j:int|0<=j<self.slots@.len()
   ==> self.slots@[j].number==j
 }
 pub fn grow(&mut self,index:u16)
  requires old(self).valid(),index<u16::MAX,
  ensures final(self).valid(),final(self).tag==old(self).tag,
   final(self).slots@.len()==if old(self).slots@.len()>index as int+1 {old(self).slots@.len() as int}else{index as int+1},
   final(self).slots@.take(old(self).slots@.len() as int)==old(self).slots@,
 {
  while self.slots.len()<=index as usize {
   let number=self.slots.len() as u16;
   self.slots.push(Slot{number});
  }
 }
 pub fn select(&mut self,index:u16)->(slot:&mut Slot)
  requires old(self).valid(),index<u16::MAX,
  ensures slot.number==index,
 {
  self.grow(index);
  &mut self.slots[index as usize]
 }
}
pub fn caller(s:&mut History,index:u16)
 requires old(s).valid(),index<u16::MAX,
 ensures final(s).valid(),final(s).tag==old(s).tag,
  final(s).slots@.len()==if old(s).slots@.len()>index as int+1 {old(s).slots@.len() as int}else{index as int+1},
  forall|j:int|0<=j<old(s).slots@.len() && j!=index ==> final(s).slots@[j]==old(s).slots@[j],
{
 let slot=s.select(index);
 slot.number=index;
}
}"#;

#[test]
fn mutable_selection_generates_identity_frames_and_conditional_invariant_restoration() {
    let d = verify_one_file(
        "selector_checked_frames",
        SELECTOR.into(),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    let report = &d.json_output.unwrap()["verus-infer"];
    assert_eq!(report["mutable_selector_assistance"].as_array().unwrap().len(), 1, "{report}");
    assert!(
        report["mutable_selector_assistance"][0]["postconditions"].as_u64().unwrap() >= 5,
        "{:?}",
        report
    );
    assert!(report["functions"].as_array().unwrap().is_empty());
}

fn inline_selector() -> String {
    let source = SELECTOR
        .replace(
            "pub struct Slot {pub number:u16}",
            "pub struct Slot {pub number:u16,pub payload:u64}",
        )
        .replace(
            "  self.grow(index);",
            r#"  while self.slots.len()<=index as usize {
   let number=self.slots.len() as u16;
   self.slots.push(Slot{number,payload:0});
  }"#,
        );
    let first = source.find(" pub fn grow").unwrap();
    let last = source.find(" pub fn select").unwrap();
    format!("{}{}", &source[..first], &source[last..]).replace("pub fn caller(s:",r#"
pub fn untouched(s:&mut History,index:u16)
 requires old(s).valid(),index<u16::MAX,
 ensures final(s).valid(),final(s).tag==old(s).tag,
  final(s).slots@.len()==if old(s).slots@.len()>index as int+1 {old(s).slots@.len() as int}else{index as int+1},
  final(s).slots@.take(old(s).slots@.len() as int)==old(s).slots@,
 {let _=s.select(index);}
pub fn caller(s:"#)
}

#[test]
fn inline_growth_and_mutable_return_need_only_semantic_input_and_result_contracts() {
    let source = inline_selector();
    let d = verify_one_file(
        "inline_selector",
        source,
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    let report = &d.json_output.unwrap()["verus-infer"];
    assert_eq!(report["growth_loop_assistance"].as_array().unwrap().len(), 1);
    assert_eq!(report["mutable_selector_assistance"].as_array().unwrap().len(), 1);
    assert!(report["functions"].as_array().unwrap().is_empty());
}

#[test]
fn inline_selector_checks_growth_identity_frames_and_future_invariant_restoration() {
    let original = inline_selector();
    for (i, source) in [
        original.replace("slot.number=index;", "slot.number=0;"),
        original.replace("&mut self.slots[index as usize]", "&mut self.slots[0]"),
        original.replace("let number=self.slots.len() as u16;", "let number=0;"),
        original.replace(
            "self.slots.push(Slot{number,payload:0});",
            "self.tag=0;self.slots.push(Slot{number,payload:0});",
        ),
        original.replace(
            "while self.slots.len()<=index as usize",
            "while self.slots.len()<index as usize",
        ),
    ]
    .iter()
    .enumerate()
    {
        let e = verify_one_file(
            &format!("inline_selector_wrong_{i}"),
            source.clone(),
            &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
        )
        .unwrap_err();
        assert!(
            e.errors.iter().any(|d| d.message.contains("postcondition")
                || d.message.contains("invariant")
                || d.message.contains("precondition")),
            "{:?}",
            e
        );
        assert_eq!(e.json_output.unwrap()["verus-infer"]["interface_written"], false);
    }
}

#[test]
fn mutable_selection_never_assumes_input_validity_frames_or_future_borrower_behavior() {
    for (i, source) in [
        SELECTOR.replace("slot.number=index;", "slot.number=0;"),
        SELECTOR.replace("&mut self.slots[index as usize]", "&mut self.slots[0]"),
        SELECTOR.replace("let number=self.slots.len() as u16;", "let number=0;"),
        SELECTOR.replace("final(self).tag==old(self).tag,", ""),
    ]
    .iter()
    .enumerate()
    {
        let e = verify_one_file(
            &format!("selector_wrong_{i}"),
            source.clone(),
            &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
        )
        .unwrap_err();
        assert!(
            e.errors
                .iter()
                .any(|d| d.message.contains("postcondition") || d.message.contains("invariant")),
            "{:?}",
            e
        );
        assert_eq!(e.json_output.unwrap()["verus-infer"]["interface_written"], false);
    }
}

#[test]
fn unrelated_zero_argument_preparation_retains_ordinary_borrow_verification() {
    let source = r#"use vstd::prelude::*;verus!{
     struct Buffer{values:Vec<u8>}
     fn noop(){}
     fn select(b:&mut Buffer,i:usize)->(r:&mut u8)
      requires i<old(b).values@.len(),
      ensures *r==old(b).values@[i as int],
     {noop();&mut b.values[i]}
    }"#;
    let d = verify_one_file(
        "selector_unmatched_preparation",
        source.into(),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    assert!(
        d.json_output.unwrap()["verus-infer"]["mutable_selector_assistance"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn growth_explanations_identify_missing_ambiguous_and_over_budget_goals() {
    let length_goal = "final(h).values@.len()==if old(h).values@.len()>n {old(h).values@.len() as int}else{n as int},";
    for (source, reason, status) in [
        (SIMPLE.replace(length_goal, ""), "missing_final_length_goal", "skipped"),
        (
            SIMPLE.replace(length_goal, &length_goal.repeat(2)),
            "ambiguous_final_length_goal",
            "skipped",
        ),
        (
            SIMPLE.replace("final(h).tag==old(h).tag,", &"final(h).tag==old(h).tag,".repeat(13)),
            "goal_budget_exhausted",
            "budget_exhausted",
        ),
    ] {
        let result = verify_one_file(
            &name(&source),
            source,
            &["vstd", "--infer-goals", "--no-cheating", "--output-json"],
        );
        let diagnostics = result.unwrap_or_else(|error| error);
        let report = &diagnostics.json_output.unwrap()["verus-infer"];
        let decision = assistance_decision(report, "growth_loop", "::grow");
        assert_eq!(decision["status"], status, "{report}");
        assert_eq!(decision["reason"], reason, "{report}");
        assert!(report["growth_loop_assistance"].as_array().unwrap().is_empty());
        assert_eq!(report["assistance_explanations"]["preparation_completed"], true);
    }
}

#[test]
fn failed_growth_preparation_reports_the_error_without_claiming_later_attempts() {
    let source =
        SIMPLE.replace("final(h).tag==old(h).tag", "final(h).tag==(choose|x:u8| x==old(h).tag)");
    let diagnostics = verify_one_file(
        &name(&source),
        source,
        &["vstd", "--infer-goals", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    let report = &diagnostics.json_output.unwrap()["verus-infer"];
    let decision = assistance_decision(report, "growth_loop", "::grow");
    assert_eq!(decision["status"], "error", "{report}");
    assert_eq!(decision["reason"], "preparation_error");
    assert!(decision["message"].as_str().unwrap().contains("growth annotation"));
    let explanations = &report["assistance_explanations"];
    assert_eq!(explanations["preparation_started"], true);
    assert_eq!(explanations["preparation_completed"], false);
    assert!(
        explanations["decisions"].as_array().unwrap().iter().all(|d| d["profile"] == "growth_loop")
    );
    assert_eq!(report["interface_written"], false);
}
