#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const BASE: &str = r#"use vstd::prelude::*;
#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]
#[derive(Copy,Clone,Eq,PartialEq,PartialOrd,Ord)]
pub struct Quantity(pub u8);
#[verus_infer] fn equal(a:Quantity,b:Quantity)->bool{a==b}
#[verus_infer] fn unequal(a:Quantity,b:Quantity)->bool{a!=b}
#[verus_infer] fn less(a:Quantity,b:Quantity)->bool{a<b}
#[verus_infer] fn less_equal(a:Quantity,b:Quantity)->bool{a<=b}
#[verus_infer] fn greater(a:Quantity,b:Quantity)->bool{a>b}
#[verus_infer] fn greater_equal(a:Quantity,b:Quantity)->bool{a>=b}
#[verus_verify] struct Record { amount:Quantity }
#[verus_verify] impl Record {
 #[verus_infer] fn below(&self,limit:Quantity)->bool{self.amount<limit}
}
"#;
fn checked(label: &str, s: String, count: usize) {
    let r = verify_one_file(label, s, &["vstd", "--no-cheating", "--output-json"])
        .unwrap()
        .json_output
        .unwrap();
    let fs = r["verus-infer"]["functions"].as_array().unwrap();
    assert_eq!(fs.len(), count);
    for f in fs {
        for p in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][p]["status"], "proved", "{f}");
        }
    }
}
#[test]
fn original_native_derives_and_inherited_boolean_methods_are_checked() {
    checked(
        "native_derived_comparisons",
        format!(
            r#"{BASE}
verus!{{fn goal(a:Quantity,b:Quantity){{
 let eq=equal(a,b);assert(eq==(a.0==b.0));let ne=unequal(a,b);assert(ne==(a.0!=b.0));
 let lt=less(a,b);assert(lt==(a.0<b.0));let le=less_equal(a,b);assert(le==(a.0<=b.0));
 let gt=greater(a,b);assert(gt==(a.0>b.0));let ge=greater_equal(a,b);assert(ge==(a.0>=b.0));
 let rec=Record{{amount:a}};let r=rec.below(b);assert(r==(a.0<b.0));
}}}}
"#
        ),
        8,
    );
}
#[test]
fn partial_order_without_ord_uses_its_actual_field_delegate() {
    checked(
        "native_derived_partial_order",
        BASE.replace("PartialEq, PartialOrd, Ord", "PartialEq, PartialOrd")
            .replace("PartialOrd,Ord)", "PartialOrd)"),
        8,
    );
}
#[test]
fn wrong_comparison_application_does_not_publish() {
    let e = verify_one_file(
        "native_derived_wrong_goal",
        format!("{BASE} verus!{{fn goal(){{let r=less(Quantity(3),Quantity(2));assert(r);}}}}"),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    let r = &e.json_output.as_ref().unwrap()["verus-infer"];
    assert_eq!(r["interface_written"], false);
    assert!(r["functions"].as_array().unwrap().iter().all(|f| f["required_checks_passed"] == true));
}

// This provider has no exact-mode target. The actual cmp body still must be
// checked and the generated semantic interface still requires publication gates.
const ORD_ONLY: &str = r#"use vstd::prelude::*;
#[verus_verify(infer_derive(Ord))]
#[derive(Copy,Clone,Eq,PartialEq,PartialOrd,Ord)] pub struct Quantity(pub u8);
"#;
#[test]
fn ordering_only_provider_has_no_precision_claim_and_requires_full_verification() {
    checked("native_derived_ord_only", ORD_ONLY.into(), 0);
    for flag in ["--no-verify", "--verify-only-module ignored"] {
        let e = verify_one_file(
            "native_derived_ord_filter",
            ORD_ONLY.into(),
            &["vstd", "--no-cheating", "--output-json", flag],
        )
        .unwrap_err();
        assert!(
            e.errors.iter().any(|d| d.message.contains("requires full-crate verification")),
            "{:?}",
            e
        );
    }
}
#[test]
fn delegation_and_shape_dependencies_are_explicit() {
    for (label, s) in [
        ("missing_ord_request", ORD_ONLY.replace("infer_derive(Ord)", "infer_derive(PartialOrd)")),
        ("missing_derive", ORD_ONLY.replace("PartialOrd,Ord)", "PartialOrd)")),
        ("generic_derive", ORD_ONLY.replace("Quantity(pub u8)", "Quantity<T>(pub T)")),
    ] {
        assert!(verify_one_file(label, s, &["vstd", "--no-cheating"]).is_err());
    }
}

const MANUAL_ORD: &str = r#"use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone,Eq,PartialEq,PartialOrd)] pub struct Quantity(pub u8);
verus!{
impl Quantity {pub open spec fn __verus_infer_derived_cmp(&self)->bool{true}}
impl vstd::std_specs::cmp::OrdSpecImpl for Quantity {
 open spec fn obeys_cmp_spec()->bool{true}
 open spec fn cmp_spec(&self,rhs:&Self)->core::cmp::Ordering{
  vstd::std_specs::cmp::OrdSpec::cmp_spec(&self.0,&rhs.0)
 }
}}
#[automatically_derived] #[verus_verify]
impl Ord for Quantity {fn cmp(&self,rhs:&Self)->core::cmp::Ordering {self.0.cmp(&rhs.0)}}
"#;
#[test]
fn ordering_model_requires_the_actual_body_and_matching_owner() {
    checked("checked_ordinary_ordering", MANUAL_ORD.into(), 0);
    for (label,s) in [
  ("reversed_cmp",MANUAL_ORD.replace("self.0.cmp(&rhs.0)","rhs.0.cmp(&self.0)")),
  ("external_cmp",MANUAL_ORD.replace("#[automatically_derived] #[verus_verify]","#[verus_verify(external)]")),
  ("missing_cmp",MANUAL_ORD.replace("impl Ord for Quantity {fn cmp(&self,rhs:&Self)->core::cmp::Ordering {self.0.cmp(&rhs.0)}}","")),
 ] {
  let e=verify_one_file(label,s,&["vstd","--no-cheating","--output-json"]).unwrap_err();
  if let Some(r)=e.json_output{assert_eq!(r["verus-infer"]["interface_written"],false);}
 }
}
#[test]
fn comparisons_of_computed_temporaries_and_shared_record_fields() {
    let s = r#"use vstd::prelude::*;
#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]
#[derive(Copy,Clone,Eq,PartialEq,PartialOrd,Ord)] pub struct Quantity(pub u8);
#[verus_infer] fn copy(a:&Quantity)->Quantity{*a}
#[verus_infer] fn below(a:&Quantity,b:&Quantity)->bool{*a<copy(b)}
#[verus_verify] struct State { amount:Quantity, round:u8 }
impl State {#[verus_infer] fn check(&self,b:Quantity)->bool{self.amount<Quantity(b.0)}}
verus!{fn goal(s:&State,b:Quantity){let r=s.check(b);assert(r==(s.amount.0<b.0));}}
"#;
    checked("computed_comparisons", s.into(), 4);
}
