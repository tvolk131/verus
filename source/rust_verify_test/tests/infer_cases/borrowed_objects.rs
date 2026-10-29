use super::*;

#[test]
fn borrowed_projection_chains_keep_complete_objects_across_imports() {
    let crates = InferCrates::new();
    let provider = r#"use vstd::prelude::*;
pub struct Buffers{pub left:Vec<u8>,pub right:Vec<u8>}
impl Buffers{
 #[verifier::infer] pub fn left(&self)->&Vec<u8>{&self.left}
}
pub struct Cached{data:Buffers,epoch:u8}
impl Cached{
 #[verifier::type_invariant] closed spec fn valid(&self)->bool{self.epoch>0}
 #[verifier::infer] pub fn buffers(&self)->&Buffers{&self.data}
 pub closed spec fn left_value(&self)->Vec<u8>{self.data.left}
 pub fn take_left(&self)->(r:&Vec<u8>) ensures r==self.left_value(),{self.buffers().left()}
}
"#;
    let consumer = r#"use vstd::prelude::*;
pub enum Error{Missing,Other}
#[verifier::infer] pub fn pass(c:borrow_provider::Cached)->Result<borrow_provider::Cached,Error>{Ok(c)}
pub fn goal(c:&borrow_provider::Cached)->(r:&Vec<u8>)
 ensures r==c.left_value(),
{c.take_left()}
pub fn repeated(c:&borrow_provider::Cached){
 let a=c.buffers().left();let b=c.buffers().left();assert(a==b);
}
pub fn preserved(c:borrow_provider::Cached)->(r:Result<borrow_provider::Cached,Error>)
 ensures r==Ok::<borrow_provider::Cached,Error>(c),
{pass(c)}
"#;
    for _ in 0..2 {
        for (name, source, deps, count) in [
            ("borrow_provider", provider, vec![], 2),
            ("borrow_consumer", consumer, vec!["borrow_provider"], 1),
        ] {
            let r = crates.build(name, source, &deps, &["--no-cheating", "--output-json"]);
            assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
            let report = json_inference(&r);
            let fs = report["functions"].as_array().unwrap();
            assert_eq!(fs.len(), count);
            for f in fs {
                for proof in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][proof]["status"], "proved", "{report}");
                }
            }
        }
    }
    let wrong = provider.replace("{&self.left}", "{&self.right}");
    let r = crates.build("borrow_provider", &wrong, &[], &["--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("borrow_provider").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}

#[test]
fn tagged_borrow_identity_crosses_private_provider_and_forwarding_interfaces(){
 let crates=InferCrates::new();
 let provider=r#"use vstd::prelude::*;
 pub struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
 pub struct Table<T>{pub left:Slots<T>,pub right:Slots<T>,pub ready:bool}
 impl<T> Table<T>{
  #[verifier::infer] pub fn select(&self)->Result<&Slots<T>,u8>{if self.ready{Ok(&self.left)}else{Err(4)}}
 }
"#;
 let consumer=r#"use vstd::prelude::*;
 #[verifier::infer] pub fn forward<T>(t:&tagged_base::Table<T>)->Result<&tagged_base::Slots<T>,u8>{t.select()}
 pub fn goal<T>(t:&tagged_base::Table<T>){let r=forward(t);assert(r.is_ok()==t.ready);
  if let Ok(value)=r{assert(*value==t.left);}}
"#;
 for _ in 0..2 {
  for (name,body,deps) in [("tagged_base",provider,vec![]),("tagged_app",consumer,vec!["tagged_base"])] {
   let r=crates.build(name,body,&deps,&["--no-cheating","--output-json"]);
   assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
   assert_eq!(json_inference(&r)["functions"][0]["required_checks_passed"],true);
  }
 }
 let wrong=provider.replace("Ok(&self.left)","Ok(&self.right)");
 let p=crates.build("tagged_base",&wrong,&[],&["--no-cheating","--output-json"]);
 assert!(p.status.success(),"{}",String::from_utf8_lossy(&p.stderr));
 let c=crates.build("tagged_app",consumer,&["tagged_base"],&["--no-cheating","--output-json"]);
 assert!(!c.status.success());assert!(!crates.vir("tagged_app").exists());
 assert_eq!(json_inference(&c)["functions"][0]["required_checks_passed"],true);
}

const OPAQUE_PROJECTIONS: &str = r#"use vstd::prelude::*;
pub mod hidden {
 use super::*;
 pub struct Slots<T>(Vec<T>);
 impl<T> View for Slots<T>{type V=Seq<T>;closed spec fn view(&self)->Self::V{self.0@}}
 impl<T> Slots<T>{#[verifier::infer] fn size(&self)->usize{self.0.len()}}
 pub struct Payload<T>{values:Slots<T>,stamp:u16}
 impl<T> View for Payload<T>{type V=(Seq<T>,u16);closed spec fn view(&self)->Self::V{(self.values@,self.stamp)}}
 impl<T> Payload<T>{#[verifier::infer] pub fn size(&self)->usize{self.values.size()}}
}
pub struct Packet<T>{payload:hidden::Payload<T>,round:u16}
impl<T> View for Packet<T>{type V=(u16,hidden::Payload<T>);closed spec fn view(&self)->Self::V{(self.round,self.payload)}}
impl<T> Packet<T>{
 #[verifier::infer] pub fn round(&self)->u16{self.round}
 #[verifier::infer] pub fn size(&self)->usize{self.payload.size()}
 #[verifier::infer] pub fn payload(&self)->&hidden::Payload<T>{&self.payload}
 #[verifier::infer] fn new(payload:hidden::Payload<T>,other:hidden::Payload<T>,round:u16)->Self{Self{payload,round}}
}
fn moved<T>(payload:hidden::Payload<T>,other:hidden::Payload<T>,round:u16)->(r:Packet<T>)
 ensures r.payload==payload,r.round==round,
{Packet::new(payload,other,round)}
fn accessors<T>(p:&Packet<T>){
 let n=p.size();assert(n==p@.1@.0.len());let round=p.round();assert(round==p@.0);
 let payload=p.payload();assert(*payload==p@.1);
}
"#;

#[test]
fn opaque_fields_use_checked_projection_views_and_complete_move_identity(){
 let crates=InferCrates::new();
 let caller=r#"use vstd::prelude::*;
 #[verifier::infer] pub fn size<T>(p:&opaque_values::Packet<T>)->usize{p.size()}
 #[verifier::infer] pub fn round<T>(p:&opaque_values::Packet<T>)->u16{p.round()}
 #[verifier::infer] pub fn payload<T>(p:&opaque_values::Packet<T>)->&opaque_values::hidden::Payload<T>{p.payload()}
 pub fn goal<T>(p:&opaque_values::Packet<T>){let n=size(p);assert(n==p@.1@.0.len());let r=round(p);assert(r==p@.0);let v=payload(p);assert(*v==p@.1);}
 "#;
 for _ in 0..2 {
  for (name,source,deps,count) in [("opaque_values",OPAQUE_PROJECTIONS,vec![],6),("opaque_caller",caller,vec!["opaque_values"],3)] {
   let r=crates.build(name,source,&deps,&["--no-cheating","--output-json"]);
   assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
   let report=json_inference(&r);let fs=report["functions"].as_array().unwrap();assert_eq!(fs.len(),count);
   assert!(fs.iter().all(|f|f["required_checks_passed"]==true),"{}",report);
  }
 }
 for wrong in [OPAQUE_PROJECTIONS.replace("Self{payload,round}","Self{payload:other,round}"),
   OPAQUE_PROJECTIONS.replace("{self.payload.size()}","{0}"),
   OPAQUE_PROJECTIONS.replace("{self.round}","{0}")] {
  let r=crates.build("opaque_values",&wrong,&[],&["--no-cheating","--output-json"]);
  assert!(!r.status.success());assert!(!crates.vir("opaque_values").exists());
  assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied")
   || String::from_utf8_lossy(&r.stderr).contains("assertion failed"));
  assert!(json_inference(&r)["functions"].as_array().unwrap().iter().all(|f|f["required_checks_passed"]==true));
 }
 for opts in [vec!["--no-verify"],vec!["--verify-root","--verify-function","round"]] {
  let failed=crates.build("opaque_values",OPAQUE_PROJECTIONS,&[],&opts);
  assert!(!failed.status.success());assert!(!crates.vir("opaque_values").exists());
  assert!(String::from_utf8_lossy(&failed.stderr).contains("requires full-crate verification"));
 }
 let missing=OPAQUE_PROJECTIONS.replace("impl<T> View for Payload<T>{type V=(Seq<T>,u16);closed spec fn view(&self)->Self::V{(self.values@,self.stamp)}}","");
 let missing=&missing[..missing.find("fn accessors<T>").unwrap()];
 let r=crates.build("opaque_values",missing,&[],&["--no-cheating"]);
 assert!(!r.status.success());assert!(!crates.vir("opaque_values").exists());
 assert!(String::from_utf8_lossy(&r.stderr).contains("private record formulas require an authored projection observation"));
 assert!(!String::from_utf8_lossy(&r.stderr).contains("panicked"));
}
