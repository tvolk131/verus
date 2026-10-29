use super::*;

const PROVIDER: &str = r#"use vstd::prelude::*;
pub struct Gate{pub permit:bool}
pub struct Envelope<T>{pub items:Vec<T>,pub sequence:u16}
pub struct Rejected<T>{pub input:Envelope<T>}
pub enum Error<T>{Denied(Rejected<T>),Expired(u16)}
impl<T> vstd::std_specs::convert::FromSpecImpl<Rejected<T>> for Error<T>{
 open spec fn obeys_from_spec()->bool{false}
 open spec fn from_spec(error:Rejected<T>)->Self{Error::Denied(error)}
}
impl<T> Envelope<T>{
 pub open spec fn checked_value(self,gate:&Gate)->Result<Self,Rejected<T>>{
  if gate.permit{Ok(self)}else{Err(Rejected{input:self})}
 }
 pub fn check(self,gate:&Gate)->(result:Result<Self,Rejected<T>>)
  ensures result==self.checked_value(gate),
 {if gate.permit{Ok(self)}else{Err(Rejected{input:self})}}
}
impl<T> From<Rejected<T>> for Error<T>{
 #[verifier::infer] fn from(error:Rejected<T>)->Self{Error::Denied(error)}
}
"#;

const CONSUMER: &str = r#"
#[verifier::infer] pub fn pipeline<T>(input:Envelope<T>,gate:&Gate,limit:u16)->Result<Envelope<T>,Error<T>>{
 let input=input.check(gate)?;
 if input.sequence>limit{return Err(Error::Expired(input.sequence));}
 Ok(input)
}
pub fn policy<T>(input:Envelope<T>,gate:&Gate,limit:u16)->(result:Result<Envelope<T>,Error<T>>)
 ensures match result{
  Err(Error::Denied(error))=>!gate.permit && error.input.items@==input.items@ && error.input.sequence==input.sequence,
  Err(Error::Expired(sequence))=>gate.permit && input.sequence>limit && sequence==input.sequence,
  Ok(value)=>gate.permit && input.sequence<=limit && value.items@==input.items@ && value.sequence==input.sequence,
 },
 {pipeline(input,gate,limit)}
"#;

#[test]
fn structured_queries_and_generic_from_preserve_complete_collection_payloads(){
 let source=format!("use vstd::prelude::*;verus!{{{PROVIDER}{CONSUMER}}}");
 let d=verify_one_file("structured_query_from",source,&["vstd","--no-cheating"]).unwrap();
 for p in ["soundness: PROVED","safe-domain maximality: PROVED","output exactness: PROVED"]{
  assert_eq!(d.notes.iter().filter(|n|n.message.contains(p)).count(),2,"{:?}",d);
 }
}

#[test]
fn resolved_trait_queries_retain_the_concrete_implementation(){
 let source=r#"use vstd::prelude::*;verus!{
 pub trait Query{
  spec fn allowed_value(&self)->bool;
  fn allowed(&self)->(r:bool) ensures r==self.allowed_value();
 }
 pub struct Gate{pub permit:bool}
 impl Query for Gate{
  open spec fn allowed_value(&self)->bool{self.permit}
  fn allowed(&self)->bool{self.permit}
 }
 #[verifier::infer] pub fn check(gate:&Gate)->Result<bool,u16>{Ok(gate.allowed())}
 fn goal(gate:&Gate)->(r:Result<bool,u16>) ensures r==Ok::<bool,u16>(gate.permit),{check(gate)}
}"#;
 verify_one_file("resolved_query",source.into(),&["vstd","--no-cheating"]).unwrap();
}

#[test]
fn structured_query_contracts_and_converted_payloads_are_checked(){
 for (name,source) in [
  ("wrong_query",PROVIDER.replace("{if gate.permit{Ok(self)}else{Err(Rejected{input:self})}}",
   "{Err(Rejected{input:self})}")),
  ("wrong_conversion",PROVIDER.replace("fn from(error:Rejected<T>)->Self{Error::Denied(error)}",
   "fn from(error:Rejected<T>)->Self{Error::Denied(Rejected{input:Envelope{items:error.input.items,sequence:0}})}")),
 ]{
  let e=verify_one_file(name,format!("use vstd::prelude::*;verus!{{{source}{CONSUMER}}}"),&["vstd","--no-cheating"]).unwrap_err();
  assert!(e.errors.iter().any(|d|d.message.contains("postcondition not satisfied")),"{:?}",e);
 }
}

