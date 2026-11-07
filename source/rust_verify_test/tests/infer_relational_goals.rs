#![feature(rustc_private)]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const CODEC: &str = r#"
use vstd::prelude::*;
pub struct Word { pub payload: u32, pub tag: u8 }
impl Word {
    #[verifier::infer]
    pub fn encode(self) -> [u8; 8] {
        let mut word = 0xffff & u64::from(self.payload);
        word |= (15 & u64::from(self.tag)) << 60;
        word.to_le_bytes()
    }
    #[verifier::infer]
    pub fn decode(bytes: [u8; 8]) -> Self {
        let word = u64::from_le_bytes(bytes);
        let tag = (word >> 60) as u8;
        let payload = (word & (0xffff >> tag)) as u32;
        Self { payload, tag }
    }
}
fn roundtrip(payload: u32, tag: u8) -> (result: Word)
    requires tag <= 12, payload <= (0xffffu32 >> tag),
    ensures result.payload == payload, result.tag == tag,
{
    Word::decode(Word { payload, tag }.encode())
}
"#;

const ARRAY_PROJECTIONS: &str = r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn pair(value: u8) -> [u8; 2] { [value & 15, value & 7] }
fn caller(value: u8) {
    let result = pair(value);
    assert(result[0] < 16);
    assert(result[1] < 8);
}
}
"#;

#[test]
fn canonical_literal_array_projections_compose_cold_warm_and_imported() {
    let crates = InferCrates::new();
    let options = &["--no-cheating", "--infer-goals", "--output-json"];
    for phase in ["cold", "warm"] {
        let run = crates.build_source("array_provider", ARRAY_PROJECTIONS, &[], options, true);
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
        assert!(crates.vir("array_provider").exists());
    }
    let client = r#"
use vstd::prelude::*;
verus! {
fn caller(value: u8) {
    let result = array_provider::pair(value);
    assert(result[0] < 16);
    assert(result[1] < 8);
}
}
"#;
    let run = crates.build_source("array_client", client, &["array_provider"], options, true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    for (goal, diagnostic) in [
        (
            "fn bad(value: u8) { let result = pair(value); assert(result[1] < 4); }",
            "bitvector assertion not satisfied",
        ),
        (
            "fn bad(value: u8) { let result = pair(value); let beyond = result[2]; }",
            "precondition not met: index in bounds for this access",
        ),
    ] {
        let failed = crates.build_source(
            "array_provider",
            &format!("{ARRAY_PROJECTIONS}\nverus! {{ {goal} }}"),
            &[],
            options,
            true,
        );
        assert!(!failed.status.success());
        assert_exact_report(&json_inference(&failed), 1);
        assert!(!crates.vir("array_provider").exists());
        assert!(
            String::from_utf8_lossy(&failed.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&failed.stderr)
        );
        let seed = crates.build_source("array_provider", ARRAY_PROJECTIONS, &[], options, true);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    }
}

#[test]
fn relational_cast_shift_and_byte_cancellation_is_checked_cold_and_warm() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let run = crates.build(
            "relational_roundtrip",
            CODEC,
            &[],
            &["--no-cheating", "--infer-goals", "--output-json"],
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 2);
    }
}

#[test]
fn false_relational_goal_removes_a_previously_verified_export() {
    let crates = InferCrates::new();
    let options = &["--no-cheating", "--infer-goals", "--output-json"];
    let positive = crates.build("relational_mutant", CODEC, &[], options);
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    let mutant = CODEC.replace("word >> 60", "word >> 61");
    let negative = crates.build("relational_mutant", &mutant, &[], options);
    assert!(!negative.status.success());
    assert!(!crates.vir("relational_mutant").exists());
    assert_exact_report(&json_inference(&negative), 2);
    assert!(
        String::from_utf8_lossy(&negative.stderr).contains("bitvector assertion not satisfied")
    );
}

