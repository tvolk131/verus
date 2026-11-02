#![feature(rustc_private)]
#[allow(unused_imports)]
use rust_verify_test_macros::*;
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const PROVIDER: &str = r#"
use vstd::prelude::*;
#[verifier::infer]
pub fn narrow(number: u64) -> u32 { u32::try_from(number).unwrap() }
#[verifier::infer]
pub fn signed(number: i64) -> u8 { u8::try_from(number).unwrap() }
#[verifier::infer]
pub fn increment(number: u64) -> u32 { u32::try_from(number + 1).unwrap() }
#[verifier::infer]
pub fn branch(number: u64, selected: bool) -> u32 {
    if selected { u32::try_from(number).unwrap() } else { 0 }
}
fn obligations(number: u64) -> (result: u32)
    requires number <= u32::MAX,
    ensures result == number,
{ narrow(number) }
fn signed_obligations(number: i64) -> (result: u8)
    requires 0 <= number <= u8::MAX,
    ensures result == number,
{ signed(number) }
fn increment_obligations(number: u64) -> (result: u32)
    requires number < u32::MAX,
    ensures result == number + 1,
{ increment(number) }
fn inactive(number: u64) -> (result: u32)
    ensures result == 0,
{ branch(number, false) }
"#;

#[test]
fn conversion_unwrap_exact_cold_warm_and_imports() {
    let crates = InferCrates::new();
    for _phase in ["cold", "warm"] {
        let run =
            crates.build("conversion_provider", PROVIDER, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert!(crates.vir("conversion_provider").exists());
        assert_exact_report(&json_inference(&run), 4);
        crates.edit_metadata("conversion_provider", |metadata| {
            let models = metadata["trusted_models"].to_string();
            assert!(models.contains("try_from"), "{}", models);
            assert!(models.contains("unwrap"), "{}", models);
        });
    }
    for _phase in ["cold", "warm"] {
        let run = crates.build(
            "conversion_client",
            r#"
            use vstd::prelude::*;
            #[verifier::infer]
            pub fn relay(number: u64) -> u32 { conversion_provider::narrow(number) }
            fn caller(number: u64) -> (result: u32)
                requires number <= u32::MAX,
                ensures result == number,
            { relay(number) }
        "#,
            &["conversion_provider"],
            &["--no-cheating", "--output-json"],
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
    }
}

#[test]
fn conversion_unwrap_preserves_panic_and_arithmetic_requirements() {
    let crates = InferCrates::new();
    for caller in [
        "fn bad(number:u64)->u32 {narrow(number)}",
        "fn bad()->u32 {narrow(4294967296)}",
        "fn bad()->u8 {signed(-1)}",
        "fn bad()->u32 {increment(18446744073709551615)}",
        "fn bad(number:u64)->u32 {branch(number,true)}",
    ] {
        let run = crates.build(
            "conversion_panic",
            &format!("{PROVIDER}{caller}"),
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        assert!(!crates.vir("conversion_panic").exists());
        assert_exact_report(&json_inference(&run), 4);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn conversion_false_goal_removes_stale_export() {
    let crates = InferCrates::new();
    let positive =
        crates.build("conversion_negative", PROVIDER, &[], &["--no-cheating", "--output-json"]);
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    assert!(crates.vir("conversion_negative").exists());
    let negative = crates.build(
        "conversion_negative",
        &format!(
            r#"{PROVIDER}
        fn wrong(number:u64)->(result:u32)
            requires number <= u32::MAX,
            ensures result != number,
        {{narrow(number)}}
    "#
        ),
        &[],
        &["--no-cheating", "--output-json"],
    );
    assert!(!negative.status.success());
    assert!(!crates.vir("conversion_negative").exists());
    assert_exact_report(&json_inference(&negative), 4);
    assert!(String::from_utf8_lossy(&negative.stderr).contains("postcondition not satisfied"));
}

#[test]
fn application_try_from_and_unwrap_lookalikes_are_not_models() {
    let crates = InferCrates::new();
    for source in [
        r#"struct Number; impl Number {fn try_from(number:u64)->u32 {number as u32}}
            #[verifier::infer] fn wrong(number:u64)->u32 {Number::try_from(number)}"#,
        r#"struct Number {number:u64} impl Number {fn unwrap(self)->u32 {self.number as u32}}
            #[verifier::infer] fn wrong(number:u64)->u32 {Number{number}.unwrap()}"#,
    ] {
        let run =
            crates.build("conversion_lookalike", source, &[], &["--no-cheating", "--output-json"]);
        assert!(!run.status.success());
        assert!(!crates.vir("conversion_lookalike").exists());
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("unsupported"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn poisoned_conversion_domains_receive_fresh_checks() {
    for (condition, failed_check) in [(true, "soundness"), (false, "maximality")] {
        let crates = InferCrates::new();
        let source = "use vstd::prelude::*; #[verifier::infer] pub fn narrow(number:u64)->u32 {u32::try_from(number).unwrap()}";
        let positive =
            crates.build("conversion_poison", source, &[], &["--no-cheating", "--output-json"]);
        assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
        for entry in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = entry.unwrap().path();
            let mut candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            candidate["pre"] = serde_json::json!({"Bool": condition});
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
        let negative =
            crates.build("conversion_poison", source, &[], &["--no-cheating", "--output-json"]);
        assert!(!negative.status.success());
        assert!(!crates.vir("conversion_poison").exists());
        let report = json_inference(&negative);
        let function = &report["functions"][0];
        assert_eq!(function["metrics"]["cache"], "hit", "{function}");
        assert_eq!(function["proofs"][failed_check]["status"], "not_proved", "{function}");
        assert_eq!(function["available_to_same_crate_callers"], false);
        assert_eq!(function["exported"], false);
    }
}
