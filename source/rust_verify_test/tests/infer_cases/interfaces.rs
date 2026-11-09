use super::*;

#[test]
fn cross_crate_inferred_chain_and_stale_dependencies() {
    let crates = InferCrates::new();
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let exported = std::fs::read(crates.vir("infer_base")).unwrap();
    assert!(!exported.windows(b"secret_helper".len()).any(|s| s == b"secret_helper"));
    crates.succeeds("infer_middle", INFER_MIDDLE, &["infer_base"]);
    let run = crates.succeeds("infer_app", INFER_APP, &["infer_base", "infer_middle"]);
    assert!(String::from_utf8_lossy(&run.stderr).contains("output exactness: PROVED"));
    // Keep Rust metadata compatible while replacing just the Verus artifact;
    // otherwise rustc rejects the stale rlib before the inference boundary runs.
    let rlib_path = crates.directory.path().join("libinfer_base.rlib");
    let original_rlib = std::fs::read(&rlib_path).unwrap();
    crates.succeeds("infer_base", &INFER_BASE.replace("x + 1", "x + 2"), &[]);
    std::fs::write(&rlib_path, original_rlib).unwrap();
    let stale = crates.build("infer_app", INFER_APP, &["infer_base", "infer_middle"], &[]);
    assert!(!stale.status.success());
    assert!(
        String::from_utf8_lossy(&stale.stderr)
            .contains("stale or missing inferred dependency fingerprint"),
        "{}",
        String::from_utf8_lossy(&stale.stderr)
    );
    assert!(!crates.vir("infer_app").exists());
    crates.succeeds("infer_base", &INFER_BASE.replace("x + 1", "x + 2"), &[]);
    crates.succeeds("infer_middle", INFER_MIDDLE, &["infer_base"]);
    crates.succeeds(
        "infer_app",
        &INFER_APP.replace("top(253)", "top(251)"),
        &["infer_base", "infer_middle"],
    );
}

