#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::verify_exact as checked;
use common::*;

const BASE: &str = r#"use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone)] pub struct Quantity(pub u8);
#[verus_infer] impl core::ops::Mul<u8> for Quantity {type Output=Self;fn mul(self,rhs:u8)->Self {Quantity(self.0*rhs)}}
#[verus_infer] impl core::ops::Div<u8> for Quantity {type Output=Self;fn div(self,rhs:u8)->Self {Quantity(self.0/rhs)}}
#[verus_infer] impl core::ops::Add for Quantity {type Output=Self;fn add(self,rhs:Self)->Self {Quantity(self.0+rhs.0)}}
#[verus_infer] impl core::ops::Sub for Quantity {type Output=Self;fn sub(self,rhs:Self)->Self {Quantity(self.0-rhs.0)}}
#[verus_infer] impl core::ops::Rem<u8> for Quantity {type Output=Self;fn rem(self,rhs:u8)->Self {Quantity(self.0%rhs)}}
"#;

#[test]
fn primitive_and_wrapper_rhs_preserve_all_five_operations() {
    checked(
        "binary_wrapper_unsigned",
        format!(
            r#"{BASE}
verus!{{fn goal(x:Quantity,y:Quantity)
 requires y.0>0,x.0>=y.0,x.0+y.0<=255,x.0*y.0<=255,
{{let a=x+y;assert(a.0==x.0+y.0);let s=x-y;assert(s.0==x.0-y.0);
 let m=x*y.0;assert(m.0==x.0*y.0);let d=x/y.0;assert(d.0==x.0/y.0);let r=x%y.0;assert(r.0==x.0%y.0);}}}}
"#
        ),
        5,
    );
}
#[test]
fn signed_division_remainder_and_overflow_domains_are_checked() {
    let s = BASE.replace("u8", "i8");
    checked(
        "binary_wrapper_signed",
        format!(
            r#"{s}
verus!{{fn goal(){{let q=Quantity(-7)/3;assert(q.0 == -2);let r=Quantity(-7)%3;assert(r.0 == -1);}}}}
"#
        ),
        5,
    );
}
#[test]
fn trait_name_does_not_assign_implementation_meaning() {
    let s = BASE.replace("Quantity(self.0+rhs.0)", "Quantity(self.0-rhs.0)");
    checked(
        "binary_nonstandard_add",
        format!(
            r#"{s}
verus!{{fn goal(x:Quantity,y:Quantity) requires x.0>=y.0,{{let r=x+y;assert(r.0==x.0-y.0);}}}}
"#
        ),
        5,
    );
}
#[test]
fn unsafe_calls_and_wrong_application_goals_are_rejected() {
    for (label, goal) in [
        ("zero", "let _=Quantity(1)/0;"),
        ("overflow", "let _=Quantity(255)+Quantity(1);"),
        ("underflow", "let _=Quantity(0)-Quantity(1);"),
        ("wrong", "let r=Quantity(8)/2;assert(r.0==5);"),
    ] {
        let e = verify_one_file(
            label,
            format!("{BASE} verus!{{fn goal(){{{goal}}}}}"),
            &["vstd", "--no-cheating", "--output-json"],
        )
        .unwrap_err();
        let r = &e.json_output.as_ref().unwrap()["verus-infer"];
        assert_eq!(r["crate_success"], false);
        assert_eq!(r["interface_written"], false);
        assert!(
            r["functions"].as_array().unwrap().iter().all(|f| f["required_checks_passed"] == true)
        );
    }
}
#[test]
fn extra_effects_are_not_silently_summarized() {
    let s = BASE.replace("Quantity(self.0*rhs)", "{let _=1u8/rhs;Quantity(self.0*rhs)}");
    assert!(verify_one_file("binary_extra_effect", s, &["vstd", "--no-cheating"]).is_err());
}

