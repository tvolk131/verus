#![feature(rustc_private)]

#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const PROVIDER: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer] pub fn encode(number: u64) -> [u8; 8] { number.to_le_bytes() }
    #[verifier::infer] pub fn decode(bytes: [u8; 8]) -> u64 { u64::from_le_bytes(bytes) }
    #[verifier::infer] pub fn word_roundtrip(number: u64) -> u64 { decode(encode(number)) }
    #[verifier::infer] pub fn byte_roundtrip(bytes: [u8; 8]) -> [u8; 8] { encode(decode(bytes)) }
    #[verifier::infer] pub fn constant() -> [u8; 8] { 0x0123456789abcdefu64.to_le_bytes() }
    #[verifier::infer] pub fn increment(number: u64) -> [u8; 8] { (number + 1).to_le_bytes() }
"#;

const GOALS: &str = r#"
    fn words(number: u64) -> (result: u64) ensures result == number {
        proof {
            vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
            assert(vstd::bytes::spec_u64_from_le_bytes(vstd::bytes::spec_u64_to_le_bytes(number)) == number);
        }
        word_roundtrip(number)
    }
    fn byte_identity(bytes: [u8; 8]) -> (result: [u8; 8]) ensures result == bytes {
        proof {
            vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
            assert(vstd::bytes::spec_u64_to_le_bytes(vstd::bytes::spec_u64_from_le_bytes(bytes@)) == bytes@);
        }
        byte_roundtrip(bytes)
    }
    fn encoding(number: u64) -> (result: [u8; 8])
        ensures result@ == vstd::bytes::spec_u64_to_le_bytes(number), result@.len() == 8,
            result[0] == (number & 255) as u8,
            result[1] == ((number >> 8) & 255) as u8,
            result[2] == ((number >> 16) & 255) as u8,
            result[3] == ((number >> 24) & 255) as u8,
            result[4] == ((number >> 32) & 255) as u8,
            result[5] == ((number >> 40) & 255) as u8,
            result[6] == ((number >> 48) & 255) as u8,
            result[7] == ((number >> 56) & 255) as u8,
    { encode(number) }
    fn decoding(bytes: [u8; 8]) -> (result: u64)
        ensures result == vstd::bytes::spec_u64_from_le_bytes(bytes@),
    { decode(bytes) }
"#;

