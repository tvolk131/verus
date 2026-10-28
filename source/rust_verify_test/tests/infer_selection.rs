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
