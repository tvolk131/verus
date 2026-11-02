#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference, verify_exact};
use common::*;

const QUERIES: &str = r#"
use vstd::prelude::*;
verus! {
pub struct Readings { pub value: u8, pub samples: Vec<u8> }
impl Readings {
    #[verifier::infer]
    pub fn select(&self, first: bool, second: bool) -> u8 {
        if first {
            if second { return self.value; }
            return 7;
        }
        if second { return 9; }
        self.value
    }
    #[verifier::infer]
    pub fn optional(&self, input: Option<u8>, stop: bool) -> Option<u8> {
        match input {
            None => return None,
            Some(value) => { if stop { return Some(value); } },
        }
        Some(self.value)
    }
    #[verifier::infer]
    pub fn branch(&self, stop: bool) -> Option<u8> {
        if stop { return None; } else { Some(self.value) }
    }
    #[verifier::infer]
    pub fn all_return(&self, input: Option<u8>) -> u8 {
        match input {
            None => return self.value,
            Some(value) => return value,
        }
    }
    #[verifier::infer]
    pub fn saturation(&self, stop: bool, value: u8) -> u8 {
        if stop { return self.value.saturating_add(value); }
        self.value.saturating_sub(value)
    }
}
}
"#;

#[test]
fn nested_returns_match_returns_and_tail_values_have_exact_contracts() {
    let source = QUERIES.trim_end().strip_suffix('}').unwrap().to_owned()
        + r#"
fn check(record: &Readings, first: bool, second: bool, input: Option<u8>) {
    let selected = record.select(first, second);
    assert(selected == if first { if second { record.value } else { 7 } } else { if second { 9 } else { record.value } });
    let optional = record.optional(input, first);
    assert(optional == match input { None => None, Some(value) => if first { Some(value) } else { Some(record.value) } });
    let branch = record.branch(first);
    assert(branch == if first { None } else { Some(record.value) });
    let all = record.all_return(input);
    assert(all == match input { None => record.value, Some(value) => value });
    let high = record.saturation(true, 1);
    assert(high == if record.value < 255 { record.value + 1 } else { 255 });
    let low = record.saturation(false, 1);
    assert(low == if record.value > 0 { record.value - 1 } else { 0 });
}
}
"#;
    verify_exact("nested_record_returns", source, 5);
}

const SAFETY: &str = r#"
use vstd::prelude::*;
verus! {
pub struct Readings { pub value: u8, pub samples: Vec<u8> }
#[verifier::infer] pub fn increment(value: u8) -> u8 { value + 1 }
impl Readings {
    #[verifier::infer]
    pub fn guarded(&self, stop: bool, index: usize) -> u8 {
        if stop { return self.value; }
        if stop { return increment(self.value) + self.samples[index]; }
        3
    }
    #[verifier::infer]
    pub fn partial(&self, take: bool) -> u8 {
        if take { return increment(self.value); }
        0
    }
    #[verifier::infer]
    pub fn arithmetic(&self, take: bool) -> u8 {
        if take { return self.value + 1; }
        self.value - 1
    }
    #[verifier::infer]
    pub fn indexed(&self, take: bool, index: usize) -> u8 {
        if take { return self.samples[index]; }
        0
    }
    #[verifier::infer]
    pub fn matched(&self, input: Option<u8>) -> u8 {
        match input {
            Some(value) if self.value + 1 > value => return value,
            _ => {},
        }
        self.value
    }
    #[verifier::infer]
    pub fn ended(&self, stop: bool, index: usize) -> u8 {
        if stop { return 0; } else { return 1; }
        self.samples[index] + increment(self.value)
    }
}
fn check(record: &Readings, stop: bool, index: usize) {
    let value = record.guarded(stop, index);
    assert(value == if stop { record.value } else { 3 });
    let zero = record.partial(false);
    assert(zero == 0);
    let unindexed = record.indexed(false, index);
    assert(unindexed == 0);
    let unmatched = record.matched(None);
    assert(unmatched == record.value);
    let ended = record.ended(stop, index);
    assert(ended == if stop { 0u8 } else { 1u8 });
}
fn executed(record: &Readings, index: usize)
    requires record.value < 255, index < record.samples@.len(),
{
    let value = record.partial(true);
    assert(value == record.value + 1);
    let read = record.indexed(true, index);
    assert(read == record.samples@[index as int]);
}
fn normal(record: &Readings) requires record.value > 0 {
    let value = record.arithmetic(false);
    assert(value == record.value - 1);
}
}
"#;

#[test]
fn unreachable_safety_is_ignored_and_executed_return_requirements_are_kept() {
    verify_exact("return_safety", SAFETY.to_owned(), 7);
}

#[test]
fn unsafe_executed_returns_and_tail_paths_reject_unrestricted_callers() {
    for call in [
        "record.partial(true)",
        "record.arithmetic(true)",
        "record.arithmetic(false)",
        "record.indexed(true, 0)",
        "record.matched(Some(0))",
    ] {
        let source = SAFETY.trim_end().strip_suffix('}').unwrap().to_owned()
            + &format!("fn unrestricted(record: &Readings) {{ let value = {call}; }}\n}}\n");
        let errors = verify_one_file(
            "unsafe_record_return",
            source,
            &["vstd", "--no-cheating", "--output-json"],
        )
        .unwrap_err();
        assert_exact_report(&errors.json_output.as_ref().unwrap()["verus-infer"], 7);
        assert!(
            errors.errors.iter().any(|error| error.message.contains("precondition not satisfied")),
            "{:?}",
            errors
        );
    }
}

