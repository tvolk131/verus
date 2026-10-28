#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const TRAITS: &str = r#"use vstd::prelude::*;
verus! {
pub trait Family {type Id:Copy;}
pub trait Event {
 type Family:Family;
 spec fn key_value(&self)->usize;
 spec fn payload_value(&self)-><Self::Family as Family>::Id;
 fn key(&self)->(r:usize) ensures r==self.key_value();
 fn payload(&self)->(r:<Self::Family as Family>::Id) ensures r==self.payload_value();
}
pub struct Table<F:Family>{pub slots:Vec<Option<F::Id>>,pub used:usize,pub epoch:u64}
impl<F:Family> Table<F> {
 pub open spec fn valid(&self)->bool {
  self.used==self.slots@.count_prefix(|s:Option<F::Id>|s.is_some(),self.slots@.len())
 }
 #[verifier::infer] pub fn add<U:Event<Family=F>>(&mut self,u:&U){
  let i=u.key();
  if i>=self.slots.len(){return;}
  let empty=match self.slots[i]{None=>true,Some(_)=>false};
  if empty{self.slots[i]=Some(u.payload());self.used+=1;}
 }
}
fn check<F:Family,U:Event<Family=F>>(t:&mut Table<F>,u:&U)
 requires old(t).valid(),
 ensures final(t).valid(),final(t).epoch==old(t).epoch,
{t.add(u);}
}
"#;

const QUERIES: &str = r#"use vstd::prelude::*;verus!{
pub trait Queries {
 spec fn in_range_value(&self,lo:u16,hi:u16)->usize;
 spec fn after_value(&self,round:u16)->bool;
 fn in_range(&self,lo:u16,hi:u16)->(r:usize) ensures r==self.in_range_value(lo,hi);
 fn after(&self,round:u16)->(r:bool) ensures r==self.after_value(round);
}
pub struct Validator<Q:Queries>{pub query:Q,pub minimum:usize}
impl<Q:Queries> Validator<Q>{
 #[verifier::infer] pub fn enough(&self,round:u16)->bool{
  let previous=round-1;
  self.query.in_range(previous,previous)>=self.minimum && !self.query.after(previous)
 }
}
fn check<Q:Queries>(v:&Validator<Q>,round:u16)->(r:bool)
 requires round>0,
 ensures r==(v.query.in_range_value((round-1) as u16,(round-1) as u16)>=v.minimum
  && !v.query.after_value((round-1) as u16)),
{v.enough(round)}
}"#;

#[test]
fn scalar_parameterized_trait_queries_keep_argument_order_and_exactness(){
 let d=verify_one_file("parameterized_queries",QUERIES.into(),&["vstd","--no-cheating"]).unwrap();
 for p in ["soundness: PROVED","safe-domain maximality: PROVED","output exactness: PROVED"]{
  assert_eq!(d.notes.iter().filter(|n|n.message.contains(p)).count(),1,"{:?}",d);
 }
}

#[test]
fn query_equations_cannot_permute_or_change_arguments(){
 for (name,args) in [("swapped","hi,lo"),("changed","lo,lo")]{
  let source=QUERIES.replace("r==self.in_range_value(lo,hi)",&format!("r==self.in_range_value({args})"));
  unsupported(name,source);
 }
}

#[test]
fn query_arguments_are_evaluated_with_their_original_safety_checks(){
 let e=verify_one_file("query_zero_round",QUERIES.replace("requires round>0,",""),&["vstd","--no-cheating"]).unwrap_err();
 assert!(e.errors.iter().any(|d|d.message.contains("precondition not satisfied")),"{:?}",e);
 for p in ["soundness: PROVED","safe-domain maximality: PROVED","output exactness: PROVED"]{
  assert!(e.notes.iter().any(|n|n.message.contains(p)),"{:?}",e);
 }
}

#[test]
fn parameterized_queries_compose_into_exact_result_policies(){
 let source=format!(r#"{QUERIES} verus!{{
 impl<Q:Queries> Validator<Q>{{
  #[verifier::infer] fn validate(&self,round:u16)->Result<(),u16>{{
   let previous=round-1;
   if self.query.in_range(previous,previous)<self.minimum{{return Err(previous);}}
   if self.query.after(previous){{return Err(round);}}
   Ok(())
  }}
 }}
 fn policy<Q:Queries>(v:&Validator<Q>,round:u16)->(r:Result<(),u16>)
  requires round>0,
  ensures r.is_ok()==(v.query.in_range_value((round-1) as u16,(round-1) as u16)>=v.minimum
   && !v.query.after_value((round-1) as u16)),
 {{v.validate(round)}}
 }}"#);
 verify_one_file("queries_result_policy",source,&["vstd","--no-cheating"]).unwrap();
}

#[test]
fn generic_trait_getters_and_nested_associated_types_preserve_accounting() {
 let r=verify_one_file("traits",TRAITS.to_owned(),&["vstd","--infer-goals"]);
 assert!(r.is_ok(),"{:?}",r);
}

