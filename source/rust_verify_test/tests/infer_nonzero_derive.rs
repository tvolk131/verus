#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::verify_exact as checked;
use common::*;

// The pinned Miri carrier and all seven actual native derives remain intact.
const BASE: &str = r#"use vstd::prelude::*;
use std::num::NonZero;
#[verus_verify(infer_derive(PartialEq))]
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorTag(NonZero<u64>);
"#;

#[test]
fn original_nonzero_equality_covers_full_width_borrowed_tags() {
    checked(
        "nonzero_derived_eq",
        format!(
            r#"{BASE}
verus! {{
fn goal(a:&BorTag,b:&BorTag) {{
    let eq=a==b; let ne=a!=b;
    assert(eq==(a.0@==b.0@)); assert(ne==!eq);
    proof {{
    if a.0@==u64::MAX && b.0@==u64::MAX {{ assert(eq); }}
    if a.0@==9223372036854775808u64 && b.0@==1 {{ assert(ne); }}
    }}
}}
}}
"#
        ),
        1,
    );
}

#[test]
fn wrong_returned_relation_fails_with_healthy_derive_obligations() {
    let e = verify_one_file(
        "nonzero_false_eq",
        format!(
            r#"{BASE}
verus! {{fn wrong(a:&BorTag,b:&BorTag){{let r=a==b;assert(r!=(a.0@==b.0@));}}}}
"#
        ),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    let r = &e.json_output.as_ref().unwrap()["verus-infer"];
    assert_eq!(r["interface_written"], false);
    let fs = r["functions"].as_array().unwrap();
    assert_eq!(fs.len(), 1);
    assert!(fs.iter().all(|f| f["required_checks_passed"] == true));
}

#[test]
fn nonzero_arithmetic_and_other_payloads_are_not_in_the_checked_profile() {
    for (name, s) in [
        (
            "nonzero_arithmetic",
            BASE.replace("infer_derive(PartialEq)", "infer_derive(PartialEq,Add)"),
        ),
        ("nonzero_u32", BASE.replace("NonZero<u64>", "NonZero<u32>")),
    ] {
        assert!(verify_one_file(name, s, &["vstd", "--no-cheating"]).is_err());
    }
}
