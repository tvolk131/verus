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

#[test]
fn native_clone_preserves_checked_deep_views_without_copy() {
    let source = r#"use vstd::prelude::*;
use vstd::std_specs::clone::{CloneView,lemma_cloned_view};
verus!{
 broadcast use lemma_cloned_view;
 pub struct Value { pub number:u8, pub representation:bool }
 impl DeepView for Value {type V=u8;open spec fn deep_view(&self)->u8{self.number}}
 impl Clone for Value {
  fn clone(&self)->(r:Self) ensures r.number==self.number,r.representation==!self.representation,
  {Value{number:self.number,representation:!self.representation}}
 }
 impl CloneView for Value {
  proof fn lemma_clone_view(a:Self,b:Self) ensures a.deep_view()==b.deep_view(),{}
 }
 fn generic<T:CloneView>(v:&Option<Vec<T>>)->(r:Option<Vec<T>>)
  ensures r.deep_view()==v.deep_view(),{v.clone()}
 fn concrete(v:&Option<Vec<Value>>){let r=generic(v);assert(r.deep_view()==v.deep_view());}
 fn string(v:&String)->(r:String) ensures r.deep_view()==v.deep_view(),{v.clone()}
}"#;
    verify_one_file("clone_view_without_copy", source.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn native_clone_view_law_rejects_changed_abstract_values() {
    let source = r#"use vstd::prelude::*;use vstd::std_specs::clone::CloneView;verus!{
 pub struct Value{pub number:u8}
 impl DeepView for Value{type V=u8;open spec fn deep_view(&self)->u8{self.number}}
 impl Clone for Value{fn clone(&self)->(r:Self) ensures r.number==0,{Value{number:0}}}
 impl CloneView for Value{proof fn lemma_clone_view(a:Self,b:Self)
  ensures a.deep_view()==b.deep_view(),{}}
}"#;
    let e = verify_one_file("dishonest_clone_view", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{e:?}");
}

#[test]
fn plain_clone_does_not_supply_a_deep_view_preservation_law() {
    let source = r#"use vstd::prelude::*;verus!{
 fn wrong<T:Clone+DeepView>(v:&Vec<T>)->(r:Vec<T>)
  ensures r.deep_view()==v.deep_view(),{v.clone()}
}"#;
    let e = verify_one_file("plain_clone_no_view_law", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{e:?}");
}

// The authored invariant and transition remain ordinary goals. The assistance
// supplies a checked pointwise bridge through Clone's existing relation.
const TRANSPORT: &str = r#"
verus! {
 struct State<T> { slots:Slots<(T,u16)>, epoch:u16 }
 impl<T> View for State<T> {
  type V=(Seq<Option<(T,u16)>>,u16);
  closed spec fn view(&self)->Self::V {(self.slots@,self.epoch)}
 }
 impl<T> State<T> {
  spec fn valid(&self)->bool {
   self@.1>0 && forall|i:int|0<=i<self@.0.len() && self@.0[i].is_some()
    ==> self@.0[i].unwrap().1<self@.1
  }
 }
 fn advance<T:Clone>(previous:&State<T>)->(r:State<T>)
  requires previous@.1<u16::MAX,
  ensures previous.valid() ==> r.valid(),r@.1==previous@.1+1,
 {
  let next=State{slots:previous.slots.clone(),epoch:previous.epoch+1};
  next
 }
}
"#;

#[test]
fn ordinary_clone_transition_transports_a_hidden_pointwise_invariant() {
    let source = format!("{WRAPPER}{TRANSPORT}");
    let d = verify_one_file(
        "clone_pointwise_transport",
        source,
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    let report = &d.json_output.unwrap()["verus-infer"];
    assert_eq!(report["pointwise_assistance"].as_array().unwrap().len(), 1, "{report}");
    assert_eq!(report["pointwise_assistance"][0]["facts"], 1, "{report}");
    assert!(report["functions"].as_array().unwrap().is_empty());
}

#[test]
fn pointwise_transport_does_not_assume_the_invariant_or_change_the_transition() {
    for (index, body) in [
        TRANSPORT.replace("epoch:previous.epoch+1", "epoch:0"),
        TRANSPORT.replace("previous.valid() ==> r.valid()", "r.valid()"),
        TRANSPORT.replace("r@.1==previous@.1+1", "r@.1==previous@.1"),
    ]
    .iter()
    .enumerate()
    {
        let e = verify_one_file(
            &format!("clone_pointwise_wrong_{index}"),
            format!("{WRAPPER}{body}"),
            &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
        )
        .unwrap_err();
        assert!(
            e.errors
                .iter()
                .any(|d| d.message.contains("assertion failed")
                    || d.message.contains("postcondition")),
            "{:?}",
            e
        );
        assert_eq!(e.json_output.unwrap()["verus-infer"]["interface_written"], false);
    }
}