fn unsupported(name:&str,source:String){
 let e=verify_one_file(name,source,&["vstd","--infer-goals"]).unwrap_err();
 assert_vir_error_msg(e,"trait inference requires a certified precondition-free getter");
}

#[test]
fn weak_trait_contracts_do_not_become_exact_getters(){
 unsupported("weak",TRAITS.replace("ensures r==self.key_value()","ensures r<=self.key_value()"));
}

#[test]
fn getter_preconditions_are_explicitly_unsupported(){
 unsupported("requires",TRAITS.replace("ensures r==self.key_value()","requires self.key_value()<10, ensures r==self.key_value()"));
}

#[test]
fn recommended_spec_queries_are_not_silently_assumed(){
 unsupported("recommended",TRAITS.replace("spec fn key_value(&self)->usize;","spec fn key_value(&self)->usize recommends false;"));
}

#[test]
fn changed_accounting_still_fails_the_application_goal(){
 let e=verify_one_file("count",TRAITS.replace("self.used+=1","self.used+=2"),&["vstd","--infer-goals"]).unwrap_err();
 for label in ["soundness: PROVED","safe-domain maximality: PROVED","output exactness: PROVED"]{
  assert!(e.notes.iter().any(|n|n.message.contains(label)),"{:?}",e);
 }
 assert!(e.errors.iter().any(|e|e.message.contains("precondition not satisfied")||e.message.contains("postcondition not satisfied")),"{:?}",e);
}

#[test]
fn a_bad_concrete_implementation_cannot_satisfy_the_trait_contract(){
 let concrete=r#"
pub struct F;
impl Family for F{type Id=u64;}
pub struct Message{pub key:usize,pub payload:u64}
impl Event for Message {
 type Family=F;
 open spec fn key_value(&self)->usize{self.key}
 open spec fn payload_value(&self)->u64{self.payload}
 fn key(&self)->usize{self.key+1}
 fn payload(&self)->u64{self.payload}
}
"#;
 let source=TRAITS.replace("pub struct Table",&format!("{concrete}\npub struct Table"));
 let e=verify_one_file("bad_impl",source,&["vstd","--infer-goals"]).unwrap_err();
 assert!(e.errors.iter().any(|e|e.message.contains("postcondition not satisfied")),"{:?}",e);
}

#[test]
fn unrelated_concrete_implementations_are_checked_normally(){
 let concrete=r#"
pub struct F;
impl Family for F{type Id=u64;}
pub struct Message{pub key:usize,pub payload:u64}
impl Event for Message {
 type Family=F;
 open spec fn key_value(&self)->usize{self.key}
 open spec fn payload_value(&self)->u64{self.payload}
 fn key(&self)->usize{self.key}
 fn payload(&self)->u64{self.payload}
}
"#;
 let source=TRAITS.replace("pub struct Table",&format!("{concrete}\npub struct Table"));
 let r=verify_one_file("impl",source,&["vstd","--infer-goals"]);
 assert!(r.is_ok(),"{:?}",r);
}

#[test]
fn associated_payloads_compose_through_private_borrowed_tuple_lookups(){
 let wrappers=r#"
pub mod storage {
 use vstd::prelude::*;
 pub struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
 impl<T> Slots<T>{
  #[verifier::infer] pub fn len(&self)->usize{self.0.len()}
  #[verifier::infer] pub fn get(&self,i:usize)->Option<&T>{self.0[i].as_ref()}
  #[verifier::infer] pub fn put(&mut self,i:usize,v:T){self.0[i]=Some(v);}
 }
}
"#;
 let source=TRAITS.replace("pub struct Table",&format!("{wrappers}\npub struct Table"))
  .replace("Vec<Option<F::Id>>","storage::Slots<(F::Id,u16)>")
  .replace("Option<F::Id>","Option<(F::Id,u16)>")
  .replace("self.slots[i]=Some(u.payload());","self.slots.put(i,(u.payload(),7u16));")
  .replace("self.slots[i]","self.slots.get(i)")
  .replace("{t.add(u);}","{let _=t.slots.len();t.add(u);}");
 let r=verify_one_file("tuple_projection",source,&["vstd","--infer-goals"]);
 assert!(r.is_ok(),"{:?}",r);
}

#[test]
fn explicitly_borrowed_trait_results_are_outside_the_getter_profile(){
 let source=r#"use vstd::prelude::*;
 verus! {
  pub trait Borrowed {
   type Item;
   spec fn model(&self)->Option<&Self::Item>;
   fn get(&self)->(r:Option<&Self::Item>) ensures r==self.model();
  }
  pub struct State{pub n:u8}
  impl State{#[verifier::infer] pub fn inspect<T:Borrowed>(&mut self,t:&T){let _=t.get();}}
 }"#;
 unsupported("borrowed_trait",source.to_owned());
}
