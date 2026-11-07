#![feature(rustc_private)]

mod common;
use common::infer::{InferCrates, json_inference};

const HELPER: &str = "#[verifier::infer] pub fn increment(value:u8)->u8 {value+1}";
const ORDINARY: &str = "pub fn identity(value:u8)->(result:u8) ensures result==value {value}";
const ORDINARY_FAILURE: &str = "pub proof fn ordinary_failure() { assert(false); }";
const NATIVE_FAILURE: &str = r#"
    #[cfg(not(verus_keep_ghost_body))]
    compile_error!("native publication regression");
"#;

fn assert_native_failure(crates: &InferCrates, name: &str, run: &std::process::Output) {
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("native publication regression"));
    assert!(!crates.vir(name).exists());
    let report = json_inference(run);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
    for function in report["functions"].as_array().unwrap() {
        assert_eq!(function["exported"], false);
        for obligation in ["soundness", "maximality", "exactness"] {
            assert_eq!(function["proofs"][obligation]["status"], "proved");
        }
    }
}

fn raw_publication(
    crates: &InferCrates,
    name: &str,
    body: &str,
    export: &std::path::Path,
    options: &[&str],
) -> std::process::Output {
    let source = crates.directory.path().join(format!("{name}.rs"));
    std::fs::write(
        &source,
        format!("use verus_builtin::*; use verus_builtin_macros::*; verus! {{ {body} }}"),
    )
    .unwrap();
    let mut arguments = vec![
        "--internal-test-mode".to_owned(),
        "--crate-type=lib".to_owned(),
        "--edition=2021".to_owned(),
        "--crate-name".to_owned(),
        name.to_owned(),
        "--cfg=verus_no_vstd".to_owned(),
        "--no-cheating".to_owned(),
        "--output-json".to_owned(),
        "--export".to_owned(),
        export.display().to_string(),
        "--extern".to_owned(),
        format!("verus_builtin={}", crates.profile.join("libverus_builtin.rlib").display()),
        "--extern".to_owned(),
        format!(
            "verus_builtin_macros={}",
            crates
                .profile
                .join(format!(
                    "{}verus_builtin_macros.{}",
                    std::env::consts::DLL_PREFIX,
                    std::env::consts::DLL_EXTENSION
                ))
                .display()
        ),
        "-L".to_owned(),
        format!("dependency={}", crates.profile.display()),
    ];
    arguments.extend(options.iter().map(|option| (*option).to_owned()));
    arguments.push(source.display().to_string());
    common::run_verus_raw(
        &arguments.iter().map(String::as_str).collect::<Vec<_>>(),
        crates.directory.path(),
    )
}

#[test]
fn erased_check_only_failure_withholds_fresh_and_seeded_exports() {
    for body in [HELPER, ORDINARY] {
        let crates = InferCrates::new();
        let name = "erased_failure";
        for seeded in [false, true] {
            if seeded {
                crates.succeeds(name, body, &[]);
            }
            let run = raw_publication(
                &crates,
                name,
                &format!("{body}\n{NATIVE_FAILURE}"),
                &crates.vir(name),
                &[],
            );
            assert_native_failure(&crates, name, &run);
        }
    }
}

#[test]
fn late_export_failure_preserves_json_diagnostics_and_withholds_publication() {
    let crates = InferCrates::new();
    let export = crates.directory.path().join("missing-directory/result.vir");
    let run = raw_publication(
        &crates,
        "late_export_failure",
        HELPER,
        &export,
        &["--no-erasure-check", "--error-format=json"],
    );
    assert!(!run.status.success());
    assert!(!export.exists());
    let report = json_inference(&run);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
    let diagnostics: Vec<serde_json::Value> = String::from_utf8_lossy(&run.stderr)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("JSON diagnostic on every stderr line"))
        .collect();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["level"] == "error"
            && diagnostic["message"].as_str().unwrap_or("").contains("create export")
    }));
    for function in report["functions"].as_array().unwrap() {
        assert_eq!(function["exported"], false);
        for obligation in ["soundness", "maximality", "exactness"] {
            assert_eq!(function["proofs"][obligation]["status"], "proved");
        }
    }
}

#[test]
fn direct_compilation_preserves_json_artifact_notifications() {
    let crates = InferCrates::new();
    let run = raw_publication(
        &crates,
        "direct_artifacts",
        ORDINARY,
        &crates.vir("direct_artifacts"),
        &["--compile", "--emit=metadata,link", "--error-format=json", "--json=artifacts"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(crates.vir("direct_artifacts").exists());
    assert!(String::from_utf8_lossy(&run.stderr).lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line).is_ok_and(|message| {
            message["$message_type"] == "artifact" && message["emit"] == "metadata"
        })
    }));
}

