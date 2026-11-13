#![feature(rustc_private)]
#[macro_use]
mod common;
fn checked(name: &str, source: String, count: usize) {
    let e = common::verify_one_file(
        name,
        source,
        &["vstd", "--infer-goals", "--no-cheating", "--output-json"],
    )
    .unwrap();
    common::infer::assert_exact_report(&e.json_output.unwrap()["verus-infer"], count);
}
use common::*;

const BASE: &str = r#"use vstd::prelude::*;
use std::num::NonZero;
#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorTag(NonZero<u64>);
"#;

#[test]
fn original_derived_ordering_full_domain_and_shared_aliases() {
    checked(
        "nonzero_order_full",
        format!(
            r#"{BASE}
verus! {{fn goal(a:&BorTag,b:&BorTag) {{
 let c=a.cmp(b);let p=a.partial_cmp(b);
 assert(c==vstd::std_specs::cmp::OrdSpec::cmp_spec(&a.0@,&b.0@));
 assert(p==Some(c));
 let same=a.cmp(a);let same_p=a.partial_cmp(a);
 assert(same==core::cmp::Ordering::Equal);
 assert(same_p==Some(core::cmp::Ordering::Equal));
 assert(a.0@==u64::MAX && b.0@==1u64 ==> c==core::cmp::Ordering::Greater);
 assert(a.0@==9223372036854775808u64 && b.0@==1u64 ==> c==core::cmp::Ordering::Greater);
}}}}
"#
        ),
        1,
    );
}

const CANDIDATES: &str = r#"
#[verus_infer] fn below(a:&BorTag,b:&BorTag)->bool {a<b}
#[verus_infer] fn at_most(a:&BorTag,b:&BorTag)->bool {a<=b}
#[verus_infer] fn above(a:&BorTag,b:&BorTag)->bool {a>b}
#[verus_infer] fn at_least(a:&BorTag,b:&BorTag)->bool {a>=b}
"#;
#[test]
fn inherited_boolean_calls_have_all_three_exact_checks() {
    checked(
        "nonzero_order_candidates",
        format!(
            r#"{BASE}{CANDIDATES}
verus! {{fn goal(a:&BorTag,b:&BorTag) {{
 let lt=below(a,b);let le=at_most(a,b);let gt=above(a,b);let ge=at_least(a,b);
 assert(lt==(a.0@<b.0@));assert(le==(a.0@<=b.0@));
 assert(gt==(a.0@>b.0@));assert(ge==(a.0@>=b.0@));
}}}}
"#
        ),
        5,
    );
}

#[test]
fn false_semantics_fail_only_the_client_with_healthy_candidates() {
    for condition in [
        "c==core::cmp::Ordering::Less",
        "c==core::cmp::Ordering::Equal",
        "c==core::cmp::Ordering::Greater",
        "c==vstd::std_specs::cmp::OrdSpec::cmp_spec(&b.0@,&a.0@)",
    ] {
        let e = verify_one_file(
            "nonzero_order_false",
            format!(
                r#"{BASE}{CANDIDATES}
verus! {{fn wrong(a:&BorTag,b:&BorTag) {{let c=a.cmp(b);assert({condition});}}}}
"#
            ),
            &["vstd", "--infer-goals", "--no-cheating", "--output-json"],
        )
        .unwrap_err();
        let j = e.json_output.unwrap();
        let r = &j["verus-infer"];
        assert_eq!(j["verification-results"]["encountered-vir-error"], false);
        assert_eq!(r["failed_functions"].as_array().unwrap().len(), 1);
        assert!(r["failed_functions"][0].as_str().unwrap().ends_with("::wrong"));
        for f in r["functions"].as_array().unwrap() {
            assert_eq!(f["required_checks_passed"], true);
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][p]["status"], "proved");
            }
        }
        assert_eq!(r["interface_written"], false);
    }
}

#[test]
fn unsupported_shapes_and_missing_delegate_fail_closed() {
    for s in [
        BASE.replace("NonZero<u64>", "NonZero<u32>"),
        BASE.replace("BorTag(NonZero<u64>)", "BorTag<T>(NonZero<T>)"),
        BASE.replace("BorTag(NonZero<u64>)", "BorTag(NonZero<u64>,u64)"),
        BASE.replace(
            "infer_derive(PartialEq,PartialOrd,Ord)",
            "infer_derive(PartialEq,PartialOrd)",
        ),
    ] {
        assert!(
            verify_one_file(
                "nonzero_order_unsupported",
                s,
                &["vstd", "--infer-goals", "--no-cheating", "--output-json"]
            )
            .is_err()
        );
    }
}

#[test]
fn selected_wrapper_ordering_removes_genuine_prior_bool_export() {
    let crates = common::infer::InferCrates::new();
    let old = "use vstd::prelude::*;verus!{pub fn old_api(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}";
    let partial_only = BASE
        .replace("infer_derive(PartialEq,PartialOrd,Ord)", "infer_derive(PartialEq,PartialOrd)")
        .replace("Eq, PartialOrd, Ord)]", "Eq, PartialOrd)]");
    // PartialOrd can use its existing carrier model without registering Ord.
    // Request detection must still refuse publication, including with goals off.
    for (source, options) in [
        (BASE, vec!["--infer-goals", "--no-cheating", "--output-json"]),
        (partial_only.as_str(), vec!["--infer-goals", "--no-cheating", "--output-json"]),
        (partial_only.as_str(), vec!["--no-cheating", "--output-json"]),
    ] {
        let p = crates.build_source(
            "nonzero_local_only",
            old,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        assert!(crates.vir("nonzero_local_only").exists());
        let p = crates.build_source("nonzero_local_only", source, &[], &options, true);
        assert!(!p.status.success());
        assert!(String::from_utf8_lossy(&p.stderr).contains("local-only"));
        assert!(!crates.vir("nonzero_local_only").exists());
    }
}