fn published(run: &std::process::Output, crates: &InferCrates, name: &str, success: bool) {
    assert_eq!(run.status.success(), success, "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(run);
    assert_eq!(report["crate_success"], success, "{report}");
    assert_eq!(report["interface_written"], success, "{report}");
    assert_eq!(crates.vir(name).exists(), success);
}

#[test]
fn ordinary_native_apis_and_existing_inverse_lemmas() {
    let crates = InferCrates::new();
    let source = format!(
        r#"use vstd::prelude::*;
        fn encode(number: u64) -> (result: [u8; 8])
            ensures result@ == vstd::bytes::spec_u64_to_le_bytes(number),
        {{ number.to_le_bytes() }}
        fn decode(bytes: [u8; 8]) -> (result: u64)
            ensures result == vstd::bytes::spec_u64_from_le_bytes(bytes@),
        {{ u64::from_le_bytes(bytes) }}
        fn word_roundtrip(number: u64) -> (result: u64)
            ensures result == vstd::bytes::spec_u64_from_le_bytes(vstd::bytes::spec_u64_to_le_bytes(number)),
        {{ u64::from_le_bytes(number.to_le_bytes()) }}
        fn byte_roundtrip(bytes: [u8; 8]) -> (result: [u8; 8])
            ensures result@ == vstd::bytes::spec_u64_to_le_bytes(vstd::bytes::spec_u64_from_le_bytes(bytes@)),
        {{ u64::from_le_bytes(bytes).to_le_bytes() }}
        {GOALS}"#,
    );
    let run = crates.build("ordinary_bytes", &source, &[], &["--no-cheating", "--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
}

#[test]
fn exact_cold_warm_and_imported_composition() {
    let crates = InferCrates::new();
    for cache in ["miss", "hit"] {
        let run = crates.build(
            "byte_provider",
            &format!("{PROVIDER}{GOALS}"),
            &[],
            &["--no-cheating", "--output-json"],
        );
        published(&run, &crates, "byte_provider", true);
        let report = json_inference(&run);
        assert_exact_report(&report, 6);
        for function_report in report["functions"].as_array().unwrap() {
            assert_eq!(function_report["metrics"]["cache"], cache, "{function_report}");
        }
        crates.edit_metadata("byte_provider", |metadata| {
            assert!(
                metadata["trusted_models"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry.to_string().contains("to_le_bytes"))
            );
            assert!(
                metadata["trusted_models"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry.to_string().contains("from_le_bytes"))
            );
        });
    }
    for cache in ["miss", "hit"] {
        let run = crates.build("byte_client", r#"
            use vstd::prelude::*;
            #[verifier::infer] pub fn again(number: u64) -> u64 { byte_provider::word_roundtrip(number) }
            #[verifier::infer] pub fn again_bytes(bytes: [u8; 8]) -> [u8; 8] { byte_provider::byte_roundtrip(bytes) }
            fn property(number: u64, bytes: [u8; 8]) {
                proof {
                    vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
                    assert(vstd::bytes::spec_u64_from_le_bytes(vstd::bytes::spec_u64_to_le_bytes(number)) == number);
                    assert(vstd::bytes::spec_u64_to_le_bytes(vstd::bytes::spec_u64_from_le_bytes(bytes@)) == bytes@);
                }
                let result = again(number); assert(result == number);
                let result = again_bytes(bytes); assert(result == bytes);
            }
        "#, &["byte_provider"], &["--no-cheating", "--output-json"]);
        published(&run, &crates, "byte_client", true);
        let report = json_inference(&run);
        assert_exact_report(&report, 2);
        assert!(
            report["functions"]
                .as_array()
                .unwrap()
                .iter()
                .all(|function_report| function_report["metrics"]["cache"] == cache)
        );
    }
}

#[test]
fn reversed_shifted_replaced_bytes_and_wrong_caller_property_are_semantic_failures() {
    let crates = InferCrates::new();
    for body in [
        "let bytes=number.to_le_bytes();u64::from_le_bytes([bytes[7],bytes[6],bytes[5],bytes[4],bytes[3],bytes[2],bytes[1],bytes[0]])",
        "let bytes=number.to_le_bytes();u64::from_le_bytes([bytes[1],bytes[2],bytes[3],bytes[4],bytes[5],bytes[6],bytes[7],bytes[0]])",
        "let bytes=number.to_le_bytes();u64::from_le_bytes([0,bytes[1],bytes[2],bytes[3],bytes[4],bytes[5],bytes[6],bytes[7]])",
        "u64::from_le_bytes(number.to_le_bytes())",
    ] {
        let property =
            if body.starts_with("u64::") { "result != number" } else { "result == number" };
        let source = format!(
            r#"use vstd::prelude::*;
            #[verifier::infer] pub fn codec(number:u64)->u64{{{body}}}
            fn wrong(number:u64)->(result:u64) ensures {property}, {{codec(number)}}"#
        );
        let run = crates.build("byte_negative", &source, &[], &["--no-cheating", "--output-json"]);
        published(&run, &crates, "byte_negative", false);
        assert_exact_report(&json_inference(&run), 1);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains("postcondition not satisfied"), "{}", stderr);
        assert!(!stderr.contains("not supported") && !stderr.contains("resource limit"));
    }
}

#[test]
fn metadata_and_model_artifacts_are_bound() {
    for mutation in ["policy", "proof", "summary", "model", "artifact"] {
        let crates = InferCrates::new();
        let run = crates.build("byte_provider", PROVIDER, &[], &["--no-cheating", "--output-json"]);
        published(&run, &crates, "byte_provider", true);
        if mutation == "artifact" {
            let mut bytes = std::fs::read(crates.profile.join("vstd.vir")).unwrap();
            bytes.push(0);
            std::fs::write(crates.directory.path().join("vstd.vir"), bytes).unwrap();
        } else {
            crates.edit_metadata("byte_provider", |metadata| match mutation {
                "policy" => metadata["policy"] = serde_json::json!("candidate-bound-evidence-v54"),
                "proof" => metadata["soundness"] = serde_json::json!(false),
                "summary" => metadata["summaries"] = serde_json::json!("corrupted"),
                "model" => metadata["trusted_models"][0][1] = serde_json::json!("stale"),
                _ => unreachable!(),
            });
        }
        let run = crates.build("bad_byte_client", "use vstd::prelude::*; #[verifier::infer] pub fn function_report(number:u64)->u64{byte_provider::word_roundtrip(number)}", &["byte_provider"], &["--no-cheating", "--output-json"]);
        assert!(!run.status.success());
        assert!(!crates.vir("bad_byte_client").exists());
        let stderr = String::from_utf8_lossy(&run.stderr);
        let expected = match mutation {
            "policy" => "incompatible inference schema, safety policy, or producer",
            "proof" => "incomplete inference proof guarantees",
            "summary" => "inferred contract fingerprint mismatch",
            _ => "stale or missing trusted-model fingerprint",
        };
        assert!(stderr.contains(expected), "{}: {}", mutation, stderr);
    }
}

#[test]
fn a_lookalike_does_not_acquire_a_library_contract() {
    let crates = InferCrates::new();
    let run = crates.build(
        "byte_lookalike",
        r#"
        use vstd::prelude::*;
        fn to_le_bytes(number:u64)->[u8;8]{[0;8]}
        #[verifier::infer] pub fn call(number:u64)->[u8;8]{to_le_bytes(number)}
    "#,
        &[],
        &["--no-cheating", "--output-json"],
    );
    published(&run, &crates, "byte_lookalike", false);
    assert!(String::from_utf8_lossy(&run.stderr).contains(
        "calls require an acyclic same-crate inferred callee or certified imported summary"
    ));
}

#[test]
fn poisoned_byte_candidates_receive_fresh_checks_and_remove_stale_exports() {
    for mutation in ["reversed", "restrictive"] {
        let crates = InferCrates::new();
        let source = "use vstd::prelude::*; #[verifier::infer] pub fn encode(number:u64)->[u8;8]{number.to_le_bytes()}";
        let run = crates.build("byte_poison", source, &[], &["--no-cheating", "--output-json"]);
        published(&run, &crates, "byte_poison", true);
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            if mutation == "reversed" {
                candidate["post"]["Binary"][2]["Array"][2].as_array_mut().unwrap().reverse();
            } else {
                candidate["pre"] = serde_json::json!({"Bool": false});
            }
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
        let run = crates.build("byte_poison", source, &[], &["--no-cheating", "--output-json"]);
        published(&run, &crates, "byte_poison", false);
        let report = json_inference(&run);
        let function_report = &report["functions"][0];
        assert_eq!(function_report["metrics"]["cache"], "hit", "{function_report}");
        let check = if mutation == "reversed" { "soundness" } else { "maximality" };
        assert_eq!(function_report["proofs"][check]["status"], "not_proved", "{function_report}");
        assert_eq!(function_report["available_to_same_crate_callers"], false);
        assert_eq!(function_report["exported"], false);
    }
}