#[test]
fn partial_and_unchecked_from_conversions_cannot_supply_result_propagation(){
 for (name,source) in [
  ("partial_from",PROVIDER.replace("fn from(error:Rejected<T>)->Self{Error::Denied(error)}",
   "fn from(error:Rejected<T>)->Self{Error::Expired(error.input.sequence+1)}")),
  ("unmarked_from",PROVIDER.replace("#[verifier::infer] fn from","fn from")),
 ]{
  let e=verify_one_file(name,format!("use vstd::prelude::*;verus!{{{source}{CONSUMER}}}"),&["vstd","--no-cheating"]).unwrap_err();
  assert!(e.errors.iter().any(|d|d.message.contains("inherited trait precondition")
   || d.message.contains("checked exact From implementation")),"{:?}",e);
 }
}

#[test]
fn structured_query_and_conversion_exports_are_rechecked_cold_and_warm(){
 let crates=InferCrates::new();
 let consumer=format!("use vstd::prelude::*;use structured_provider::*;{CONSUMER}");
 for _ in 0..2{
  let r=crates.build("structured_provider",PROVIDER,&[],&["--no-cheating","--output-json"]);
  assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
  let r=crates.build("structured_consumer",&consumer,&["structured_provider"],&["--no-cheating","--output-json"]);
  assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
  assert_eq!(json_inference(&r)["functions"][0]["proofs"]["exactness"]["status"],"proved");
 }
 let r=crates.build("structured_provider",PROVIDER,&[],&["--no-verify"]);
 assert!(!r.status.success());assert!(!crates.vir("structured_provider").exists());
 let wrong=PROVIDER.replace("Error::Denied(error)}","Error::Expired(error.input.sequence)}");
 let r=crates.build("structured_provider",&wrong,&[],&["--no-cheating"]);
 assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
 let r=crates.build("structured_consumer",&consumer,&["structured_provider"],&["--no-cheating"]);
 assert!(!r.status.success());assert!(!crates.vir("structured_consumer").exists());
 assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}

#[test]
fn larger_callers_keep_discovered_checked_view_bounds(){
 let crates=InferCrates::new();
 let provider=r#"use vstd::prelude::*;
 pub struct Slots(Vec<u8>);
 impl View for Slots{type V=Seq<u8>;closed spec fn view(&self)->Self::V{self.0@}}
 impl Slots{
  pub broadcast proof fn __verus_infer_view_len_bound(&self)
   ensures #[trigger] self@.len()<=usize::MAX as int,
  {vstd::std_specs::vec::axiom_spec_len(&self.0);}
  #[verifier::infer]pub fn len(&self)->usize{self.0.len()}
 }
 "#;
 let r=crates.build("bounds_provider",provider,&[],&["--no-cheating"]);
 assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
 let mut consumer="use vstd::prelude::*;use bounds_provider::Slots;".to_string();
 for i in 0..40{
  let tail=if i==39{"true".to_string()}else{format!("noise{}(s)",i+1)};
  consumer.push_str(&format!("pub closed spec fn noise{i}(s:&Slots)->bool{{{tail}}}\n"));
 }
 consumer.push_str("pub fn bound(s:&Slots) requires noise0(s), ensures s@.len()<=usize::MAX, {}\n");
 let r=crates.build("bounds_consumer",&consumer,&["bounds_provider"],&["--infer-goals","--no-cheating","--output-json"]);
 assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
 let report=json_inference(&r);
 assert!(report["goal_assistance"].as_array().unwrap().iter().any(|x|x["strategy"]=="sequence_view_bound"));
 let r=crates.build("bounds_consumer",&consumer,&["bounds_provider"],&["--no-cheating"]);
 assert!(!r.status.success());
 assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}