#[test]
fn derived_opt_in_is_a_request_not_proof_evidence() {
    let s = r#"use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone)] pub struct Quantity(pub u8);
verus!{
impl Quantity {pub open spec fn __verus_infer_derived_add(&self)->bool {true}}
impl vstd::std_specs::ops::AddSpecImpl for Quantity {
 open spec fn obeys_add_spec()->bool {true}
 open spec fn add_req(self,rhs:Self)->bool {self.0 as int + rhs.0 as int <=255}
 open spec fn add_spec(self,rhs:Self)->Self {Quantity((self.0+rhs.0) as u8)}
}}
#[automatically_derived]
#[verus_verify]
impl core::ops::Add for Quantity {
 type Output=Self;
 fn add(self,rhs:Self)->Self {Quantity(self.0+rhs.0)}
}
"#;
    checked("derived_operator_request", s.into(), 1);
    let missing = s.replace("#[automatically_derived]", "");
    let e = verify_one_file(
        "missing_checked_derive",
        missing,
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("derived trait request requires")),
        "{:?}",
        e
    );
    assert_eq!(e.json_output.unwrap()["verus-infer"]["interface_written"], false);
    for (label, bad) in [
        ("wrong_derived_body", s.replace("Quantity(self.0+rhs.0)", "Quantity(self.0-rhs.0)")),
        ("vacuous_derived_domain", s.replace("self.0 as int + rhs.0 as int <=255", "false")),
    ] {
        let e =
            verify_one_file(label, bad, &["vstd", "--no-cheating", "--output-json"]).unwrap_err();
        let r = &e.json_output.as_ref().unwrap()["verus-infer"];
        assert_eq!(r["crate_success"], false);
        assert_eq!(r["interface_written"], false);
        assert!(
            r["functions"].as_array().unwrap().iter().any(|f| f["required_checks_passed"] == false)
        );
    }
}

#[test]
fn explicit_primitive_trait_calls_have_exact_checked_arithmetic_domains() {
    let s = r#"use vstd::prelude::*;
#[verus_infer] fn plus(x:u8,y:u8)->u8 {<u8 as core::ops::Add>::add(x,y)}
#[verus_infer] fn div(x:i8,y:i8)->i8 {<i8 as core::ops::Div>::div(x,y)}
#[verus_infer] fn rem(x:i8,y:i8)->i8 {<i8 as core::ops::Rem>::rem(x,y)}
verus!{fn goal(){let p=plus(255,0);assert(p==255);let d=div(-7i8,3i8);assert(d== -2);let r=rem(-7i8,3i8);assert(r== -1);}}
"#;
    checked("primitive_trait_calls", s.into(), 3);
    for (label, call) in [
        ("primitive_add_overflow", "plus(255,1)"),
        ("primitive_div_edge", "div(-128i8,-1i8)"),
        ("primitive_rem_edge", "rem(-128i8,-1i8)"),
        ("primitive_div_zero", "div(1,0)"),
    ] {
        let bad = s.replace("let p=plus(255,0);", &format!("let p={call};"));
        let e =
            verify_one_file(label, bad, &["vstd", "--no-cheating", "--output-json"]).unwrap_err();
        let r = &e.json_output.as_ref().unwrap()["verus-infer"];
        assert_eq!(r["functions"].as_array().unwrap().len(), 3);
        assert!(
            r["functions"].as_array().unwrap().iter().all(|f| f["required_checks_passed"] == true)
        );
        assert_eq!(r["interface_written"], false);
    }
}

#[test]
fn copied_shared_receivers_compose_original_operator_calls() {
    checked(
        "shared_wrapper_quorum",
        format!(
            r#"{BASE}
#[verus_infer] fn quota(n:&Quantity)->Quantity {{(*n*2)/3+Quantity(1)}}
verus!{{fn goal(n:&Quantity)->(r:Quantity) requires n.0<=127, ensures r.0==(2*n.0)/3+1,{{quota(n)}}}}
"#
        ),
        6,
    );
}

#[test]
fn every_machine_integer_width_checks_its_own_operator_domains() {
    for typ in ["u16", "u32", "u64", "u128", "usize", "i16", "i32", "i64", "i128", "isize"] {
        checked(&format!("binary_width_{typ}"), BASE.replace("u8", typ), 5);
    }
}
