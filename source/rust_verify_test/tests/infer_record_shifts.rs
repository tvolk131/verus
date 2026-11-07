#![feature(rustc_private)]
#[allow(unused_imports)]
use rust_verify_test_macros::*;
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const METHOD: &str = r#"
use vstd::prelude::*;
pub struct Page {pub order:u8,pub marker:u32}
impl Page {
    #[verifier::infer]
    pub fn size(&self, base:u32)->u64 {
        let pages=1u64 << self.order;
        pages * u64::from(base)
    }
}
"#;
const CALLERS: &str = r#"
fn full_domain(order:u8,base:u32)->(result:u64)
    requires order<64, (1u64 << order) as int * base as int <= u64::MAX,
    ensures result == (1u64 << order) as int * base as int,
{Page{order,marker:0}.size(base)}
fn zero()->(result:u64) ensures result==0,
{Page{order:63,marker:0}.size(0)}
fn largest()->(result:u64) ensures result==9223372036854775808,
{
    proof {assert((1u64 << 63) == 9223372036854775808u64) by(bit_vector);}
    Page{order:63,marker:0}.size(1)
}
"#;

#[test]
fn readonly_variable_shift_is_exact_cold_warm_and_imported() {
    let crates = InferCrates::new();
    for _phase in ["cold", "warm"] {
        let run = crates.build(
            "shift_provider",
            &(METHOD.to_owned() + CALLERS),
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
    }
    let client = crates.build(
        "shift_client",
        r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn relay(page:&shift_provider::Page,base:u32)->u64 {page.size(base)}
        fn caller(order:u8,base:u32)->(result:u64)
            requires order<64, (1u64 << order) as int * base as int <= u64::MAX,
            ensures result==(1u64 << order) as int * base as int,
        {relay(&shift_provider::Page{order,marker:0},base)}
    "#,
        &["shift_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(client.status.success(), "{}", String::from_utf8_lossy(&client.stderr));
    assert_exact_report(&json_inference(&client), 1);
}

#[test]
fn shift_width_is_eager_and_multiplication_remains_checked() {
    let crates = InferCrates::new();
    let positive = crates.build("shift_positive", METHOD, &[], &["--no-cheating", "--output-json"]);
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    let seed = std::fs::read(crates.vir("shift_positive")).unwrap();
    for (name, caller) in [
        ("shift64_zero", "fn bad()->u64{Page{order:64,marker:0}.size(0)}"),
        ("shift255_zero", "fn bad()->u64{Page{order:255,marker:0}.size(0)}"),
        (
            "shift_invalid_domain",
            "fn bad(order:u8)->u64 requires order>=64, {Page{order,marker:0}.size(0)}",
        ),
        (
            "shift_overflow",
            "fn bad()->u64{proof{assert((1u64 << 63)==9223372036854775808u64) by(bit_vector);} Page{order:63,marker:0}.size(2)}",
        ),
    ] {
        std::fs::write(crates.vir(name), &seed).unwrap();
        let run = crates.build(
            name,
            &(METHOD.to_owned() + caller),
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        assert_exact_report(&json_inference(&run), 1);
        assert!(!crates.vir(name).exists());
        assert!(String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"));
    }
    assert!(crates.vir("shift_positive").exists());
}

#[test]
fn false_shift_equation_fails_semantically_and_removes_seeded_export() {
    let crates = InferCrates::new();
    let source = METHOD.to_owned()
        + "fn wrong()->(result:u64) ensures result==1, {Page{order:20,marker:0}.size(0)}";
    std::fs::write(crates.vir("shift_false"), b"seeded stale export").unwrap();
    let run = crates.build("shift_false", &source, &[], &["--no-cheating", "--output-json"]);
    assert!(!run.status.success());
    assert_exact_report(&json_inference(&run), 1);
    assert!(!crates.vir("shift_false").exists());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
}

#[test]
fn variable_shift_does_not_admit_control_flow_or_mutable_records() {
    let crates = InferCrates::new();
    for (name, receiver, body) in [
        ("shift_branch", "&self", "if selected {1u64 << self.order} else {0}"),
        ("shift_mutable", "&mut self", "self.order=1; 1u64 << self.order"),
        ("shift_local_effect", "&self", "let mut amount=self.order; 1u64 << {amount+=1; amount}"),
    ] {
        let source = format!(
            "pub struct Page{{pub order:u8,pub marker:u32}} impl Page{{#[verifier::infer] pub fn size({receiver},selected:bool)->u64{{{body}}}}}"
        );
        let run = crates.build(name, &source, &[], &["--no-cheating", "--output-json"]);
        assert!(!run.status.success(), "{}", name);
        assert!(!crates.vir(name).exists());
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("Verus Infer"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn signed_shift_amount_retains_lower_bound() {
    let crates = InferCrates::new();
    let source = r#"
        pub struct Page{pub order:i8,pub marker:u32}
        impl Page{#[verifier::infer] pub fn size(&self)->u64{1u64 << self.order}}
        fn valid(order:i8)->(result:u64) requires 0<=order<64,
            ensures result==1u64 << order, {Page{order,marker:0}.size()}
    "#;
    let positive = crates.build("signed_shift", source, &[], &["--no-cheating", "--output-json"]);
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    assert_exact_report(&json_inference(&positive), 1);
    let negative = crates.build(
        "signed_shift_negative",
        &(source.to_owned() + "fn bad()->u64{Page{order:-1,marker:0}.size()}"),
        &[],
        &["--no-cheating", "--output-json"],
    );
    assert!(!negative.status.success());
    assert_exact_report(&json_inference(&negative), 1);
    assert!(String::from_utf8_lossy(&negative.stderr).contains("precondition not satisfied"));
}
