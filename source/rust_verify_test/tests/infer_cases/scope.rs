use super::*;

fn scope(run: &std::process::Output) -> serde_json::Value {
    json_inference(run)["verification_scope"].clone()
}

fn status<'a>(report: &'a serde_json::Value, suffix: &str) -> &'a str {
    report["native_functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| {
            let name = f["rust_function"].as_str().unwrap();
            name == suffix.trim_start_matches("::") || name.ends_with(suffix)
        })
        .unwrap_or_else(|| panic!("missing {}: {}", suffix, report))["body_status"]
        .as_str()
        .unwrap()
}

#[test]
fn scope_distinguishes_checked_bodies_unannotated_code_and_unproved_interfaces() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
fn unverified() -> u8 { panic!("outside proof scope") }
#[verus_infer] pub fn plus_one(n:u8)->u8 {n+1}
verus! {
 pub trait Event { fn number(&self)->u8; }
 pub open spec fn meaning(n:u8)->int { n as int+1 }
 pub fn goal(n:u8)->(r:u8) requires n<255, ensures r as int==meaning(n), { plus_one(n) }
}
"#;
    let run = crates.build_source(
        "scope_provider",
        source,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = scope(&run);
    assert_eq!(report["native_collection_completed"], true);
    assert_eq!(report["complete_selected_scope_verified"], true);
    assert_eq!(status(&report, "::unverified"), "outside_selected_scope");
    assert_eq!(status(&report, "::plus_one"), "checked_exec_body");
    assert_eq!(status(&report, "::goal"), "checked_exec_body");
    assert_eq!(status(&report, "::number"), "interface_without_checked_body");
    assert_eq!(status(&report, "::meaning"), "specification_definition_or_declaration");
    assert_eq!(report["checked_exec_bodies"], 2);

    let bad = source.replace("r as int==meaning(n)", "r as int==meaning(n)+1");
    let run =
        crates.build_source("scope_provider", &bad, &[], &["--no-cheating", "--output-json"], true);
    assert!(!run.status.success());
    let report = scope(&run);
    assert_eq!(report["complete_selected_scope_verified"], false);
    assert_eq!(report["checked_exec_bodies"], 0);
    assert_eq!(status(&report, "::goal"), "selected_body_not_certified_by_successful_full_run");
    assert!(!crates.vir("scope_provider").exists());
}

#[test]
fn scope_never_calls_an_external_body_a_checked_implementation() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
verus! {
 #[verifier::external_body] pub fn assumed()->(r:u8) ensures r==12, {0}
 pub fn goal(){let r=assumed();assert(r==12);}
}
"#;
    let run = crates.build_source("scope_assumption", source, &[], &["--output-json"], true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = scope(&run);
    assert_eq!(status(&report, "::assumed"), "unverified_body_with_contract");
    assert_eq!(status(&report, "::goal"), "checked_exec_body");
    assert_eq!(report["checked_exec_bodies"], 1);
}

#[test]
fn scope_does_not_certify_filtered_or_no_verify_runs() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;verus!{pub fn ordinary()->u8{3}}"#;
    for options in [
        vec!["--no-verify", "--output-json"],
        vec!["--verify-root", "--verify-function", "ordinary", "--output-json"],
    ] {
        let run = crates.build_source("scope_filtered", source, &[], &options, true);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = scope(&run);
        assert_eq!(report["complete_selected_scope_verified"], false);
        assert_eq!(report["checked_exec_bodies"], 0);
        assert_eq!(
            status(&report, "::ordinary"),
            "selected_body_not_certified_by_successful_full_run"
        );
    }
}

#[test]
fn native_scope_is_available_on_frontend_failure_without_claiming_proofs() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
fn outside()->u8{4}
#[verus_verify] pub fn unsupported(){let _=std::process::id();}
"#;
    let run = crates.build_source("scope_frontend", source, &[], &["--output-json"], true);
    assert!(!run.status.success());
    let report = scope(&run);
    assert_eq!(report["native_collection_completed"], true);
    assert_eq!(report["complete_selected_scope_verified"], false);
    assert_eq!(report["checked_exec_bodies"], 0);
    assert_eq!(status(&report, "::outside"), "outside_selected_scope");
    assert_eq!(status(&report, "::unsupported"), "selected_item_without_translated_body");
}