#[test]
fn native_failure_withholds_fresh_and_previously_seeded_inferred_exports() {
    let crates = InferCrates::new();
    let name = "native_failure";
    for seed in [None, Some(HELPER), Some("pub fn ordinary(value:u8)->u8 {value}")] {
        if let Some(source) = seed {
            crates.succeeds(name, source, &[]);
            assert!(crates.vir(name).exists());
        }
        let run = crates.build(
            name,
            &format!("{HELPER}\n{NATIVE_FAILURE}"),
            &[],
            &["--output-json", "--no-cheating"],
        );
        assert_native_failure(&crates, name, &run);
        assert_eq!(json_inference(&run)["functions"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn native_failure_withholds_import_only_and_goal_assisted_exports() {
    let crates = InferCrates::new();
    crates.succeeds("publication_provider", HELPER, &[]);
    let caller = r#"
        pub fn caller(value:u8)->(result:u8)
            requires value<255,
            ensures result as int==value as int+1,
        {publication_provider::increment(value)}
    "#;
    crates.succeeds("publication_consumer", caller, &["publication_provider"]);
    let run = crates.build(
        "publication_consumer",
        &format!("{caller}\n{NATIVE_FAILURE}"),
        &["publication_provider"],
        &["--output-json", "--no-cheating"],
    );
    assert_native_failure(&crates, "publication_consumer", &run);
    assert!(
        !json_inference(&run)["imported_inference_dependencies"].as_array().unwrap().is_empty()
    );
    let goal = "pub fn goal(value:u8)->u8 {value}";
    let seed = crates.build("publication_goal", goal, &[], &["--infer-goals"]);
    assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    assert!(crates.vir("publication_goal").exists());
    let run = crates.build(
        "publication_goal",
        &format!("{goal}\n{NATIVE_FAILURE}"),
        &[],
        &["--output-json", "--no-cheating", "--infer-goals"],
    );
    assert_native_failure(&crates, "publication_goal", &run);
}

#[test]
fn successful_native_and_verification_only_runs_still_publish() {
    let crates = InferCrates::new();
    let run = crates.build("native_success", HELPER, &[], &["--output-json", "--no-cheating"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(json_inference(&run)["interface_written"], true);
    assert!(std::fs::read(crates.vir("native_success")).unwrap().ends_with(b"VRINFER1"));
    let run = crates.verify("verification_success", HELPER, &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(json_inference(&run)["interface_written"], true);
    assert!(std::fs::read(crates.vir("verification_success")).unwrap().ends_with(b"VRINFER1"));
}

#[test]
fn ordinary_proof_failures_withhold_fresh_and_seeded_exports_in_both_modes() {
    for enabled in [false, true] {
        for compile in [false, true] {
            let crates = InferCrates::new();
            let name = "ordinary_failure";
            let mut options = vec!["--output-json", "--no-cheating"];
            if enabled {
                options.push("--infer-goals");
            }
            for seeded in [false, true] {
                if seeded {
                    crates.succeeds(name, ORDINARY, &[]);
                    assert!(crates.vir(name).exists());
                }
                let run = if compile {
                    crates.build(name, ORDINARY_FAILURE, &[], &options)
                } else {
                    crates.verify(name, ORDINARY_FAILURE, &[], &options)
                };
                assert!(!run.status.success());
                assert!(String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
                assert!(!crates.vir(name).exists());
                assert_eq!(json_inference(&run)["interface_written"], false);
            }
        }
    }
}

#[test]
fn ordinary_early_failures_remove_legacy_inferred_and_malformed_seeds() {
    for (source, diagnostic) in
        [("pub fn broken()->u8 {false}", "mismatched types"), ("pub fn broken(", "error")]
    {
        let crates = InferCrates::new();
        let name = "ordinary_early_failure";
        for seed in [Some(ORDINARY), Some(HELPER), None] {
            if let Some(seed) = seed {
                crates.succeeds(name, seed, &[]);
            } else {
                std::fs::write(crates.vir(name), b"unmarked malformed previous export").unwrap();
            }
            assert!(crates.vir(name).exists());
            let run = crates.build(name, source, &[], &["--no-cheating"]);
            assert!(!run.status.success());
            assert!(String::from_utf8_lossy(&run.stderr).contains(diagnostic));
            assert!(!crates.vir(name).exists());
        }
    }
}

#[test]
fn ordinary_native_failures_withhold_fresh_and_seeded_exports() {
    for enabled in [false, true] {
        let crates = InferCrates::new();
        let name = "ordinary_native_failure";
        let mut options = vec!["--output-json", "--no-cheating"];
        if enabled {
            options.push("--infer-goals");
        }
        for seeded in [false, true] {
            if seeded {
                crates.succeeds(name, ORDINARY, &[]);
            }
            let run = crates.build(name, &format!("{ORDINARY}\n{NATIVE_FAILURE}"), &[], &options);
            assert_native_failure(&crates, name, &run);
        }
    }
}

#[test]
fn ordinary_success_keeps_legacy_format_and_imports_normally() {
    let crates = InferCrates::new();
    crates.succeeds("ordinary_provider", ORDINARY, &[]);
    assert!(!std::fs::read(crates.vir("ordinary_provider")).unwrap().ends_with(b"VRINFER1"));
    let caller = "pub fn call(value:u8)->(result:u8) ensures result==value {ordinary_provider::identity(value)}";
    let run = crates.succeeds("ordinary_consumer", caller, &["ordinary_provider"]);
    assert!(run.status.success());
    assert!(!std::fs::read(crates.vir("ordinary_consumer")).unwrap().ends_with(b"VRINFER1"));
    let run = crates.verify("ordinary_verify_only", ORDINARY, &[], &["--no-cheating"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(!std::fs::read(crates.vir("ordinary_verify_only")).unwrap().ends_with(b"VRINFER1"));
    let run = crates.build("ordinary_explicit_no_verify", ORDINARY_FAILURE, &[], &["--no-verify"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(
        !std::fs::read(crates.vir("ordinary_explicit_no_verify")).unwrap().ends_with(b"VRINFER1")
    );
}
