#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const WRAPPER: &str = r#"use vstd::prelude::*;
#[verus_verify(clone_via_view)]
#[derive(Clone)] pub struct Slots<T>(Vec<Option<T>>);
verus! {
 impl<T> View for Slots<T> {type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
}
"#;

#[test]
fn native_derive_supports_exact_generic_clone_with_checked_value_laws() {
    let source = format!(
        r#"{WRAPPER}
use vstd::std_specs::clone::{{CloneIdentity,lemma_cloned_identity}};
verus!{{pub trait Family{{type Hash:Copy+CloneIdentity;}} broadcast use lemma_cloned_identity;}}
#[verus_infer] pub fn copy<H:Family>(v:&Slots<(H::Hash,u16)>)->Slots<(H::Hash,u16)>{{v.clone()}}
verus!{{fn goal<H:Family>(v:&Slots<(H::Hash,u16)>)->(r:Slots<(H::Hash,u16)>) ensures r@==v@,{{copy::<H>(v)}}}}
"#
    );
    let d = verify_one_file("native_clone", source, &["vstd", "--no-cheating"]).unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(d.notes.iter().filter(|n| n.message.contains(p)).count(), 1, "{d:?}");
    }
}

#[test]
fn copy_bound_alone_cannot_establish_clone_identity() {
    let source = format!(
        r#"{WRAPPER}
#[verus_infer] pub fn copy<T:Copy>(v:&Slots<T>)->Slots<T>{{v.clone()}}
"#
    );
    let e = verify_one_file("copy_is_not_clone_identity", source, &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
    assert!(!e.notes.iter().any(|d| d.message.contains("verified contracts available")), "{:?}", e);
}

#[test]
fn checked_clone_identity_law_cannot_be_forged_for_nonidentity_clone() {
    let source = r#"use vstd::prelude::*;use vstd::std_specs::clone::CloneIdentity;verus!{
 #[derive(Copy)] pub struct Odd(pub u8);
 impl Clone for Odd { fn clone(&self)->(r:Self) ensures r.0==0,{Odd(0)} }
 impl CloneIdentity for Odd {proof fn lemma_clone_identity(a:Self,b:Self) ensures a==b,{}}
}"#;
    let e = verify_one_file("dishonest_clone_law", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
}

#[test]
fn tuple_clone_relations_preserve_custom_nonidentity_clone_behavior() {
    let source = r#"use vstd::prelude::*;verus!{
 pub struct Odd(pub u8);
 impl Clone for Odd {fn clone(&self)->(r:Self) ensures r.0==0,{Odd(0)}}
 fn check(v:&(Odd,u8))->(r:(Odd,u8)) ensures r.0.0==0 || r.0==v.0,r.1==v.1,{v.clone()}
 fn nested(v:&((u8,u16),(u32,u64)))->(r:((u8,u16),(u32,u64))) ensures r==*v,{v.clone()}
 fn by_function_pointer(v:&(u8,u16)){let f=<(u8,u16) as Clone>::clone;let r=f(v);assert(r==*v);}
 fn one(v:&(u8,))->(r:(u8,)) ensures r==*v,{v.clone()}
 fn three(v:&(u8,u16,u32))->(r:(u8,u16,u32)) ensures r==*v,{v.clone()}
 fn four(v:&(u8,u16,u32,u64))->(r:(u8,u16,u32,u64)) ensures r==*v,{v.clone()}
}"#;
    verify_one_file("tuple_clone_relations", source.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn tuple_clone_never_invents_equality_for_a_nonidentity_component() {
    let source = r#"use vstd::prelude::*;verus!{
 pub struct Odd(pub u8);
 impl Clone for Odd {fn clone(&self)->(r:Self) ensures r.0==0,{Odd(0)}}
 fn wrong(v:&(Odd,u8))->(r:(Odd,u8)) ensures r==*v,{v.clone()}
}"#;
    let e =
        verify_one_file("tuple_clone_wrong_equality", source.into(), &["vstd", "--no-cheating"])
            .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
}

#[test]
fn false_native_clone_relation_is_checked_against_the_real_derive_body() {
    let source = r#"use vstd::prelude::*;
verus!{
#[derive(Clone)] pub struct Slots<T>(Vec<Option<T>>);
impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
impl<T:Clone> Slots<T>{pub open spec fn __verus_clone_relation(&self,r:Self)->bool{false}}
}"#;
    let e = verify_one_file("false_clone_relation", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
}

#[test]
fn wrong_relation_signature_fails_without_malformed_calls() {
    let source = r#"use vstd::prelude::*;
verus!{
#[derive(Clone)] pub struct Slots<T>(Vec<Option<T>>);
impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
impl<T:Clone> Slots<T>{pub open spec fn __verus_clone_relation(&self,r:u64)->bool{true}}
fn wrong(v:&Slots<u8>)->(r:Slots<u8>) ensures r@==v@,{v.clone()}
}"#;
    let e = verify_one_file("malformed_clone_relation", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
}

#[test]
fn unsupported_tuple_arity_has_an_explicit_diagnostic() {
    let s = "use vstd::prelude::*;verus!{fn copy(v:&(u8,u8,u8,u8,u8)){let _=v.clone();}}";
    let e = verify_one_file("tuple_arity", s.into(), &["vstd"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("built-in instance")), "{:?}", e);
}