#[test]
fn generic_shared_option_return_and_unit_effect_regression() {
    verify_exact(
        "shared_and_unit_returns",
        r#"
use vstd::prelude::*;
verus! {
pub struct Slot<T> { pub value: T }
impl<T> Slot<T> {
    #[verifier::infer]
    pub fn get(&self, stop: bool) -> Option<&T> {
        if stop { return None; }
        Some(&self.value)
    }
}
fn check<T>(record: &Slot<T>, stop: bool) {
    let value = record.get(stop);
    assert(value == if stop { None } else { Some(&record.value) });
}
pub struct Counter { pub value: u8 }
impl Counter {
    #[verifier::infer]
    pub fn set(&mut self, stop: bool, value: u8) {
        if stop { return; }
        self.value = value;
    }
}
fn check_set(record: &mut Counter, stop: bool, value: u8)
    ensures final(record).value == if stop { old(record).value } else { value },
{ record.set(stop, value); }
}
"#
        .to_owned(),
        2,
    );
}

const PROVIDER: &str = r#"
use vstd::prelude::*;
pub struct Reading { pub value: u8 }
impl Reading {
    #[verifier::infer]
    pub fn query(&self, stop: bool) -> u8 {
        if stop { return 0; }
        self.value + 1
    }
    #[verifier::infer]
    pub fn clamp(&self, stop: bool, value: u8) -> u8 {
        if stop { return 0; }
        self.value.saturating_sub(value)
    }
}
"#;

#[test]
fn imported_returns_retain_domains_fresh_proofs_and_failed_goals_remove_exports() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let provider = crates.build("provider", PROVIDER, &[], &["--no-cheating", "--output-json"]);
        assert!(provider.status.success(), "{}", String::from_utf8_lossy(&provider.stderr));
        assert_exact_report(&json_inference(&provider), 2);
    }
    let client = r#"
use vstd::prelude::*;
extern crate provider;
use provider::Reading;
#[verifier::infer]
fn forward(record: &Reading, stop: bool) -> u8 {
    if stop { return record.clamp(true, 0); }
    record.query(false)
}
fn total(record: &Reading) {
    let value = forward(record, true);
    assert(value == 0);
}
fn checked(record: &Reading) requires record.value < 255 {
    let value = forward(record, false);
    assert(value == record.value + 1);
}
"#;
    let run = crates.build("client", client, &["provider"], &["--no-cheating", "--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
    assert!(crates.vir("client").exists());
    for goal in [
        "fn false_goal(record: &Reading) { let value = forward(record, true); assert(value == 1); }",
        "fn unsafe_goal(record: &Reading) { let value = forward(record, false); }",
    ] {
        let failed = crates.build(
            "client",
            &(client.to_owned() + goal),
            &["provider"],
            &["--no-cheating", "--output-json"],
        );
        assert!(!failed.status.success());
        assert_exact_report(&json_inference(&failed), 1);
        assert!(!crates.vir("client").exists());
        let stderr = String::from_utf8_lossy(&failed.stderr);
        assert!(
            stderr.contains("assertion failed") || stderr.contains("precondition not satisfied"),
            "{}",
            stderr
        );
    }
    let mutant = PROVIDER.replacen("return 0;", "return 1;", 1)
        + "fn policy(record: &Reading) { let value = record.query(true); assert(value == 0); }";
    let failed = crates.build("provider", &mutant, &[], &["--no-cheating", "--output-json"]);
    assert!(!failed.status.success());
    assert_exact_report(&json_inference(&failed), 2);
    assert!(String::from_utf8_lossy(&failed.stderr).contains("assertion failed"));
    assert!(!crates.vir("provider").exists());
}

#[test]
fn borrowed_result_return_profile_keeps_error_and_success_paths() {
    verify_exact("borrowed_result_returns", r#"
use vstd::prelude::*;
verus! {
pub struct Samples { pub values: Vec<u8> }
impl Samples {
    #[verifier::infer]
    pub fn query(&self, stop: bool, index: usize) -> Result<&Vec<u8>, u8> {
        if stop { return Err(1); }
        if index >= self.values.len() { return Err(2); }
        Ok(&self.values)
    }
}
fn check(record: &Samples, stop: bool, index: usize) -> (result: Result<&Vec<u8>, u8>)
    ensures result == if stop { Err(1) } else if index >= record.values@.len() { Err(2) } else { Ok(&record.values) },
{ record.query(stop, index) }
}
"#.to_owned(), 1);
}

#[test]
fn unsupported_effects_and_loops_remain_rejected() {
    for body in [
        "let mut value = self.value; value += 1; return value;",
        "while self.value > 0 {} return 0;",
    ] {
        let source = format!(
            "verus! {{ pub struct Reading {{ pub value: u8 }} impl Reading {{ #[verifier::infer] fn query(&self) -> u8 {{ {body} }} }} }}"
        );
        let errors =
            verify_one_file("unsupported_record_return", source, &["vstd", "--no-cheating"])
                .unwrap_err();
        assert!(
            errors.errors.iter().any(|error| error.message.contains("Verus Infer unsupported")),
            "{:?}",
            errors
        );
    }
}
