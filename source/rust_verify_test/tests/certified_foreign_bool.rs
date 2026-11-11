#![feature(rustc_private)]
mod common;
use common::infer::InferCrates;

const PROVIDER: &str = "use vstd::prelude::*; verus! {pub fn all_bool(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}";
fn ok(o: &std::process::Output) {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}
fn metadata(path: &std::path::Path) -> (Vec<u8>, serde_json::Value) {
    let b = std::fs::read(path).unwrap();
    assert!(b.ends_with(b"VRINFER1"));
    let i = b.windows(b"{\"schema\":".len()).rposition(|x| x == b"{\"schema\":").unwrap();
    (b[..i].to_vec(), serde_json::from_slice(&b[i..b.len() - 8]).unwrap())
}

#[test]
fn checked_full_domain_export_and_ordinary_transitive_import() {
    let c = InferCrates::new();
    ok(&c.build_source("provider", PROVIDER, &[], &["--infer-goals", "--no-cheating"], true));
    let (_, j) = metadata(&c.vir("provider"));
    assert_eq!(j["schema"], 39);
    assert_eq!(j["foreign_dependencies"].as_array().unwrap().len(), 1);
    assert!(j["proofs"].as_array().unwrap().is_empty());
    let client =
        "pub fn result(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {provider::all_bool(b)}";
    ok(&c.build("bridge", client, &["provider"], &["--no-cheating"]));
    let (_, b) = metadata(&c.vir("bridge"));
    assert_eq!(j["foreign_dependencies"], b["foreign_dependencies"]);
    ok(&c.build(
        "third",
        &client.replace("provider::all_bool", "bridge::result"),
        &["provider", "bridge"],
        &["--no-cheating"],
    ));
    let bad = c.build(
        "false_client",
        &client.replace("r==if b {1u64}else{0u64}", "r==0u64"),
        &["provider"],
        &["--no-cheating"],
    );
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("postcondition not satisfied"));
    assert!(!c.vir("false_client").exists());
    let missing = c.build(
        "missing",
        &client.replace("provider::all_bool", "bridge::result"),
        &["bridge"],
        &["--no-cheating"],
    );
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("dependency fingerprint"));
}

#[test]
fn private_helper_evidence_survives_pruning_and_disabled_checks_refuse() {
    let c = InferCrates::new();
    let source = PROVIDER.replace("pub fn all_bool", "fn all_bool")
        + "verus! {pub fn value()->(r:u64) ensures r==7u64, {7}}";
    ok(&c.build_source("private", &source, &[], &["--infer-goals", "--no-cheating"], true));
    let (_, j) = metadata(&c.vir("private"));
    assert_eq!(j["foreign_dependencies"].as_array().unwrap().len(), 1);
    ok(&c.build(
        "client",
        "pub fn value()->(r:u64) ensures r==7u64, {private::value()}",
        &["private"],
        &["--no-cheating"],
    ));
    for flag in ["--no-verify", "--no-lifetime", "--no-trait-conflicts", "--no-erasure-check"] {
        let o = c.build(
            "disabled",
            "pub fn value()->u64 {private::value()}",
            &["private"],
            &[flag, "--no-cheating"],
        );
        assert!(!o.status.success(), "{flag}");
        assert!(!c.vir("disabled").exists(), "{flag}");
    }
}

#[test]
fn missing_stale_or_incompatible_required_evidence_refuses() {
    let c = InferCrates::new();
    ok(&c.build_source("provider", PROVIDER, &[], &["--infer-goals", "--no-cheating"], true));
    let (prefix, j) = metadata(&c.vir("provider"));
    for change in [
        "schema",
        "missing",
        "empty",
        "target",
        "typed_mir",
        "core_artifact",
        "compiler",
        "semantic_instance",
    ] {
        let mut m = j.clone();
        match change {
            "schema" => m["schema"] = 38.into(),
            "missing" => {
                m.as_object_mut().unwrap().remove("foreign_dependencies");
            }
            "empty" => m["foreign_dependencies"] = serde_json::json!([]),
            field => m["foreign_dependencies"][0]["identity"][field] = "changed".into(),
        }
        let mut bytes = prefix.clone();
        bytes.extend(serde_json::to_vec(&m).unwrap());
        bytes.extend(b"VRINFER1");
        std::fs::write(c.vir("provider"), bytes).unwrap();
        let o=c.build("consumer","pub fn result(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {provider::all_bool(b)}",&["provider"],&["--no-cheating"]);
        assert!(!o.status.success(), "{change}");
        assert!(!c.vir("consumer").exists(), "{change}");
    }
}
