use super::*;

const REVERSE: &str = r#"use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone)] pub struct Quantity(pub u8);
verus! {
 impl Quantity { pub open spec fn __verus_infer_derived_into(&self)->bool {true} }
 impl vstd::std_specs::convert::FromSpecImpl<Quantity> for u8 {
  open spec fn obeys_from_spec()->bool {true}
  open spec fn from_spec(value:Quantity)->Self {value.0}
 }
}
#[automatically_derived]
impl From<Quantity> for u8 {fn from(value:Quantity)->Self {u8::from(value.0)}}
#[verus_infer] pub fn unwrap(value:Quantity)->u8 {value.into()}
"#;

#[test]
fn reverse_generated_conversion_reproves_across_crates_and_blocks_failed_exports() {
    let crates = InferCrates::new();
    let client = r#"use vstd::prelude::*;use reverse_provider::Quantity;
verus!{pub fn goal(n:u8)->(r:u8) ensures r==n, {reverse_provider::unwrap(Quantity(n))}}
"#;
    for _ in 0..2 {
        let run = crates.build_source(
            "reverse_provider",
            REVERSE,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let fs = json_inference(&run)["functions"].as_array().unwrap().clone();
        assert_eq!(fs.len(), 2);
        assert!(fs.iter().all(|f| {
            f["required_checks_passed"] == true
                && ["soundness", "maximality", "exactness"]
                    .iter()
                    .all(|p| f["proofs"][p]["status"] == "proved")
        }));
        let run = crates.build_source(
            "reverse_client",
            client,
            &["reverse_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
    let run = crates.build_source(
        "reverse_client",
        &client.replace("ensures r==n", "ensures r as int==n as int+1"),
        &["reverse_provider"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("reverse_client").exists());
    for options in [
        vec!["--no-verify", "--output-json"],
        vec!["--verify-root", "--verify-function", "unwrap", "--output-json"],
    ] {
        let run = crates.build_source("reverse_provider", REVERSE, &[], &options, true);
        assert!(!run.status.success());
        assert!(!crates.vir("reverse_provider").exists());
        assert!(String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"));
    }
    for body in ["0", "value.0 - 1"] {
        let changed = REVERSE.replace("u8::from(value.0)", body);
        let run = crates.build_source(
            "reverse_provider",
            &changed,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!run.status.success(), "wrong conversion unexpectedly verified");
        assert!(!crates.vir("reverse_provider").exists());
    }
}

#[test]
fn a_reverse_request_without_its_checked_native_impl_cannot_publish() {
    let crates = InferCrates::new();
    // Rust still has a From impl, so the extension trait is well-formed. It is
    // deliberately outside verification and cannot satisfy the proof request.
    let missing = REVERSE
        .replace("#[automatically_derived]\n", "")
        .replace("#[verus_infer] pub fn unwrap(value:Quantity)->u8 {value.into()}", "");
    let run = crates.build_source(
        "reverse_missing",
        &missing,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("reverse_missing").exists());
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("derived trait request requires one checked implementation")
    );
}

#[test]
fn a_forward_request_does_not_implicitly_request_reverse_inference() {
    let crates = InferCrates::new();
    let source=REVERSE.replace("__verus_infer_derived_into","__verus_infer_derived_from")
        .replace("FromSpecImpl<Quantity> for u8","FromSpecImpl<u8> for Quantity")
        .replace("from_spec(value:Quantity)->Self {value.0}","from_spec(value:u8)->Self {Quantity(value)}")
        .replace("#[verus_infer] pub fn unwrap(value:Quantity)->u8 {value.into()}",
            "#[automatically_derived] impl From<u8> for Quantity {fn from(n:u8)->Self{Quantity(n)}}");
    let run = crates.build_source(
        "only_forward",
        &source,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    // Only the requested forward contract is inferred. The legacy unrequested
    // reverse derive remains outside scope and supplies no usable contract.
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(json_inference(&run)["functions"].as_array().unwrap().len(), 1);
    assert_eq!(json_inference(&run)["functions"][0]["required_checks_passed"], true);
    let called =
        format!("{}\n#[verus_infer] pub fn unapproved(n:Quantity)->u8{{n.into()}}", source);
    let run = crates.build_source(
        "only_forward",
        &called,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("only_forward").exists());
    let scoped = source.replace("#[verus_verify]", "#[verus_verify(external_derive(Into))]");
    let run = crates.build_source(
        "only_forward",
        &scoped,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(json_inference(&run)["functions"].as_array().unwrap().len(), 1);
}

#[test]
fn primitive_identity_conversions_keep_all_integer_domains() {
    let crates = InferCrates::new();
    for ty in
        ["u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize"]
    {
        let source = format!(
            r#"use vstd::prelude::*;
#[verus_infer] pub fn identity(n:{ty})->{ty} {{<{ty} as From<{ty}>>::from(n)}}
verus!{{pub fn goal(n:{ty})->(r:{ty}) ensures r==n, {{identity(n)}}}}
"#
        );
        let run = crates.build_source(
            "integer_identity",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}: {}", ty, String::from_utf8_lossy(&run.stderr));
        assert_eq!(json_inference(&run)["functions"][0]["required_checks_passed"], true);
    }
}
