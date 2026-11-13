#![feature(rustc_private)]

#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const PROVIDER: &str = r#"
use vstd::prelude::*;
#[verifier::infer]
pub fn append(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
    let mut result = prefix.clone();
    result.extend_from_slice(key);
    result
}
"#;

fn run_source(crates: &InferCrates, name: &str, source: &str) -> std::process::Output {
    crates.build(name, source, &[], &["--no-cheating", "--output-json"])
}

#[test]
fn exact_full_view_cold_warm_and_imported_composition() {
    let crates = InferCrates::new();
    for _phase in ["cold", "warm"] {
        let source = format!(
            r#"{PROVIDER}
fn goal(prefix: &Vec<u8>, key: &[u8]) -> (result: Vec<u8>)
    ensures result@ == prefix@ + key@,
{{ append(prefix, key) }}"#
        );
        let run = run_source(&crates, "append_provider", &source);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_exact_report(&report, 1);
        assert_eq!(report["functions"][0]["metrics"]["cache"], "miss");
        assert_eq!(report["functions"][0]["metrics"]["cache_policy"], "regenerated_state");
        assert!(report["functions"][0]["metrics"]["solver_micros"].as_u64().unwrap() > 0);
        assert_eq!(report["interface_written"], true);
    }
    let run = crates.build(
        "append_consumer",
        r#"
use vstd::prelude::*;
#[verifier::infer]
pub fn composed(prefix: &Vec<u8>, first: &[u8], second: &[u8]) -> Vec<u8> {
    let left = append_provider::append(prefix, first);
    append_provider::append(&left, second)
}
fn goal(prefix: &Vec<u8>, first: &[u8], second: &[u8]) -> (result: Vec<u8>)
    ensures result@ == (prefix@ + first@) + second@,
{ composed(prefix, first, second) }
"#,
        &["append_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
}

#[test]
fn order_and_duplicate_mutants_are_exact_but_not_concatenation() {
    for mutation in [
        "let mut result = Vec::new(); result.extend_from_slice(key); result",
        "let mut result = prefix.clone(); result.extend_from_slice(key); result.extend_from_slice(key); result",
    ] {
        let crates = InferCrates::new();
        let original =
            "let mut result = prefix.clone();\n    result.extend_from_slice(key);\n    result";
        let source = PROVIDER.replace(original, mutation);
        let run = run_source(&crates, "mutant", &source);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
        let with_goal = format!(
            r#"{source}
fn goal(prefix: &Vec<u8>, key: &[u8]) -> (result: Vec<u8>)
    ensures result@ == prefix@ + key@,
{{ append(prefix, key) }}"#
        );
        let run = run_source(&crates, "mutant", &with_goal);
        assert!(!run.status.success());
        let report = json_inference(&run);
        assert_eq!(report["status"], "caller_proof_failed");
        assert_exact_report(&report, 1);
        assert_eq!(report["interface_written"], false);
        assert!(!crates.vir("mutant").exists());
    }
}

#[test]
fn unsupported_element_effects_and_nonlocal_receivers_decline() {
    for source in [
        r#"use vstd::prelude::*;
#[verifier::infer] fn generic<T: Clone>(prefix: &Vec<T>, key: &[T]) -> Vec<T> {
 let mut result = prefix.clone(); result.extend_from_slice(key); result
}"#,
        r#"use vstd::prelude::*;
#[verifier::infer] fn wrong(prefix: &Vec<u16>, key: &[u16]) -> Vec<u16> {
 let mut result = prefix.clone(); result.extend_from_slice(key); result
}"#,
        r#"use vstd::prelude::*;
struct Record { values: Vec<u8> }
#[verifier::infer] fn nonlocal(record: &mut Record, key: &[u8]) {
 record.values.extend_from_slice(key);
}"#,
        r#"use vstd::prelude::*;
fn effect(key: &[u8]) -> &[u8] { key }
#[verifier::infer] fn side_effect(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
 let mut result = prefix.clone(); result.extend_from_slice(effect(key)); result
}"#,
    ] {
        let crates = InferCrates::new();
        let run = run_source(&crates, "unsupported_append", source);
        assert!(!run.status.success());
        let report = json_inference(&run);
        assert_eq!(report["status"], "unsupported", "{report}");
        assert_eq!(report["interface_written"], false);
        assert!(!crates.vir("unsupported_append").exists());
    }
}

