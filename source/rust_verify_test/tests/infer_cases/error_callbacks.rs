use super::*;

const PROVIDER: &str = r#"use vstd::prelude::*;
pub struct Snapshot<T> { pub value: T }
fn copy_snapshot<T: Clone>(source: &Snapshot<T>) -> (result: Snapshot<T>)
 ensures cloned(source.value, result.value),
{ Snapshot { value: source.value.clone() } }
pub enum Error<T> { Missing, Context(Snapshot<T>, u8) }
pub fn wrap<T: Clone>(source: &Snapshot<T>, input: Result<u16, u8>) -> (result: Result<u16, Error<T>>)
 ensures match input {
  Ok(value) => matches!(result, Ok(output) if output == value),
  Err(code) => match result {
   Err(Error::Context(payload, output)) => cloned(source.value, payload.value) && output == code,
   _ => false,
  },
 },
{ input.map_err(|code| Error::Context(copy_snapshot(source), code)) }
"#;

#[test]
fn ordinary_error_callbacks_compose_checked_callee_relations_across_crates() {
    let crates = InferCrates::new();
    let client = r#"use vstd::prelude::*;
pub fn goal<T: Clone>(source: &callback_provider::Snapshot<T>, code: u8) {
 let result = callback_provider::wrap(source, Err(code));
 match result {
  Err(callback_provider::Error::Context(payload, output)) => {
   assert(cloned(source.value, payload.value)); assert(output == code);
  },
  _ => assert(false),
 }
}
"#;
    for _ in 0..2 {
        let p = crates.build(
            "callback_provider",
            PROVIDER,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
        );
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        let report = json_inference(&p);
        assert!(report["functions"].as_array().unwrap().is_empty());
        assert_eq!(report["error_callback_assistance"].as_array().unwrap().len(), 1);
        let c = crates.build(
            "callback_client",
            client,
            &["callback_provider"],
            &["--infer-goals", "--no-cheating"],
        );
        assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
    }
    for wrong in [
        PROVIDER.replace("copy_snapshot(source), code)", "copy_snapshot(source), 0)"),
        PROVIDER.replace("Error::Context(copy_snapshot(source), code)", "Error::Missing"),
        PROVIDER.replace("copy_snapshot(source), code)", "copy_snapshot(source), code + 1)"),
    ] {
        assert_ne!(wrong, PROVIDER);
        let p = crates.build("callback_provider", &wrong, &[], &["--infer-goals", "--no-cheating"]);
        assert!(!p.status.success());
        assert!(!crates.vir("callback_provider").exists());
        assert!(!String::from_utf8_lossy(&p.stderr).contains("panicked"));
    }
    for options in [
        vec!["--infer-goals", "--no-verify"],
        vec!["--infer-goals", "--verify-root", "--verify-function", "wrap"],
    ] {
        // Ordinary authored contracts follow ordinary Verus filtering rules.
        // A crate publishing inferred guarantees must still pass all checks.
        let guarded = format!("{}\n#[verifier::infer] fn identity(x: u8) -> u8 {{ x }}", PROVIDER);
        let p = crates.build("callback_provider", &guarded, &[], &options);
        assert!(!p.status.success());
        assert!(!crates.vir("callback_provider").exists());
        assert!(String::from_utf8_lossy(&p.stderr).contains("requires full-crate verification"));
    }
}

#[test]
fn error_callback_assistance_declines_partial_callees() {
    let crates = InferCrates::new();
    let provider = r#"use vstd::prelude::*;
pub enum Error { Context(u8) }
fn partial(code: u8) -> (result: u8)
 requires code > 0,
 ensures result == code,
{ code }
pub fn wrap(input: Result<u8, u8>) -> (result: Result<u8, Error>)
 ensures match input {
  Ok(value) => matches!(result, Ok(output) if output == value),
  Err(code) => matches!(result, Err(Error::Context(output)) if output == code),
 },
{ input.map_err(|code| Error::Context(partial(code))) }
"#;
    let p = crates.build(
        "partial_callback",
        provider,
        &[],
        &["--infer-goals", "--no-cheating", "--output-json"],
    );
    assert!(!p.status.success());
    assert!(!crates.vir("partial_callback").exists());
    assert!(json_inference(&p)["error_callback_assistance"].as_array().unwrap().is_empty());
    assert!(String::from_utf8_lossy(&p.stderr).contains("precondition not satisfied"));
    assert!(!String::from_utf8_lossy(&p.stderr).contains("panicked"));
}