#[test]
fn imported_inference_requires_complete_compatible_metadata() {
    let crates = InferCrates::new();
    for (field, value, expected) in [
        ("schema", serde_json::json!(999), "incompatible inference schema"),
        ("policy", serde_json::json!("wrapping"), "incompatible inference schema"),
        ("producer", serde_json::json!("incompatible compiler"), "incompatible inference schema"),
        ("soundness", serde_json::json!(false), "incomplete inference proof guarantees"),
        ("maximality", serde_json::json!(false), "incomplete inference proof guarantees"),
        ("exactness", serde_json::json!(false), "incomplete inference proof guarantees"),
        ("summaries", serde_json::json!("corrupted"), "inferred contract fingerprint mismatch"),
    ] {
        crates.succeeds("infer_base", INFER_BASE, &[]);
        crates.edit_metadata("infer_base", |metadata| metadata[field] = value);
        let run = crates.build("infer_middle", INFER_MIDDLE, &["infer_base"], &[]);
        assert!(!run.status.success(), "accepted incompatible metadata: {}", field);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(!crates.vir("infer_middle").exists());
    }
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let path = crates.vir("infer_base");
    let mut bytes = std::fs::read(&path).unwrap();
    let start = bytes.windows(b"{\"schema\":".len()).rposition(|s| s == b"{\"schema\":").unwrap();
    bytes.truncate(start);
    std::fs::write(path, bytes).unwrap();
    let run = crates.build("infer_middle", INFER_MIDDLE, &["infer_base"], &[]);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("missing or invalid inference proof metadata"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn failed_or_partial_verification_never_exports_inference() {
    let crates = InferCrates::new();
    for options in [vec!["--no-verify"], vec!["--verify-root", "--verify-function", "increment"]] {
        crates.succeeds("infer_base", INFER_BASE, &[]);
        let run = crates.build("infer_base", INFER_BASE, &[], &options);
        assert!(!run.status.success());
        assert!(
            !crates.vir("infer_base").exists(),
            "options {:?}: {}",
            options,
            String::from_utf8_lossy(&run.stderr)
        );
    }
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let run = crates.build(
        "infer_base",
        &format!("{} pub fn failed() {{ assert(false); }}", INFER_BASE),
        &[],
        &[],
    );
    assert!(!run.status.success());
    assert!(!crates.vir("infer_base").exists());
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let invalid = crates.build("infer_base", "this is not valid Rust;", &[], &[]);
    assert!(!invalid.status.success());
    assert!(!crates.vir("infer_base").exists());
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let limited = r#"
        #[verifier::infer]
        #[verifier::rlimit(0.000001)]
        pub fn multiply(x: u64, y: u64) -> u64 { x * y }
    "#;
    let run = crates.build("infer_base", limited, &[], &[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("Resource limit"));
    assert!(!crates.vir("infer_base").exists());
}

#[test]
fn ordinary_library_exports_remain_compatible() {
    let crates = InferCrates::new();
    crates.succeeds(
        "ordinary_base",
        r#"
        pub fn increment(x: u8) -> (r: u8)
            requires x < 255,
            ensures r as int == x as int + 1,
        { x + 1 }
    "#,
        &[],
    );
    assert!(!std::fs::read(crates.vir("ordinary_base")).unwrap().ends_with(b"VRINFER1"));
    crates.succeeds(
        "ordinary_user",
        r#"
        pub fn caller() { let r = ordinary_base::increment(254); assert(r == 255); }
    "#,
        &["ordinary_base"],
    );
}

#[test]
fn ordinary_goal_providers_require_full_proofs_and_reject_previous_policy() {
    let crates = InferCrates::new();
    let invalid = "pub fn selected(){} pub mod omitted {pub fn value()->(r:u8) ensures r==1, {0}}";
    let valid = invalid.replace("r==1", "r==0");
    let options = ["--infer-goals", "--output-json"];
    for bypass in [
        vec!["--no-verify"],
        vec!["--verify-root"],
        vec!["--verify-root", "--verify-function", "selected"],
        vec!["--verify-only-module", "omitted"],
        vec!["--verify-module", "omitted"],
    ] {
        // Seed a genuinely checked ordinary-only interface before each failed
        // rebuild. No inferred helper or recognized assistance profile can
        // accidentally trigger the older, narrower scope guard in this case.
        let seed = crates.build("ordinary_goal_provider", &valid, &[], &options);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let report = json_inference(&seed);
        assert!(report["functions"].as_array().unwrap().is_empty());
        assert_eq!(report["interface_written"], true);
        assert!(crates.vir("ordinary_goal_provider").exists());
        let flags: Vec<_> = options.iter().copied().chain(bypass.iter().copied()).collect();
        let run = crates.build("ordinary_goal_provider", invalid, &[], &flags);
        assert!(!run.status.success(), "bypass accepted: {:?}", bypass);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let report = json_inference(&run);
        assert_eq!(report["interface_written"], false);
        assert!(!crates.vir("ordinary_goal_provider").exists());
    }

    let caller = "pub fn caller(){let r=ordinary_goal_provider::omitted::value(); assert(r==0);}";
    for old_policy in ["candidate-bound-evidence-v63", "candidate-bound-evidence-v64"] {
        let provider = crates.build("ordinary_goal_provider", &valid, &[], &options);
        assert!(provider.status.success(), "{}", String::from_utf8_lossy(&provider.stderr));
        let checked =
            crates.build("ordinary_goal_consumer", caller, &["ordinary_goal_provider"], &options);
        assert!(checked.status.success(), "{}", String::from_utf8_lossy(&checked.stderr));
        assert!(crates.vir("ordinary_goal_consumer").exists());
        crates.edit_metadata("ordinary_goal_provider", |metadata| {
            assert_ne!(metadata["policy"], old_policy);
            metadata["policy"] = serde_json::json!(old_policy);
        });
        let stale =
            crates.build("ordinary_goal_consumer", caller, &["ordinary_goal_provider"], &options);
        assert!(!stale.status.success());
        assert!(
            String::from_utf8_lossy(&stale.stderr).contains("incompatible inference schema"),
            "{}",
            String::from_utf8_lossy(&stale.stderr)
        );
        assert!(!crates.vir("ordinary_goal_consumer").exists());
    }
}

#[test]
fn ordinary_goal_providers_require_lifetime_and_trait_consistency_checks() {
    let crates = InferCrates::new();
    let identity = "pub proof fn identity<T>(tracked value:T)->(tracked result:T){value}";
    let duplicate =
        "pub proof fn duplicate<T>(tracked value:T)->(tracked result:(T,T)){(value,value)}";
    let options = ["--infer-goals", "--output-json"];
    for (flag, body) in [("--no-lifetime", duplicate), ("--no-trait-conflicts", identity)] {
        let seed = crates.build("tracked_goal_provider", identity, &[], &options);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        assert!(crates.vir("tracked_goal_provider").exists());
        let run = crates.build(
            "tracked_goal_provider",
            body,
            &[],
            &["--infer-goals", "--output-json", flag],
        );
        assert!(!run.status.success(), "unchecked provider accepted: {}", flag);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("normal lifetime/trait checks"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(json_inference(&run)["interface_written"], false);
        assert!(!crates.vir("tracked_goal_provider").exists());
    }
    let checked = crates.build("tracked_goal_provider", duplicate, &[], &options);
    assert!(!checked.status.success());
    assert!(String::from_utf8_lossy(&checked.stderr).contains("use of moved value"));
    assert!(!crates.vir("tracked_goal_provider").exists());
}

#[test]
fn ordinary_word_goal_exports_bind_standard_library_models() {
    let crates = InferCrates::new();
    let options = ["--infer-goals", "--output-json"];
    let provider = r#"
        pub fn grant(word:u64,index:u64)->(result:u64)
            requires index<64,
            ensures vstd::word::view(result,64) ==
                vstd::word::view(word,64).update(index as int,true),
        {word | (1u64 << index)}
    "#;
    let run = crates.build("word_goal_provider", provider, &[], &options);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(json_inference(&run)["functions"].as_array().unwrap().is_empty());
    let caller = r#"
        pub fn grant(word:u64,index:u64)->(result:u64)
            requires index<64,
            ensures vstd::word::view(result,64) ==
                vstd::word::view(word,64).update(index as int,true),
        {word_goal_provider::grant(word,index)}
    "#;
    let positive = crates.build("word_goal_consumer", caller, &["word_goal_provider"], &options);
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    assert!(crates.vir("word_goal_consumer").exists());
    crates.edit_metadata("word_goal_provider", |metadata| {
        let models = metadata["trusted_models"].as_array_mut().unwrap();
        assert!(!models.is_empty(), "ordinary checked word laws need library provenance");
        models[0][1] = serde_json::json!("changed-word-model");
    });
    let stale = crates.build("word_goal_consumer", caller, &["word_goal_provider"], &options);
    assert!(!stale.status.success());
    assert!(
        String::from_utf8_lossy(&stale.stderr).contains("stale or missing trusted-model fingerprint"),
        "{}",
        String::from_utf8_lossy(&stale.stderr)
    );
    assert!(!crates.vir("word_goal_consumer").exists());
}