#[test]
fn safety_and_false_goals_remove_seeded_exports() {
    let crates = InferCrates::new();
    for bad in [
        format!(
            "{PROVIDER} fn bad(prefix:&Vec<u8>,key:&[u8]) {{ let out=append(prefix,key); let value=out[0]; }}"
        ),
        format!(
            "{PROVIDER} fn bad(prefix:&Vec<u8>,key:&[u8])->(out:Vec<u8>) ensures false, {{append(prefix,key)}}"
        ),
    ] {
        let good = run_source(&crates, "failed_append", PROVIDER);
        assert!(good.status.success(), "{}", String::from_utf8_lossy(&good.stderr));
        assert!(crates.vir("failed_append").exists());
        let run = run_source(&crates, "failed_append", &bad);
        assert!(!run.status.success());
        let report = json_inference(&run);
        assert_eq!(report["status"], "caller_proof_failed");
        assert_exact_report(&report, 1);
        assert_eq!(report["interface_written"], false);
        assert!(!crates.vir("failed_append").exists());
    }
}

#[test]
fn slice_order_is_exact_and_reversal_fails_the_authored_goal() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
#[verifier::infer] pub fn reversed(first: &[u8], second: &[u8]) -> Vec<u8> {
 let mut result=Vec::new(); result.extend_from_slice(second); result.extend_from_slice(first); result
}
fn actual(first:&[u8],second:&[u8])->(result:Vec<u8>) ensures result@==second@+first@,
{reversed(first,second)}"#;
    let run = run_source(&crates, "ordered_append", source);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
    let changed = source.replace("result@==second@+first@", "result@==first@+second@");
    let run = run_source(&crates, "ordered_append", &changed);
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
    assert_exact_report(&json_inference(&run), 1);
    assert!(!crates.vir("ordered_append").exists());
}

#[test]
fn canonical_byte_range_composes_without_generic_call_relaxation() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*; use std::ops::Range;
struct Prefix<Inner> { inner:Inner, prefix:Vec<u8> }
impl<Inner> Prefix<Inner> {
 #[verifier::infer] fn key(&self,key:&[u8])->Vec<u8> {
  let mut result=self.prefix.clone();result.extend_from_slice(key);result
 }
 #[verifier::infer] fn bounds(&self,bounds:Range<&[u8]>)->Range<Vec<u8>> {
  Range {start:self.key(bounds.start),end:self.key(bounds.end)}
 }
}

fn goal<Inner>(prefix:&Prefix<Inner>,bounds:Range<&[u8]>)->(result:Range<Vec<u8>>)
 ensures result.start@==prefix.prefix@+bounds.start@,result.end@==prefix.prefix@+bounds.end@,
{prefix.bounds(bounds)}"#;
    let run = run_source(&crates, "byte_range", source);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 2);
    for source in [
        r#"use vstd::prelude::*; use std::ops::Range;
#[verifier::infer] fn wrong(bounds:Range<&Vec<u16>>)->Range<Vec<u16>> {
 Range {start:bounds.start.clone(),end:bounds.end.clone()}
}"#,
        r#"use vstd::prelude::*; use std::ops::Range;
#[verifier::infer] fn wrong(bounds:&mut Range<Vec<u8>>,key:&[u8]) {
 bounds.start.extend_from_slice(key);
}"#,
    ] {
        let run = run_source(&crates, "unsupported_range", source);
        assert!(!run.status.success());
        assert_eq!(json_inference(&run)["status"], "unsupported");
    }
}

const BORROWED_PROVIDER: &str = r#"
use vstd::prelude::*;
struct Prefix<Inner> { inner: Inner, prefix: Vec<u8> }
impl<Inner> Prefix<Inner> {
 #[verifier::infer] fn field(&self, key: &[u8]) -> Vec<u8> {
  let mut result = Vec::new();
  result.extend_from_slice(&self.prefix);
  result.extend_from_slice(key);
  result
 }
}
#[verifier::infer]
pub fn append(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
 let mut result = Vec::new();
 let local_vec = prefix.clone();
 result.extend_from_slice(&local_vec);
 result.extend_from_slice(key);
 result
}
fn field_goal<Inner>(prefix: &Prefix<Inner>, key: &[u8]) -> (result: Vec<u8>)
 ensures result@ == prefix.prefix@ + key@,
{ prefix.field(key) }
"#;

