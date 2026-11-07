#![feature(rustc_private)]
#[allow(unused_imports)]
use rust_verify_test_macros::*;
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const RECORD: &str = r#"
use vstd::prelude::*;
pub struct Guarded {pub limit:u64,pub marker:bool}
impl Guarded {
    #[verifier::infer]
    pub fn range(&self,start:u64,length:u64)->std::ops::Range<u64> {
        debug_assert!(start < self.limit);
        let end=start+length;
        start..end
    }
    #[verifier::infer]
    pub fn scalar(&self,start:u64)->u64 {
        assert!(start < self.limit);
        start+1
    }
}
"#;

#[test]
fn literal_panics_compose_with_scalar_and_record_profiles() {
    let crates = InferCrates::new();
    let source = RECORD.to_owned()
        + r#"
        #[verifier::infer]
        pub fn scalar(input:u8)->u8 {assert!(input<200);input+1}
        #[verifier::infer]
        pub fn divergent(input:u8)->u8 {if input<200 {input+1} else {panic!("literal")}}
        fn caller(input:u8,start:u64,limit:u64,length:u64)
            requires input<200,start<limit,start<u64::MAX,start as int+length as int<=u64::MAX,
        {
            let guarded=Guarded{limit,marker:false};
            let range=guarded.range(start,length);
            assert(range.start==start);
            assert(range.end as int-range.start as int==length);
            let record_value=guarded.scalar(start);
            let scalar_value=scalar(input);
            let divergent_value=divergent(input);
            assert(record_value==start+1);
            assert(scalar_value==input+1);
            assert(divergent_value==input+1);
        }
        "#;
    let run = crates.build("literal_profiles", &source, &[], &["--no-cheating", "--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 4);
}

#[test]
fn canonical_range_returns_are_exact_cold_warm_and_imported() {
    let crates = InferCrates::new();
    for _phase in ["cold", "warm"] {
        let run = crates.build("range_provider", RECORD, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 2);
    }
    let run = crates.build(
        "range_client",
        r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn relay(guarded:&range_provider::Guarded,start:u64,length:u64)->std::ops::Range<u64> {
            guarded.range(start,length)
        }
        fn caller(start:u64,limit:u64,length:u64)
            requires start<limit,start as int+length as int<=u64::MAX,
        {
            let range=relay(&range_provider::Guarded{limit,marker:false},start,length);
            assert(range.end as int-range.start as int==length);
        }
        "#,
        &["range_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
}

#[test]
fn guard_overflow_and_false_containment_remain_ordinary_failures() {
    let crates = InferCrates::new();
    for (name, caller, diagnostic) in [
        (
            "guard",
            "fn rejected(){Guarded{limit:0,marker:false}.range(0,0);}",
            "precondition not satisfied",
        ),
        (
            "overflow",
            "fn rejected(){Guarded{limit:u64::MAX,marker:false}.range(u64::MAX-1,2);}",
            "precondition not satisfied",
        ),
        (
            "containment",
            "fn rejected(){let result=Guarded{limit:4096,marker:false}.range(4090,4096);assert(result.end<=4096);}",
            "assertion failed",
        ),
    ] {
        std::fs::write(crates.vir(name), b"seeded obsolete export").unwrap();
        let run = crates.build(
            name,
            &(RECORD.to_owned() + caller),
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        assert!(
            String::from_utf8_lossy(&run.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(!crates.vir(name).exists());
    }
}

#[test]
fn dynamic_and_effectful_panic_operands_fail_closed() {
    let crates = InferCrates::new();
    for (name, expression) in [
        ("formatted", "panic!(\"{}\",input+1)"),
        ("dynamic", "core::panicking::panic(if input==0 {\"zero\"} else {\"other\"})"),
    ] {
        let source = format!(
            "use vstd::prelude::*; #[verifier::infer] pub fn rejected(input:u8)->u8 {{{expression}}}"
        );
        let run = crates.build(name, &source, &[], &["--no-cheating", "--output-json"]);
        assert!(!run.status.success());
        assert!(!crates.vir(name).exists());
    }
}
