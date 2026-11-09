#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, json_inference};
use common::*;

test_verify_one_file! {
    #[test] common_runtime_smoke code! {
        verus! { fn checked(x: u64) -> (r: u64) ensures r == x, { x } }
    } => Ok(())
}

const GENERIC: &str = r#"
use vstd::prelude::*;
verus! {
pub open spec fn observed(values: Seq<u16>, position: int) -> int { values[position] as int % 3 }
pub fn repaint(values: &mut Vec<u16>, chosen: u16)
    ensures final(values)@.len() == old(values)@.len(),
        forall|position: int| 0 <= position < final(values)@.len() ==>
            #[trigger] observed(final(values)@, position) == chosen as int % 3,
{
    let ghost original = values@;
    let mut cursor: usize = 0;
    loop
        invariant
            cursor <= values@.len(), values@.len() == original.len(),
            forall|position: int| 0 <= position < values@.len() ==>
                #[trigger] observed(values@, position) ==
                    if position < cursor { chosen as int % 3 } else { observed(original, position) },
        ensures cursor == values@.len(),
        decreases values@.len() - cursor,
    {
        if cursor == values.len() { break; }
        let ghost before = values@;
        values.set(cursor, chosen);
        assert forall|position: int| 0 <= position < values@.len() implies
            #[trigger] observed(values@, position) ==
                if position <= cursor { chosen as int % 3 } else { observed(original, position) }
        by {
            if position == cursor as int { } else { }
        }
        cursor += 1;
    }
}
}
"#;

fn run(source: &str, enabled: bool, success: bool) {
    let crates = InferCrates::new();
    let mut options = vec!["--no-cheating", "--output-json", "--num-threads", "1"];
    if enabled {
        options.push("--infer-goals");
    }
    let result = crates.build_source("loop_case", source, &[], &options, true);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!stderr.contains("panicked"), "{}\n{}", stderr, stdout);
    assert_eq!(result.status.success(), success, "{}\n{}", stderr, stdout);
    let report = json_inference(&result);
    if enabled && success {
        assert!(
            report["loop_point_preparation"]
                .as_array()
                .unwrap()
                .iter()
                .any(|decision| decision["prepared_instances"].as_u64().unwrap_or(0) > 0),
            "{}",
            report
        );
    }
    if !success {
        assert!(stderr.contains("assertion failed"), "{}\n{}", stderr, stdout);
        assert!(!crates.vir("loop_case").exists());
    }
}

#[test]
fn generic_typed_loop_point() {
    run(GENERIC, true, true);
}

#[test]
fn renamed_typed_loop_point() {
    run(
        &GENERIC
            .replace("repaint", "advance")
            .replace("values", "storage")
            .replace("cursor", "offset")
            .replace("position", "query"),
        true,
        true,
    );
}

#[test]
fn feature_off_retains_original_point_failure() {
    run(GENERIC, false, false);
}

#[test]
fn wrong_write_does_not_become_an_assumption() {
    run(
        &GENERIC.replace("values.set(cursor, chosen)", "values.set(cursor, chosen ^ 1)"),
        true,
        false,
    );
}

#[test]
fn stale_snapshot_does_not_describe_the_second_write() {
    run(
        &GENERIC.replace(
            "values.set(cursor, chosen);",
            "values.set(cursor, chosen); values.set(cursor, chosen ^ 1);",
        ),
        true,
        false,
    );
}

#[test]
fn missing_range_guards_and_wrong_frame_still_fail() {
    run(
        &GENERIC.replace(
            "assert forall|position: int| 0 <= position < values@.len() implies",
            "assert forall|position: int| position < values@.len() implies",
        ),
        true,
        false,
    );
    run(
        &GENERIC.replace(
            "assert forall|position: int| 0 <= position < values@.len() implies",
            "assert forall|position: int| 0 <= position implies",
        ),
        true,
        false,
    );
    run(
        &GENERIC.replace(
            "values.set(cursor, chosen);",
            "values.set(cursor, chosen); if cursor + 1 < values.len() { values.set(cursor + 1, chosen ^ 1); }",
        ),
        true,
        false,
    );
}