#[test]
fn borrowed_field_and_local_bytes_are_exact_cold_warm_and_imported() {
    let crates = InferCrates::new();
    for _phase in ["cold", "warm"] {
        let run = run_source(&crates, "borrowed_provider", BORROWED_PROVIDER);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_exact_report(&report, 2);
        for function in report["functions"].as_array().unwrap() {
            assert_eq!(function["metrics"]["cache"], "miss");
            assert_eq!(function["metrics"]["cache_policy"], "regenerated_state");
            assert!(function["metrics"]["solver_micros"].as_u64().unwrap() > 0);
        }
    }
    let run = crates.build(
        "borrowed_consumer",
        r#"
use vstd::prelude::*;
#[verifier::infer] pub fn composed(prefix: &Vec<u8>, first: &[u8], second: &[u8]) -> Vec<u8> {
 let local_vec = borrowed_provider::append(prefix, first);
 let mut result = Vec::new();
 result.extend_from_slice(&local_vec);
 result.extend_from_slice(second);
 result
}
fn goal(prefix: &Vec<u8>, first: &[u8], second: &[u8]) -> (result: Vec<u8>)
 ensures result@ == (prefix@ + first@) + second@,
{ composed(prefix, first, second) }
"#,
        &["borrowed_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
}

#[test]
fn borrowed_semantic_wrong_goals_and_safety_remove_real_exports() {
    let crates = InferCrates::new();
    for goal in ["result@ == key@ + prefix@", "false"] {
        let good = run_source(&crates, "borrowed_failed", BORROWED_PROVIDER);
        assert!(good.status.success(), "{}", String::from_utf8_lossy(&good.stderr));
        assert!(crates.vir("borrowed_failed").exists());
        let source = format!(
            "{BORROWED_PROVIDER}\nfn wrong(prefix:&Vec<u8>,key:&[u8])->(result:Vec<u8>) ensures {goal}, {{ append(prefix,key) }}"
        );
        let bad = run_source(&crates, "borrowed_failed", &source);
        assert!(!bad.status.success());
        let report = json_inference(&bad);
        assert_exact_report(&report, 2);
        assert_eq!(report["status"], "caller_proof_failed");
        assert_eq!(report["interface_written"], false);
        assert!(!crates.vir("borrowed_failed").exists());
    }
    let good = run_source(&crates, "borrowed_failed", BORROWED_PROVIDER);
    assert!(good.status.success());
    let source = format!(
        "{BORROWED_PROVIDER}\nfn unsafe_empty(prefix:&Vec<u8>,key:&[u8]) {{ let result=append(prefix,key); let byte=result[0]; }}"
    );
    let bad = run_source(&crates, "borrowed_failed", &source);
    assert!(!bad.status.success());
    assert_eq!(json_inference(&bad)["status"], "caller_proof_failed");
    assert!(!crates.vir("borrowed_failed").exists());
}

#[test]
fn borrowed_effects_temporaries_mutable_aliases_and_other_elements_decline() {
    let crates = InferCrates::new();
    for source in [
        r#"use vstd::prelude::*;
fn effect(prefix: &Vec<u8>) -> &Vec<u8> { prefix }
#[verifier::infer] pub fn append(prefix:&Vec<u8>,key:&[u8])->Vec<u8> {
 let mut result=Vec::new(); result.extend_from_slice(effect(prefix)); result
}"#,
        r#"use vstd::prelude::*;
#[verifier::infer] pub fn append(prefix:&Vec<u8>,key:&[u8])->Vec<u8> {
 let mut result=Vec::new(); result.extend_from_slice(&prefix.clone()); result
}"#,
        r#"use vstd::prelude::*;
#[verifier::infer] pub fn append(prefix:&Vec<u8>,key:&[u8])->Vec<u8> {
 let mut result=prefix.clone(); result.extend_from_slice(&result.clone()); result
}"#,
        r#"use vstd::prelude::*;
#[verifier::infer] pub fn append(prefix:&Vec<u8>,key:&[u8])->Vec<u8> {
 let mut result=Vec::new(); let alias=&result; result.extend_from_slice(alias); result
}"#,
        r#"use vstd::prelude::*;
#[verifier::infer] pub fn append(prefix:&mut Vec<u8>,key:&[u8])->Vec<u8> {
 let mut result=Vec::new(); result.extend_from_slice(prefix); result
}"#,
        r#"use vstd::prelude::*;
#[verifier::infer] pub fn append(prefix:&Vec<u16>,key:&[u16])->Vec<u16> {
 let mut result=Vec::new(); result.extend_from_slice(prefix); result
}"#,
    ] {
        let good = run_source(&crates, "borrowed_declined", BORROWED_PROVIDER);
        assert!(good.status.success(), "{}", String::from_utf8_lossy(&good.stderr));
        assert!(crates.vir("borrowed_declined").exists());
        let run = run_source(&crates, "borrowed_declined", source);
        assert!(!run.status.success(), "{}", String::from_utf8_lossy(&run.stdout));
        let report = json_inference(&run);
        assert_eq!(report["status"], "unsupported", "{report}");
        assert_eq!(report["interface_written"], false);
        assert!(!crates.vir("borrowed_declined").exists());
    }
}
