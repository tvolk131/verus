#![feature(rustc_private)]
mod common;
use common::*;

const BOOL: &str = r#"
use vstd::prelude::*;
verus! {
pub fn all_bool(b:bool)->(r:u64)
    ensures r == if b {1u64} else {0u64},
{ u64::from(b) }
}
"#;

#[test]
fn ordinary_full_domain_checked_body_and_false_caller() {
    for _ in 0..2 {
        let r = verify_one_file(
            "all_bool",
            BOOL.into(),
            &["vstd", "--infer-goals", "--no-cheating", "--compile"],
        );
        assert!(r.is_ok(), "{:?}", r);
    }
    let e = verify_one_file(
        "false_caller",
        BOOL.replace("r == if b {1u64} else {0u64}", "r == 0u64"),
        &["vstd", "--infer-goals", "--no-cheating", "--compile"],
    )
    .unwrap_err();
    assert_eq!(e.errors.len(), 1, "{:?}", e);
    assert!(e.errors[0].message.contains("postcondition not satisfied"), "{:?}", e);
}

#[test]
fn goals_off_keeps_inherited_behavior() {
    let e =
        verify_one_file("off", BOOL.into(), &["vstd", "--no-cheating", "--compile"]).unwrap_err();
    assert_eq!(e.errors.len(), 1, "{:?}", e);
    assert!(e.errors[0].message.contains("postcondition not satisfied"), "{:?}", e);
}

#[test]
fn incomplete_checks_refuse() {
    for option in ["--no-verify", "--no-lifetime", "--no-erasure-check"] {
        let e = verify_one_file(
            "incomplete",
            BOOL.into(),
            &["vstd", "--infer-goals", "--no-cheating", option],
        )
        .unwrap_err();
        assert!(
            e.errors.iter().any(|e| e.message.contains("complete source/body")),
            "{}: {:?}",
            option,
            e
        );
    }
}

#[test]
fn generic_packing_proposals_check_their_premises() {
    let source = r#"
use vstd::prelude::*;
verus! {
fn pack(low:u64,b:bool)->(r:bool)
    requires low < 0x8000000000000000u64,
    ensures r==b,
{
    let w=low | (u64::from(b)<<63u64);
    w & 0x8000000000000000u64 > 0
}
}
"#;
    let r = verify_one_file(
        "pack",
        source.into(),
        &["vstd", "--infer-goals", "--no-cheating", "--compile"],
    );
    assert!(r.is_ok(), "{:?}", r);
    let e = verify_one_file(
        "unbounded_pack",
        source.replace("requires low < 0x8000000000000000u64,", ""),
        &["vstd", "--infer-goals", "--no-cheating", "--compile"],
    )
    .unwrap_err();
    assert!(e.errors.iter().any(|e| e.message.contains("postcondition not satisfied")), "{:?}", e);
}

#[test]
fn incomplete_foreign_export_refuses_and_removes_legacy_output() {
    use common::infer::InferCrates;
    let crates = InferCrates::new();
    for options in [
        vec!["--no-verify"],
        vec!["--no-lifetime"],
        vec!["--no-trait-conflicts"],
        vec!["--no-erasure-check"],
        vec!["--verify-root", "--verify-function", "all_bool"],
    ] {
        let run = |source: &str, options: &[&str]| {
            let file = crates.directory.path().join("component.rs");
            std::fs::write(&file, source).unwrap();
            let mut command = std::process::Command::new(crates.profile.join("verus"));
            command.arg(file).args(["--crate-type=lib", "--crate-name=component", "--no-cheating"]);
            if !options.contains(&"--no-erasure-check") {
                command.arg("--compile");
            }
            command
                .arg("--export")
                .arg(crates.vir("component"))
                .arg("-o")
                .arg(crates.directory.path().join("libcomponent.rlib"))
                .args(options)
                .output()
                .unwrap()
        };
        let seed = run("use vstd::prelude::*; verus! {pub fn seed()->u64 {0}}", &[]);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        assert!(crates.vir("component").exists());
        let mut args = vec!["--infer-goals"];
        args.extend(options);
        let r = run(BOOL, &args);
        assert!(!r.status.success());
        assert!(
            String::from_utf8_lossy(&r.stderr).contains("complete source/body"),
            "{}",
            String::from_utf8_lossy(&r.stderr)
        );
        assert!(!crates.vir("component").exists());
    }
}
