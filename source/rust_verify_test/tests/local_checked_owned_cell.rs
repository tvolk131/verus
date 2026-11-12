//! Inactive preflight controls, NOT the design's native proof acceptance gates.
//! Refusing every cell body cannot satisfy its positive/semantic acceptance.
#![feature(rustc_private)]
mod common;
use common::infer::InferCrates;

const FROZEN: &str =
    include_str!("../fixtures/infer/native_cell/owned_goals.rs");
const REQUEST: &str = "--experimental-owned-unsafe-cell";

fn run(source: &str, options: &[&str]) -> std::process::Output {
    let c = InferCrates::new();
    let file = c.directory.path().join("case.rs");
    std::fs::write(&file, source).unwrap();
    std::process::Command::new(c.profile.join("verus"))
        .arg(file)
        .args(["--crate-type=lib", "--crate-name=case", "--edition=2024"])
        .arg("-o")
        .arg(c.directory.path().join("libcase.rlib"))
        .args(options)
        .env("VERUS_INFER_CACHE", c.directory.path().join("infer-cache"))
        .output()
        .unwrap()
}
fn refused(r: &std::process::Output, reason: &str) {
    assert!(!r.status.success(), "unexpected acceptance");
    assert!(
        String::from_utf8_lossy(&r.stderr).contains(reason),
        "{}",
        String::from_utf8_lossy(&r.stderr)
    );
}

#[test]
fn default_off_preserves_scalar_checks_in_both_modes() {
    let good = "use vstd::prelude::*; verus! {pub fn scalar(v:u64)->(r:u64) ensures r==v, {v}}";
    for infer in [false, true] {
        let mut opts = vec!["--no-cheating", "--compile"];
        if infer {
            opts.push("--infer-goals");
        }
        let r = run(good, &opts);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        refused(&run(&good.replace("r==v", "r!=v"), &opts), "postcondition not satisfied");
    }
}

#[test]
fn frozen_native_bodies_remain_inactive_in_both_modes() {
    for _ in 0..2 {
        for infer in [false, true] {
            let mut opts = vec![REQUEST, "--no-cheating", "--compile"];
            if infer {
                opts.push("--infer-goals");
            }
            refused(&run(FROZEN, &opts), "experimental owned UnsafeCell: inactive:");
        }
    }
}

#[test]
fn request_cannot_disable_checks_or_select_partial_verification() {
    for infer in [false, true] {
        for extra in [
            vec!["--no-verify"],
            vec!["--no-lifetime"],
            vec!["--no-trait-conflicts"],
            vec!["--no-erasure-check"],
            vec!["--verify-root"],
            vec!["--verify-module", "some_module"],
            vec!["--verify-only-module", "some_module"],
            vec!["--verify-root", "--verify-function", "owned_roundtrip"],
        ] {
            let mut opts = vec![REQUEST, "--no-cheating"];
            if infer {
                opts.push("--infer-goals");
            }
            opts.extend(extra);
            refused(
                &run(FROZEN, &opts),
                "requires --no-cheating and full body/lifetime/trait/erasure",
            );
        }
        let mut opts = vec![REQUEST];
        if infer {
            opts.push("--infer-goals");
        }
        refused(&run(FROZEN, &opts), "requires --no-cheating");
    }
}

#[test]
fn local_only_request_invalidates_seeded_export_even_without_a_cell_or_helper() {
    let c = InferCrates::new();
    let source = c.directory.path().join("empty.rs");
    std::fs::write(&source, "use vstd::prelude::*; verus! {pub fn scalar()->u64 {0}}").unwrap();
    for infer in [false, true] {
        let artifact = c.vir("empty");
        std::fs::write(&artifact, b"stale ordinary or experimental metadata").unwrap();
        let mut cmd = std::process::Command::new(c.profile.join("verus"));
        cmd.arg(&source)
            .args(["--crate-type=lib", "--crate-name=empty", "--no-cheating", REQUEST, "--export"])
            .arg(&artifact);
        if infer {
            cmd.arg("--infer-goals");
        }
        cmd.env("VERUS_INFER_CACHE", c.directory.path().join(format!("export-cache-{infer}")));
        let r = cmd.output().unwrap();
        refused(&r, "local-only; selected VIR export is unsupported");
        assert!(!artifact.exists());
    }
}

#[test]
fn cargo_and_explicit_outputs_are_both_invalidated_on_refusal_and_parse_failure() {
    let c = InferCrates::new();
    let source = c.directory.path().join("cargo_case.rs");
    let explicit = c.vir("explicit");
    let cargo = c.vir("libcargo");
    for infer in [false, true] {
        for malformed in [false, true] {
            std::fs::write(
                &source,
                if malformed { "pub fn broken( {" } else { "pub fn scalar()->u64 {0}" },
            )
            .unwrap();
            std::fs::write(&explicit, b"stale explicit metadata").unwrap();
            std::fs::write(&cargo, b"stale cargo metadata").unwrap();
            let mut cmd = std::process::Command::new(c.profile.join("rust_verify"));
            cmd.arg("rustc")
                .arg(&source)
                .args([
                    "--crate-name",
                    "cargo_case",
                    "--crate-type=lib",
                    "--emit=metadata",
                    "--edition=2024",
                    "--no-cheating",
                    REQUEST,
                    "--export",
                ])
                .arg(&explicit)
                .arg("-o")
                .arg(c.directory.path().join("libcargo.rmeta"))
                .env("__VERUS_DRIVER_VIA_CARGO__", "1")
                .env("CARGO_PKG_NAME", "owned_cell_preflight")
                .env("CARGO_PKG_VERSION", "0.0.0")
                .env("CARGO_MANIFEST_DIR", "/owned-cell-preflight-fixture")
                .env("__VERUS_DRIVER_VERIFY_owned_cell_preflight-0.0.0-af08914f15a5", "1");
            if infer {
                cmd.arg("--infer-goals");
            }
            cmd.env("VERUS_INFER_CACHE", c.directory.path().join(format!("export-cache-{infer}")));
            let r = cmd.output().unwrap();
            if malformed {
                refused(&r, "error");
            } else {
                refused(&r, "local-only; selected VIR export is unsupported");
            }
            assert!(
                !explicit.exists() && !cargo.exists(),
                "both selected paths must be cleared: {}",
                String::from_utf8_lossy(&r.stderr)
            );
        }
    }
}
