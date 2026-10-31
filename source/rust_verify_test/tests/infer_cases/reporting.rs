use super::*;

#[test]
fn structured_report_cold_and_warm_candidates_have_fresh_proofs() {
    let crates = InferCrates::new();
    let source = "#[verifier::infer] pub fn increment(x:u8)->u8{x+1} fn caller(){let r=increment(2);assert(r==3);}";
    let mut first_contract = None;
    for cache in ["miss", "hit"] {
        let run = crates.build("json_cache", source, &[], &["--output-json", "--num-threads", "4"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["status"], "verified");
        assert_eq!(report["requested_functions"], 1);
        assert_eq!(report["interface_written"], true);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], cache);
        assert_eq!(f["required_checks_passed"], true);
        assert_eq!(f["exported"], true);
        assert_eq!(f["contract"]["requires"], "(x <= 254)");
        assert_eq!(f["contract"]["ensures"], "result == (x + 1)");
        for obligation in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][obligation]["status"], "proved");
        }
        assert_eq!(f["candidate_id"].as_str().unwrap().len(), 64);
        if let Some(contract) = first_contract {
            assert_eq!(f["contract"], contract);
        }
        first_contract = Some(f["contract"].clone());
    }
}

#[test]
fn structured_report_optional_timeouts_and_skipped_precision_are_honest() {
    let crates = InferCrates::new();
    let source = r#"
        #[verifier::infer(sound)] #[verifier::infer_precision_rlimit(0.000001)]
        pub fn leaf(b:bool)->u8 {if b{1}else{2}}
        #[verifier::infer(sound)] pub fn caller(b:bool)->u8 {leaf(b)+1}
    "#;
    let run = crates.build("json_optional", source, &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert_eq!(report["status"], "verified");
    let functions = report["functions"].as_array().unwrap();
    let leaf =
        functions.iter().find(|f| f["function"].as_str().unwrap().ends_with("::leaf")).unwrap();
    assert_eq!(leaf["proofs"]["exactness"]["status"], "incomplete");
    assert_eq!(leaf["proofs"]["exactness"]["reason"], "resource_limit");
    let caller =
        functions.iter().find(|f| f["function"].as_str().unwrap().ends_with("::caller")).unwrap();
    assert_eq!(caller["proofs"]["maximality"]["status"], "not_established");
    assert_eq!(caller["proofs"]["maximality"]["reason"], "sound_mode_dependency");
    assert_eq!(caller["dependencies"][0]["candidate_id"], leaf["candidate_id"]);
    for f in functions {
        assert_eq!(f["proofs"]["soundness"]["status"], "proved");
    }
}

#[test]
fn structured_report_caller_failure_blocks_export_after_valid_inference() {
    let crates = InferCrates::new();
    let run = crates.build("json_caller", "#[verifier::infer] pub fn identity(x:u8)->u8{x} fn wrong(){let r=identity(1);assert(r==2);}", &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "caller_proof_failed");
    assert_eq!(report["interface_written"], false);
    assert_eq!(report["functions"][0]["required_checks_passed"], true);
    assert_eq!(report["functions"][0]["available_to_same_crate_callers"], true);
    assert_eq!(report["functions"][0]["exported"], false);
    assert!(!crates.vir("json_caller").exists());
}

#[test]
fn structured_report_unsupported_and_required_timeout_do_not_publish() {
    let crates = InferCrates::new();
    for (body, expected) in [
        ("#[verifier::infer] pub fn looping(x:u8)->u8{loop{}}", "unsupported"),
        (
            "#[verifier::infer] #[verifier::infer_precision_rlimit(0.000001)] pub fn select(b:bool)->u8{if b{1}else{2}}",
            "required_proof_incomplete",
        ),
    ] {
        let run = crates.build("json_failure", body, &[], &["--output-json"]);
        assert!(!run.status.success());
        let report = json_inference(&run);
        assert_eq!(report["status"], expected, "{report}");
        assert_eq!(report["interface_written"], false);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["available_to_same_crate_callers"], false);
            assert_eq!(f["exported"], false);
        }
    }
}

#[test]
fn structured_report_imported_identity_and_model_fingerprints() {
    let crates = InferCrates::new();
    let base = crates.build(
        "json_base",
        "#[verifier::infer] pub fn base(x:u8)->u8{x+1}",
        &[],
        &["--output-json"],
    );
    assert!(base.status.success());
    let base = json_inference(&base);
    let run = crates.build(
        "json_middle",
        "#[verifier::infer] pub fn next(x:u8)->u8{json_base::base(x)+1}",
        &["json_base"],
        &["--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert_eq!(report["imported_inference_dependencies"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["functions"][0]["dependencies"][0]["candidate_id"],
        base["functions"][0]["candidate_id"]
    );
    let run = crates.build("json_model", "use vstd::prelude::*; #[verifier::infer] pub fn checked(x:u8)->Option<u8>{x.checked_add(1)}", &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert!(!report["trusted_models"].as_array().unwrap().is_empty());
    assert!(report["trusted_models"][0]["fingerprint"].as_str().unwrap().len() == 64);
}

#[test]
fn structured_report_poisoned_cache_is_reproved_and_stale_export_removed() {
    let crates = InferCrates::new();
    let body = "#[verifier::infer] pub fn increment(x:u8)->u8{x+1}";
    let run = crates.build("json_poison", body, &[], &["--output-json"]);
    assert!(run.status.success());
    let original = json_inference(&run);
    let cache = crates.directory.path().join("candidate-cache");
    for entry in std::fs::read_dir(cache).unwrap() {
        let path = entry.unwrap().path();
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        candidate["pre"] = serde_json::json!({"Bool": true});
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    }
    let run = crates.build("json_poison", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_failed");
    let f = &report["functions"][0];
    assert_eq!(f["metrics"]["cache"], "hit");
    assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
    assert_eq!(f["required_checks_passed"], false);
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert_eq!(f["exported"], false);
    assert_ne!(f["candidate_id"], original["functions"][0]["candidate_id"]);
    assert!(!crates.vir("json_poison").exists());
}
