#![feature(rustc_private)]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const SOURCE: &str = include_str!("infer_cases/redb_page_relations.rs");
const OPTIONS: &[&str] = &["--no-cheating", "--infer-goals", "--output-json"];

#[test]
fn original_page_relations_compose_cold_warm_and_imported() {
    let crates = InferCrates::new();
    let source = SOURCE.replace("pub(crate)", "pub");
    for phase in ["cold", "warm"] {
        let run = crates.build_source("provider", &source, &[], OPTIONS, true);
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 2);
    }
    let (_, callers) = source.split_once("fn strict_start").unwrap();
    let client =
        format!("use vstd::prelude::*; use std::ops::Range; verus! {{ fn strict_start{callers}")
            .replace("PageNumber{", "provider::PageNumber{");
    let run = crates.build_source("client", &client, &["provider"], OPTIONS, true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
}

#[test]
fn false_polynomial_fails_its_auxiliary_theorem_and_removes_seeded_export() {
    let crates = InferCrates::new();
    let run = crates.build_source("provider", SOURCE, &[], OPTIONS, true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let wrong = SOURCE.replacen(
        "ensures result.0.start<result.1.start",
        "ensures result.0.start>result.1.start",
        1,
    );
    let run = crates.build_source("provider", &wrong, &[], OPTIONS, true);
    assert!(!run.status.success());
    assert_exact_report(&json_inference(&run), 2);
    assert!(!crates.vir("provider").exists());
    assert!(String::from_utf8_lossy(&run.stderr).contains("polynomial proposal not satisfied"));
}

#[test]
fn mixed_order_nonoverlap_and_zero_start_claims_fail_semantically() {
    let crates = InferCrates::new();
    for (goal, call) in [
        ("result.0.end<=result.1.start || result.1.end<=result.0.start", "mixed_witness()"),
        ("result.0.start<result.1.start", "zero_witness()"),
        ("result.0.start!=result.1.start", "zero_witness()"),
    ] {
        let seed = crates.build_source("provider", SOURCE, &[], OPTIONS, true);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let source = format!(
            "{SOURCE}\nverus! {{ fn wrong()->(result:(std::ops::Range<u64>,std::ops::Range<u64>)) ensures {goal}, {{{call}}} }}"
        );
        let run = crates.build_source("provider", &source, &[], OPTIONS, true);
        assert!(!run.status.success());
        assert_exact_report(&json_inference(&run), 2);
        assert!(!crates.vir("provider").exists());
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains("postcondition not satisfied"), "{}", stderr);
        assert!(!stderr.contains("polynomial proposal not satisfied"), "{}", stderr);
        assert!(!stderr.contains("precondition not satisfied"), "{}", stderr);
    }
}
