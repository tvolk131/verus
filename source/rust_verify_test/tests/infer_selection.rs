#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const QUERY: &str = r#"use vstd::prelude::*;verus!{
#[derive(Copy,Clone,Debug)] enum Error{TooFew,Missing}
struct Collector<T:Copy>{slots:Vec<Option<(T,u16)>>,count:usize,round:u32}
impl<T:Copy> Collector<T>{
 #[verifier::infer] fn parents(&self,i:usize)->Result<&Vec<Option<(T,u16)>>,Error>{
  if self.count<3{return Err(Error::TooFew);}
  if i>=self.slots.len(){return Err(Error::Missing);}
  match self.slots[i]{Some((_,r)) if u32::from(r)+1==self.round=>Ok(&self.slots),_=>Err(Error::Missing)}
 }
}
#[verifier::infer] fn composed<T:Copy>(c:&Collector<T>,i:usize)->Result<&Vec<Option<(T,u16)>>,Error>{c.parents(i)}
fn goal<T:Copy>(c:&Collector<T>,i:usize)->(r:Result<&Vec<Option<(T,u16)>>,Error>)
 ensures r.is_ok()==(c.count>=3 && i<c.slots@.len() && c.slots@[i as int].is_some() && c.slots@[i as int].unwrap().1 as int+1==c.round),
 r.is_ok() ==> r.unwrap()@==c.slots@,
 c.count<3 ==> r==Err::<&Vec<Option<(T,u16)>>,Error>(Error::TooFew),
{composed(c,i)}
}"#;
#[test]
fn borrowed_result_queries_and_composition_have_all_exact_checks() {
    let d = verify_one_file("selection", QUERY.into(), &["vstd", "--no-cheating"]).unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(d.notes.iter().filter(|n| n.message.contains(p)).count(), 2, "{d:?}");
    }
}
#[test]
fn error_variants_are_part_of_the_exact_result() {
    let s = QUERY.replace("return Err(Error::TooFew)", "return Err(Error::Missing)");
    let e = verify_one_file("wrong_error", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{e:?}");
    assert_eq!(
        e.notes.iter().filter(|n| n.message.contains("output exactness: PROVED")).count(),
        2,
        "{e:?}"
    );
}
#[test]
fn unchecked_index_remains_a_caller_safety_requirement() {
    let s = QUERY.replace("if i>=self.slots.len(){return Err(Error::Missing);}", "");
    let e = verify_one_file("index_domain", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{e:?}");
}
#[test]
fn borrowed_output_includes_all_selected_contents() {
    let s = r#"use vstd::prelude::*;verus!{
 struct Pair{left:Vec<u8>,right:Vec<u8>}
 #[verifier::infer] fn select(p:&Pair,flag:bool)->Result<&Vec<u8>,u8>{if flag{Ok(&p.left)}else{Err(4)}}
 fn check(p:&Pair)->(r:Result<&Vec<u8>,u8>) ensures r.is_ok(),r.unwrap()@==p.right@,{select(p,true)}
}"#;
    let e = verify_one_file("wrong_contents", s.into(), &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{e:?}");
    assert!(e.notes.iter().any(|d| d.message.contains("output exactness: PROVED")), "{e:?}");
}
#[test]
fn bounded_error_payloads_compose_with_borrowed_collection_results() {
    let s = r#"use vstd::prelude::*;verus!{
 enum Error{Bad(u64),Other}
 #[verifier::infer] fn query(v:&Vec<u8>)->Result<&Vec<u8>,Error>{Ok(v)}
}"#;
    verify_one_file("error_payload", s.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn borrowed_option_payloads_support_canonical_primitive_widening() {
    let s = r#"use vstd::prelude::*;verus!{
 struct Slots{v:Vec<Option<u16>>,round:u32}
 impl Slots{#[verifier::infer] fn query(&self,i:usize)->Result<&Vec<Option<u16>>,u8>{
 if i>=self.v.len(){return Err(1);}
 match self.v[i].as_ref(){Some(r) if u32::from(*r)+1==self.round=>Ok(&self.v),_=>Err(2)}
 }}
 fn check(s:&Slots,i:usize)->(r:Result<&Vec<Option<u16>>,u8>)
 ensures r.is_ok() ==> r.unwrap()@==s.v@,{s.query(i)}
}"#;
    let d = verify_one_file("borrowed_widen", s.into(), &["vstd", "--no-cheating"]).unwrap();
    assert!(d.notes.iter().any(|d| d.message.contains("output exactness: PROVED")), "{d:?}");
}

#[test]
fn tagged_borrows_preserve_complete_selected_values() {
    let s = r#"use vstd::prelude::*;verus!{
 struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
 struct Table<T>{slots:Slots<T>,ready:bool}
 impl<T> Table<T>{
  #[verifier::infer] fn select(&self)->Result<&Slots<T>,u8>{if self.ready{Ok(&self.slots)}else{Err(4)}}
  #[verifier::infer] fn maybe(&self)->Option<&Slots<T>>{if self.ready{Some(&self.slots)}else{None}}
 }
 #[verifier::infer] fn forward<T>(t:&Table<T>)->Result<&Slots<T>,u8>{t.select()}
 fn goal<T>(t:&Table<T>){
  let r=forward(t);assert(r.is_ok()==t.ready);if let Ok(value)=r{assert(*value==t.slots);}
  let r=t.maybe();assert(r.is_some()==t.ready);if let Some(value)=r{assert(*value==t.slots);}
 }
}"#;
    let d = verify_one_file(
        "tagged_complete_borrow",
        s.into(),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap();
    let fs = d.json_output.unwrap()["verus-infer"]["functions"].as_array().unwrap().clone();
    assert_eq!(fs.len(), 3);
    for f in fs {
        for p in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][p]["status"], "proved", "{f}");
        }
    }
}

#[test]
fn equal_views_cannot_substitute_a_different_returned_object() {
    let s = r#"use vstd::prelude::*;verus!{
 struct Slots(Vec<u8>);
 impl View for Slots {type V=Seq<u8>;closed spec fn view(&self)->Self::V{self.0@}}
 struct Table{left:Slots,right:Slots}
 #[verifier::infer] fn select(t:&Table)->Result<&Slots,u8>{Ok(&t.left)}
 fn wrong(t:&Table) requires t.left@==t.right@, {
  let r=select(t);if let Ok(value)=r{assert(*value==t.right);}
 }
}"#;
    let e = verify_one_file(
        "wrong_equal_view_borrow",
        s.into(),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("assertion failed")), "{:?}", e);
    let r = &e.json_output.unwrap()["verus-infer"];
    assert_eq!(r["interface_written"], false);
    assert_eq!(r["functions"][0]["required_checks_passed"], true);
}

#[test]
fn copying_a_selected_borrow_keeps_contents_without_asserting_clone_identity() {
    let s = r#"use vstd::prelude::*;verus!{
 struct Holder{data:Vec<u8>,ready:bool}
 #[verifier::infer] fn select(h:&Holder)->Result<&Vec<u8>,u8>{if h.ready{Ok(&h.data)}else{Err(1)}}
 #[verifier::infer] fn copy_selected(h:&Holder)->Result<Vec<u8>,u8>{Ok(select(h)?.clone())}
 fn goal(h:&Holder)->(r:Result<Vec<u8>,u8>)
  ensures r.is_ok()==h.ready,r.is_ok() ==> r.unwrap()@==h.data@,
 {copy_selected(h)}
}"#;
    verify_one_file("selected_borrow_copy", s.into(), &["vstd", "--no-cheating"]).unwrap();
}
