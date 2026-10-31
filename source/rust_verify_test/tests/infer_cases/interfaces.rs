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