#[test]
fn cancellation_does_not_invent_the_relational_domain() {
    let crates = InferCrates::new();
    let source = CODEC.replace("payload <= (0xffffu32 >> tag)", "payload <= 0xffff");
    let negative = crates.build(
        "relational_missing_domain",
        &source,
        &[],
        &["--no-cheating", "--infer-goals", "--output-json"],
    );
    assert!(!negative.status.success());
    assert!(!crates.vir("relational_missing_domain").exists());
    assert_exact_report(&json_inference(&negative), 2);
}

#[test]
fn imported_byte_codec_composes_with_unaided_relational_callers() {
    let crates = InferCrates::new();
    let options = &["--no-cheating", "--infer-goals", "--output-json"];
    let provider = crates.build("relational_provider", CODEC, &[], options);
    assert!(provider.status.success(), "{}", String::from_utf8_lossy(&provider.stderr));
    let client = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn relay(bytes:[u8;8])->relational_provider::Word {
            relational_provider::Word::decode(bytes)
        }
        fn caller(payload:u32,tag:u8)->(result:relational_provider::Word)
            requires tag <= 12, payload <= (0xffffu32 >> tag),
            ensures result.payload == payload, result.tag == tag,
        {relay(relational_provider::Word{payload,tag}.encode())}
    "#;
    let run = crates.build("relational_client", client, &["relational_provider"], options);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
}

#[test]
fn canonical_array_equality_preserves_only_checked_cancellation_proposals() {
    let crates = InferCrates::new();
    let source = format!(
        r#"{CODEC}
        fn injectivity(payload:u32,tag:u8,other_payload:u32,other_tag:u8)
            requires tag <= 12, payload <= (0xffffu32 >> tag),
                other_tag <= 12, other_payload <= (0xffffu32 >> other_tag),
        {{
            let first = Word{{payload,tag}}.encode();
            let second = Word{{payload:other_payload,tag:other_tag}}.encode();
            if first == second {{
                let decoded = Word::decode(first);
                assert(decoded.payload == payload && decoded.tag == tag);
                assert(decoded.payload == other_payload && decoded.tag == other_tag);
                assert(payload == other_payload && tag == other_tag);
            }}
        }}
        "#
    );
    let run = crates.build(
        "relational_injectivity",
        &source,
        &[],
        &["--no-cheating", "--infer-goals", "--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 2);
}

const MULTIPLIER: &str = r#"
use vstd::prelude::*;
pub struct Factor { pub order: u8 }
impl Factor {
    #[verifier::infer]
    pub fn expand(&self, units:u32)->u64 {
        let count = 1u64 << self.order;
        count * u64::from(units)
    }
}
"#;

#[test]
fn checked_shift_product_bounds_cover_all_u32_sizes_and_large_valid_orders() {
    let crates = InferCrates::new();
    let source = format!(
        r#"{MULTIPLIER}
        fn bounded(order:u8,units:u32)->(result:u64)
            requires order <= 17,
            ensures result == (1u64 << order) as int * units as int,
                result <= 562949953290240,
                result <= 131072 * units as int,
        {{Factor{{order}}.expand(units)}}
        fn zero()->(result:u64) ensures result == 0,
        {{Factor{{order:63}}.expand(0)}}
        fn one()->(result:u64) ensures result == 9223372036854775808,
        {{Factor{{order:63}}.expand(1)}}
        "#
    );
    let run = crates.build(
        "relational_shift_product",
        &source,
        &[],
        &["--no-cheating", "--infer-goals", "--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
}

#[test]
fn eager_invalid_shifts_and_multiplication_overflow_are_not_hidden_by_assistance() {
    let crates = InferCrates::new();
    for (order, units) in [(64, 0), (255, 0), (63, 2)] {
        let source =
            format!("{MULTIPLIER} fn rejected()->u64 {{Factor{{order:{order}}}.expand({units})}}");
        let run = crates.build(
            "relational_invalid_product",
            &source,
            &[],
            &["--no-cheating", "--infer-goals", "--output-json"],
        );
        assert!(!run.status.success());
        assert!(!crates.vir("relational_invalid_product").exists());
        assert_exact_report(&json_inference(&run), 1);
        assert!(String::from_utf8_lossy(&run.stderr).contains("bitvector assertion not satisfied"));
    }
}
