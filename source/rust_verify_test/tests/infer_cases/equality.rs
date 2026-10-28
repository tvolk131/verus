use super::*;

#[test]
fn generic_equality_requires_a_checked_identity_law() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
use vstd::std_specs::cmp::{PartialEqIdentity,group_eq_identity};
verus! {
broadcast use group_eq_identity;
pub struct Key<T>(pub T);
#[verifier::infer] pub fn same<T:PartialEqIdentity>(a:&Key<T>,b:&Key<T>)->bool{a.0==b.0}
#[verifier::infer] pub fn different<T:PartialEqIdentity>(a:&Key<T>,b:&Key<T>)->bool{a.0!=b.0}
pub fn goal<T:PartialEqIdentity>(a:&Key<T>,b:&Key<T>) {
 let same=same(a,b);let different=different(a,b);
 assert(same==(a.0==b.0));assert(different==!same);
}
pub fn empty_arrays(a:&Key<[u8;0]>,b:&Key<[u8;0]>) {
 proof {broadcast use vstd::array::group_array_axioms;assert(a.0 =~= b.0);}
 let result=same(a,b);assert(result);
}
}"#;
    for _ in 0..2 {
        let r = crates.build("generic_equality", source, &[], &["--no-cheating", "--output-json"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        for f in json_inference(&r)["functions"].as_array().unwrap() {
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][proof]["status"], "proved");
            }
        }
    }
    // `Eq` is a Rust marker, not a verified law. It cannot justify the proposal.
    let weak = source.replace("T:PartialEqIdentity", "T:Eq");
    let r = crates.build("generic_equality", &weak, &[], &["--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("generic_equality").exists());
    let error = String::from_utf8_lossy(&r.stderr);
    assert!(
        error.contains("postcondition not satisfied") || error.contains("output exactness"),
        "{}",
        error
    );
}
