#![feature(rustc_private)]
#[allow(unused_imports)]
use rust_verify_test_macros::*;
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference, verify_exact};

const CODEC: &str = r#"
use vstd::prelude::*;
pub struct Word { pub number: u64, pub marker: u32 }
impl Word {
    #[verifier::infer]
    pub fn bytes(self) -> [u8; 8] {
        let mut number = self.number;
        number |= u64::from(self.marker) << 32;
        number.to_le_bytes()
    }
}
pub struct Halves { pub low: u32, pub high: u32 }
#[verifier::infer]
pub fn decode(bytes: [u8; 8]) -> Halves {
    let number = u64::from_le_bytes(bytes);
    let low = u32::try_from(number & 0xffff_ffff).unwrap();
    let high = (number >> 32) as u32;
    Halves { low, high }
}
fn caller(number: u64, marker: u32) -> (bytes: [u8; 8])
    ensures bytes@ == vstd::bytes::spec_u64_to_le_bytes(number | ((marker as u64) << 32)),
{ Word { number, marker }.bytes() }
"#;

#[test]
fn transparent_owned_record_array_codec_composition_is_exact() {
    let crates = InferCrates::new();
    for _phase in ["cold", "warm"] {
        let run = crates.build("codec_values", CODEC, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 2);
        assert!(crates.vir("codec_values").exists());
    }
}

#[test]
fn nested_record_codec_blocks_preserve_outer_updates_and_lexical_scope() {
    let crates = InferCrates::new();
    let source = CODEC.replace(
        "number |= u64::from(self.marker) << 32;",
        "{ { number |= u64::from(self.marker) << 32; } let mut number = 0u64; number |= 1; }",
    );
    for _phase in ["cold", "warm"] {
        let run =
            crates.build("codec_nested_blocks", &source, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 2);
        assert!(crates.vir("codec_nested_blocks").exists());
    }
}

#[test]
fn record_codec_initializer_blocks_preserve_executed_updates() {
    let crates = InferCrates::new();
    let source = r#"
        use vstd::prelude::*;
        pub struct Word { pub number:u64, pub marker:u32 }
        impl Word {
            #[verifier::infer]
            pub fn bytes(self)->[u8;8] {
                let mut number = self.number;
                let _other = { number |= 1; number };
                number.to_le_bytes()
            }
        }
        fn caller(number:u64, marker:u32)->(bytes:[u8;8])
            ensures bytes@ == vstd::bytes::spec_u64_to_le_bytes(number | 1),
        { Word { number, marker }.bytes() }
    "#;
    let run =
        crates.build("codec_initializer_block", source, &[], &["--no-cheating", "--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
}

#[test]
fn readonly_record_codec_operands_do_not_hide_nested_mutation() {
    let crates = InferCrates::new();
    let source =
        CODEC.replace("number |= u64::from(self.marker) << 32;", "number |= { number |= 1; 2 };");
    let run =
        crates.build("codec_readonly_block", &source, &[], &["--no-cheating", "--output-json"]);
    assert!(!run.status.success());
    assert!(!crates.vir("codec_readonly_block").exists());
    assert!(String::from_utf8_lossy(&run.stderr).contains("side effects inside record values"));
}

#[test]
fn record_byte_contracts_import_and_compose_without_application_summaries() {
    let crates = InferCrates::new();
    let provider = crates.build("codec_provider", CODEC, &[], &["--no-cheating", "--output-json"]);
    assert!(provider.status.success(), "{}", String::from_utf8_lossy(&provider.stderr));
    let client = crates.build(
        "codec_client",
        r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn relay(bytes:[u8;8])->codec_provider::Halves {codec_provider::decode(bytes)}
        fn encoding(number:u64, marker:u32)->(bytes:[u8;8])
            ensures bytes@ == vstd::bytes::spec_u64_to_le_bytes(number | ((marker as u64) << 32)),
        {codec_provider::Word{number,marker}.bytes()}
    "#,
        &["codec_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(client.status.success(), "{}", String::from_utf8_lossy(&client.stderr));
    assert_exact_report(&json_inference(&client), 1);
}

#[test]
fn failed_record_codec_goal_is_not_a_frontend_rejection() {
    let crates = InferCrates::new();
    let positive =
        crates.build("codec_values_negative", CODEC, &[], &["--no-cheating", "--output-json"]);
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    let source = CODEC.replace("bytes@ == vstd::bytes", "bytes@ != vstd::bytes");
    let negative =
        crates.build("codec_values_negative", &source, &[], &["--no-cheating", "--output-json"]);
    assert!(!negative.status.success());
    assert!(!crates.vir("codec_values_negative").exists());
    assert_exact_report(&json_inference(&negative), 2);
    assert!(String::from_utf8_lossy(&negative.stderr).contains("postcondition not satisfied"));
}

#[test]
fn record_codec_invariant_bearing_inputs_remain_rejected() {
    let crates = InferCrates::new();
    let run = crates.build(
        "codec_invariant",
        r#"
        use vstd::prelude::*;
        pub struct Word {number:u64}
        impl Word {
            #[verifier::type_invariant]
            pub closed spec fn valid(&self)->bool {self.number < 100}
            #[verifier::infer]
            pub fn bytes(self)->[u8;8] {self.number.to_le_bytes()}
        }
    "#,
        &[],
        &["--no-cheating", "--output-json"],
    );
    assert!(!run.status.success());
    assert!(!crates.vir("codec_invariant").exists());
    assert!(String::from_utf8_lossy(&run.stderr).contains("unsupported"));
}

#[test]
fn record_array_mutation_does_not_admit_control_flow_or_variable_shifts() {
    let crates = InferCrates::new();
    for body in [
        "let mut number=self.number; if selected {number |= 1;} number.to_le_bytes()",
        "let number=self.number << amount; number.to_le_bytes()",
    ] {
        let source = format!(
            r#"
            use vstd::prelude::*;
            pub struct Word {{pub number:u64, pub marker:u32}}
            impl Word {{
                #[verifier::infer]
                pub fn bytes(self, selected:bool, amount:u32)->[u8;8] {{{body}}}
            }}
        "#
        );
        let run = crates.build(
            "codec_profile_boundary",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        assert!(!crates.vir("codec_profile_boundary").exists());
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("straight-line control flow") || stderr.contains("literal amounts"),
            "{}",
            stderr
        );
    }
}

#[test]
fn core_enum_array_queries_keep_their_existing_scalar_profile() {
    verify_exact(
        "core_enum_array_profile",
        r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn result_array(input: Result<u8, u8>) -> [u8; 2] {
    match input {
        Ok(value) if value < 255 => [value, value + 1],
        Ok(value) | Err(value) => [value, 0],
    }
}
#[verifier::infer]
pub fn option_array(input: Option<u8>) -> [u8; 2] {
    match input { Some(value) => [value, 1], None => [0, 0] }
}
fn caller() {
    let result = result_array(Ok(254));
    assert(result[0] == 254 && result[1] == 255);
    let result = result_array(Err(255));
    assert(result[0] == 255 && result[1] == 0);
    let result = option_array(None);
    assert(result[0] == 0 && result[1] == 0);
}
}
"#
        .to_owned(),
        2,
    );
}
