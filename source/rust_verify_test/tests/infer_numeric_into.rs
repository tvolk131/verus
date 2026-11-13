#![feature(rustc_private)]
#[allow(unused_imports)]
use rust_verify_test_macros::*;
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};

const PROVIDER: &str = r#"
use vstd::prelude::*;
#[verifier::infer] pub fn widen(number:u32)->u64 {number.into()}
#[verifier::infer] pub fn signed_widen(number:i32)->i64 {number.into()}
#[verifier::infer] pub fn mixed_widen(number:u8)->i32 {number.into()}
#[verifier::infer] pub fn identity(number:usize)->usize {number.into()}
#[verifier::infer] pub fn narrow(number:u64)->usize {number.try_into().unwrap()}
#[verifier::infer] pub fn signed(number:i64)->usize {number.try_into().unwrap()}
#[verifier::infer] pub fn huge(number:u128)->usize {number.try_into().unwrap()}
#[verifier::infer] pub fn increment(number:u64)->usize {(number+1).try_into().unwrap()}
#[verifier::infer] pub fn branch(number:u64,selected:bool)->usize {
    if selected {number.try_into().unwrap()} else {0}
}
"#;

#[test]
fn numeric_into_exact_both_architectures_cold_warm_and_imported() {
    let crates = InferCrates::new();
    let seed = crates.build("numeric_provider", PROVIDER, &[], &["--no-cheating", "--output-json"]);
    assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    assert_exact_report(&json_inference(&seed), 9);
    crates.edit_metadata("numeric_provider", |metadata| {
        let models = metadata["trusted_models"].to_string();
        for model in ["Into", "TryInto", "try_from", "from", "unwrap"] {
            assert!(models.contains(model), "{}", models);
        }
    });
    for bytes in [4, 8] {
        for _phase in ["cold", "warm"] {
            let body = format!(
                r#"
                global size_of usize == {bytes};
                use vstd::prelude::*;
                #[verifier::infer] pub fn relay(number:u64)->usize {{numeric_provider::narrow(number)}}
                fn exact(number:u64)->(result:usize)
                    requires number<=usize::MAX,
                    ensures result as int==number,
                {{relay(number)}}
                fn signed(number:i64)->(result:usize)
                    requires 0<=number<=usize::MAX,
                    ensures result as int==number,
                {{numeric_provider::signed(number)}}
                fn boundary()->(result:usize) ensures result==usize::MAX,
                {{numeric_provider::huge(usize::MAX as u128)}}
                fn inactive()->(result:usize) ensures result==0,
                {{numeric_provider::branch(u64::MAX,false)}}
                fn widening(number:u32)->(result:u64) ensures result==number,
                {{numeric_provider::widen(number)}}
                fn mixed(number:u8)->(result:i32) ensures result==number,
                {{numeric_provider::mixed_widen(number)}}
                fn signed_widen(number:i32)->(result:i64) ensures result==number,
                {{numeric_provider::signed_widen(number)}}
                fn identity(number:usize)->(result:usize) ensures result==number,
                {{numeric_provider::identity(number)}}
            "#
            );
            let run =
                crates.verify("numeric_client", &body, &["numeric_provider"], &["--output-json"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            assert_exact_report(&json_inference(&run), 1);
        }
    }
}

#[test]
fn numeric_into_safety_failures_do_not_publish() {
    let crates = InferCrates::new();
    for bytes in [4, 8] {
        for call in [
            "signed(-1)".to_string(),
            format!("huge({})", 1u128 << (bytes * 8)),
            "increment(u64::MAX)".to_string(),
            "branch(u64::MAX,true)".to_string(),
        ] {
            if bytes == 8 && call == "branch(u64::MAX,true)" {
                continue;
            }
            let run = crates.verify(
                "numeric_safety",
                &format!("global size_of usize == {bytes};{PROVIDER}fn bad()->usize{{{call}}}"),
                &[],
                &["--output-json"],
            );
            assert!(!run.status.success());
            assert_exact_report(&json_inference(&run), 9);
            assert!(
                String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            assert!(!crates.vir("numeric_safety").exists());
        }
    }
}

#[test]
fn numeric_into_semantic_false_goal_removes_seeded_export() {
    let crates = InferCrates::new();
    for bytes in [4, 8] {
        let seed = crates.verify(
            "numeric_false",
            &format!("global size_of usize == {bytes};{PROVIDER}"),
            &[],
            &["--output-json"],
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let run = crates.verify("numeric_false", &format!("global size_of usize == {bytes};{PROVIDER}fn wrong(number:u64)->(result:usize) requires number<=usize::MAX, ensures result as int!=number, {{narrow(number)}}"), &[], &["--output-json"]);
        assert!(!run.status.success());
        assert_exact_report(&json_inference(&run), 9);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains("postcondition not satisfied"), "{}", stderr);
        assert!(!stderr.contains("precondition not satisfied"), "{}", stderr);
        assert!(!crates.vir("numeric_false").exists());
    }
}

#[test]
fn numeric_into_lookalikes_and_non_numeric_impls_stay_closed() {
    let crates = InferCrates::new();
    for body in [
        "struct Number{value:u32} impl Number{fn into(self)->u64{self.value as u64+1}} #[verifier::infer] fn candidate(number:Number)->u64{number.into()}",
        "struct Number{value:u32} impl From<Number> for u64{fn from(number:Number)->u64{number.value as u64+1}} #[verifier::infer] fn candidate(number:Number)->u64{number.into()}",
        "struct Number{value:u64} impl TryFrom<Number> for usize{type Error=u8;fn try_from(number:Number)->Result<usize,u8>{Ok(0)}} #[verifier::infer] fn candidate(number:Number)->usize{number.try_into().unwrap()}",
        "struct Number{value:u64} impl Number{fn try_into(self)->Result<usize,u8>{Ok(0)}} #[verifier::infer] fn candidate(number:Number)->usize{number.try_into().unwrap()}",
    ] {
        let run = crates.build(
            "numeric_lookalike",
            &format!("use vstd::prelude::*;{body}"),
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains("Verus Infer unsupported"), "{}", stderr);
        assert!(!crates.vir("numeric_lookalike").exists());
    }
}

#[test]
fn numeric_into_import_policy_and_model_poisoning_fail_closed() {
    let crates = InferCrates::new();
    for mutation in 0..3 {
        let run =
            crates.build("numeric_provider", PROVIDER, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let seed = crates.build(
            "numeric_import",
            "#[verifier::infer] fn seed()->u64{0}",
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(seed.status.success());
        crates.edit_metadata("numeric_provider", |metadata| match mutation {
            0 => metadata["policy"] = "candidate-bound-evidence-v64".into(),
            1 => metadata["trusted_models"][0][1] = "poisoned".into(),
            _ => metadata["summaries"] = "poisoned".into(),
        });
        let run = crates.build(
            "numeric_import",
            "use vstd::prelude::*; #[verifier::infer] fn relay()->usize{numeric_provider::narrow(0)}",
            &["numeric_provider"],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        assert!(
            !crates.vir("numeric_import").exists(),
            "mutation {}: {}\n{}",
            mutation,
            String::from_utf8_lossy(&run.stderr),
            String::from_utf8_lossy(&run.stdout)
        );
    }
}

#[test]
fn numeric_into_stale_corrupt_and_ill_typed_active_cache_entries_regenerate() {
    let crates = InferCrates::new();
    let source = "use vstd::prelude::*; #[verifier::infer] pub fn narrow(number:u64)->usize{number.try_into().unwrap()} fn exact(number:u64)->(result:usize) requires number<=usize::MAX, ensures result as int==number, {narrow(number)}";
    for mutation in 0..3 {
        let seed = crates.build("numeric_cache", source, &[], &["--no-cheating", "--output-json"]);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let paths: Vec<_> = std::fs::read_dir(crates.directory.path().join("candidate-cache"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert!(!paths.is_empty());
        for path in paths {
            if mutation == 0 {
                std::fs::write(path, "corrupt").unwrap();
            } else {
                let mut candidate: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                if mutation == 1 {
                    candidate["version"] = "verus-infer-v70:checked-polynomial-goals:4096".into();
                } else {
                    candidate["pre"] = serde_json::json!({"Bool":true});
                    candidate["post"] = serde_json::json!({"Bool":true});
                }
                std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
            }
        }
        let run = crates.build("numeric_cache", source, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
    }
}

#[test]
fn numeric_into_valid_typed_cache_domain_poisoning_receives_fresh_required_checks() {
    for (condition, failed_check) in [(true, "soundness"), (false, "maximality")] {
        let crates = InferCrates::new();
        let source = "use vstd::prelude::*; #[verifier::infer] pub fn narrow(number:u64)->usize{number.try_into().unwrap()}";
        let seed = crates.build("numeric_poison", source, &[], &["--no-cheating", "--output-json"]);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        for entry in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = entry.unwrap().path();
            let mut candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            candidate["pre"] = serde_json::json!({"Bool":condition});
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
        let run = crates.build("numeric_poison", source, &[], &["--no-cheating", "--output-json"]);
        assert!(!run.status.success());
        let report = json_inference(&run);
        let function = &report["functions"][0];
        assert_eq!(function["metrics"]["cache"], "hit");
        assert_eq!(function["proofs"][failed_check]["status"], "not_proved");
        assert_eq!(function["proofs"][failed_check]["required"], true);
        assert_eq!(function["available_to_same_crate_callers"], false);
        assert_eq!(function["exported"], false);
        assert!(!crates.vir("numeric_poison").exists());
    }
}

#[test]
fn numeric_into_rust_width_and_signedness_near_misses_are_type_rejections() {
    let crates = InferCrates::new();
    for (input, output) in [("u64", "u32"), ("i32", "u32"), ("u64", "usize"), ("u32", "u16")] {
        let source = format!(
            "use vstd::prelude::*; #[verifier::infer] fn candidate(number:{input})->{output}{{number.into()}}"
        );
        let run = crates.build(
            "numeric_type_rejection",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(!run.status.success());
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(stderr.contains("trait bound"), "{}", stderr);
        assert!(!stderr.contains("postcondition not satisfied"), "{}", stderr);
        assert!(!crates.vir("numeric_type_rejection").exists());
    }
}