#[test]
fn assisted_ordinary_repeats_and_imports_without_inferred_exports() {
    let crates = InferCrates::new();
    let options = ["--no-cheating", "--output-json", "--infer-goals", "--num-threads", "1"];
    for _ in 0..2 {
        let result = crates.build_source("loop_provider", GENERIC, &[], &options, true);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let report = json_inference(&result);
        assert_eq!(report["requested_functions"], 0);
        assert_eq!(report["generated_functions"], 0);
        assert!(report["imported_inference_dependencies"].as_array().unwrap().is_empty());
        assert!(
            report["loop_point_preparation"]
                .as_array()
                .unwrap()
                .iter()
                .any(|decision| { decision["prepared_instances"].as_u64().unwrap_or(0) > 0 })
        );
        assert!(report["functions"].as_array().unwrap().is_empty());
    }
    let consumer = r#"
use vstd::prelude::*;
verus! {
pub fn use_checked(values: &mut Vec<u16>, chosen: u16)
    ensures forall|position: int| 0 <= position < final(values)@.len() ==>
        #[trigger] loop_provider::observed(final(values)@, position) == chosen as int % 3,
{ loop_provider::repaint(values, chosen); }
}
"#;
    let result = crates.build_source("loop_consumer", consumer, &["loop_provider"], &options, true);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report = json_inference(&result);
    assert_eq!(report["requested_functions"], 0);
    assert_eq!(report["generated_functions"], 0);
    assert!(report["functions"].as_array().unwrap().is_empty());
    let metadata = ordinary_envelope(&crates.vir("loop_provider"));
    let certificate = &metadata["certificate"];
    assert_eq!(certificate["schema"], 38);
    assert_eq!(certificate["policy"], "candidate-bound-evidence-v76");
    assert!(!certificate["producer"].as_str().unwrap().is_empty());
    for guarantee in ["soundness", "maximality", "exactness"] {
        assert_eq!(certificate[guarantee], true);
    }
    for empty in ["proofs", "dependencies", "trusted_models"] {
        assert!(certificate[empty].as_array().unwrap().is_empty());
    }
    assert_eq!(
        report["imported_inference_dependencies"],
        serde_json::json!([{
            "crate_id": metadata["crate_id"], "fingerprint": metadata["fingerprint"]
        }])
    );
    crates.edit_metadata("loop_provider", |certificate| certificate["soundness"] = false.into());
    let rejected =
        crates.build_source("loop_consumer", consumer, &["loop_provider"], &options, true);
    assert!(!rejected.status.success());
    assert!(!crates.vir("loop_consumer").exists());
    assert!(!String::from_utf8_lossy(&rejected.stderr).contains("panicked"));
}

fn ordinary_envelope(path: &std::path::Path) -> serde_json::Value {
    let script = r#"
import hashlib, json, sys
from pathlib import Path
raw = Path(sys.argv[1]).read_bytes()
if not raw.endswith(b'VRINFER1'):
    raise ValueError('missing ordinary proof envelope')
start = raw.rindex(b'{"schema":')
certificate = json.loads(raw[start:-8])
prefix_hash = hashlib.sha256()
prefix_length = None
for position in range(start):
    prefix_hash.update(raw[position:position + 1])
    if prefix_hash.hexdigest() == certificate['summaries']:
        prefix_length = position + 1
        break
if prefix_length is None or prefix_length + 8 > start:
    raise ValueError('unbound ordinary logical payload')
print(json.dumps(dict(certificate=certificate,
    crate_id=int.from_bytes(raw[prefix_length:prefix_length + 8], 'little'),
    fingerprint=hashlib.sha256(raw[start:-8]).hexdigest())))
"#;
    let result = std::process::Command::new("python3")
        .args(["-B", "-c", script])
        .arg(path)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn assisted_failure_withdraws_known_vir_but_does_not_refresh_stale_rlib() {
    for seeded in [false, true] {
        let crates = InferCrates::new();
        let options = ["--no-cheating", "--output-json", "--infer-goals"];
        let library = crates.directory.path().join("libloop_provider.rlib");
        let previous = if seeded {
            let result = crates.build_source("loop_provider", GENERIC, &[], &options, true);
            assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
            assert!(crates.vir("loop_provider").exists());
            Some(std::fs::read(&library).unwrap())
        } else {
            None
        };
        let failure = GENERIC.replace(
            "let ghost original = values@;",
            "assert(false); let ghost original = values@;",
        );
        let result = crates.build_source("loop_provider", &failure, &[], &options, true);
        assert!(!result.status.success());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "{}", stderr);
        assert!(stderr.contains("assertion failed"), "{}", stderr);
        assert!(!crates.vir("loop_provider").exists());
        assert_eq!(json_inference(&result)["interface_written"], false);
        if let Some(previous) = previous {
            assert_eq!(std::fs::read(&library).unwrap(), previous);
        } else {
            assert!(!library.exists());
        }
        let result = crates.build_source(
            "failed_import",
            "use loop_provider::*;",
            &["loop_provider"],
            &options,
            true,
        );
        assert!(!result.status.success());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
        assert!(!crates.vir("failed_import").exists());
    }
}

#[test]
fn condition_setup_writes_do_not_capture_post_effect_values() {
    run(
        &GENERIC.replace(
            "if cursor == values.len() { break; }",
            "if 0 < values.len() { values.set(0, chosen ^ 1); } if cursor == values.len() { break; }",
        ),
        true,
        false,
    );
}

#[test]
fn original_executable_safety_goals_are_still_checked() {
    let crates = InferCrates::new();
    let unsafe_source =
        GENERIC.replace("values.set(cursor, chosen)", "values.set(values.len(), chosen)");
    let result = crates.build_source(
        "unsafe_index",
        &unsafe_source,
        &[],
        &["--no-cheating", "--output-json", "--infer-goals"],
        true,
    );
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!stderr.contains("panicked"), "{}", stderr);
    assert!(stderr.contains("precondition not satisfied"), "{}", stderr);
    assert!(!crates.vir("unsafe_index").exists());
}
